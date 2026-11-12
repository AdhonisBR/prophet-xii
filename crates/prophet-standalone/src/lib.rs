// SPDX-License-Identifier: GPL-3.0-only
use prophet_control::ProphetXiiPlugin;
use prophet_dsp::{KernelTelemetrySnapshot, MAX_BLOCK_SIZE};
use prophet_params::{ParamId, NUM_PARAMS};
use rtrb::{Consumer, Producer};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};
#[derive(Clone, Copy, Debug)]
pub enum Command {
    Param {
        id: usize,
        value: f32,
    },
    Midi {
        status: u8,
        data1: u8,
        data2: u8,
    },
    Preset(usize),
    Snapshot([f32; NUM_PARAMS]),
    Learn(usize),
    Panic,
    Trimpot {
        voice: usize,
        field: usize,
        value: f32,
    },
}
#[derive(Clone, Copy)]
pub struct Telemetry {
    pub kernel: KernelTelemetrySnapshot,
    pub scope: [f32; 1024],
    pub cpu: f32,
    pub peak_cpu: f32,
    pub deadline_misses: u64,
    pub frames: u64,
    pub sample_rate: f32,
    pub params: [f32; NUM_PARAMS],
    pub trimpots: [[f32; 4]; 12],
    pub late_commands: u64,
    pub dropped_telemetry: u64,
}
pub struct AudioProcessor {
    pub engine: ProphetXiiPlugin,
    commands: Consumer<Command>,
    telemetry: Producer<Telemetry>,
    scope: [f32; 1024],
    scope_pos: usize,
    pub frames: u64,
    last_publish: u64,
    cpu: f32,
    peak_cpu: f32,
    deadline_misses: u64,
    late_commands: u64,
    dropped_telemetry: u64,
    pub underruns: Arc<AtomicU64>,
}
impl AudioProcessor {
    pub fn new(
        sr: f32,
        commands: Consumer<Command>,
        telemetry: Producer<Telemetry>,
        underruns: Arc<AtomicU64>,
    ) -> Self {
        Self {
            engine: ProphetXiiPlugin::new(sr),
            commands,
            telemetry,
            scope: [0.0; 1024],
            scope_pos: 0,
            frames: 0,
            last_publish: 0,
            cpu: 0.0,
            peak_cpu: 0.0,
            deadline_misses: 0,
            late_commands: 0,
            dropped_telemetry: 0,
            underruns,
        }
    }
    pub fn command(&mut self, c: Command) {
        match c {
            Command::Param { id, value } => {
                if let Some(id) = ParamId::from_index(id) {
                    self.engine.ipc.params.set(id, value)
                }
            }
            Command::Midi {
                status,
                data1,
                data2,
            } => self.engine.handle_midi_message(status, data1, data2),
            Command::Preset(i) => {
                self.engine.panic();
                self.engine.load_preset(i)
            }
            Command::Snapshot(values) => {
                for (i, v) in values.into_iter().enumerate() {
                    self.engine
                        .ipc
                        .params
                        .set(ParamId::from_index(i).unwrap(), v)
                }
            }
            Command::Learn(id) => {
                if id < NUM_PARAMS {
                    self.engine
                        .ipc
                        .params
                        .midi_learn_target_param
                        .store(id as u32, Ordering::Relaxed);
                    self.engine
                        .ipc
                        .params
                        .midi_learn_active
                        .store(true, Ordering::Relaxed);
                }
            }
            Command::Panic => self.engine.panic(),
            Command::Trimpot {
                voice,
                field,
                value,
            } => {
                if voice < 12 && value.is_finite() {
                    let p = &self.engine.ipc.params;
                    match field {
                        0 => p.voice_vco_a_cents[voice].store(value.clamp(-6.0, 6.0)),
                        1 => p.voice_vco_b_cents[voice].store(value.clamp(-6.0, 6.0)),
                        2 => p.voice_vcf_pct[voice].store(value.clamp(-0.05, 0.05)),
                        3 => p.voice_env_rc_pct[voice].store(value.clamp(-0.1, 0.1)),
                        _ => {}
                    }
                }
            }
        }
    }
    /// Called only by the audio backend. Bounded control draining, no heap allocation or locks.
    pub fn process<T: cpal::Sample + cpal::FromSample<f32>>(
        &mut self,
        output: &mut [T],
        channels: usize,
    ) {
        let start = Instant::now();
        for _ in 0..256 {
            match self.commands.pop() {
                Ok(c) => self.command(c),
                Err(_) => break,
            }
        }
        if self.commands.slots() > 0 {
            self.late_commands += 1;
        }
        let mut l = [0.0; MAX_BLOCK_SIZE];
        let mut r = [0.0; MAX_BLOCK_SIZE];
        let frames = output.len() / channels;
        let mut pos = 0;
        while pos < frames {
            let n = (frames - pos).min(MAX_BLOCK_SIZE);
            self.engine.process_audio_block(&mut l[..n], &mut r[..n]);
            for i in 0..n {
                self.scope[self.scope_pos] = l[i];
                self.scope_pos = (self.scope_pos + 1) % 1024;
                for c in 0..channels {
                    output[(pos + i) * channels + c] = T::from_sample(if c == 0 {
                        l[i]
                    } else if c == 1 {
                        r[i]
                    } else {
                        0.0
                    });
                }
            }
            pos += n;
        }
        self.frames += frames as u64;
        let budget = frames as f64 / self.engine.kernel.sample_rate as f64;
        let load = (start.elapsed().as_secs_f64() / budget * 100.0) as f32;
        self.cpu = self.cpu * 0.9 + load * 0.1;
        self.peak_cpu = self.peak_cpu.max(load);
        if load > 100.0 {
            self.deadline_misses += 1;
        }
        if self.frames - self.last_publish >= self.engine.kernel.sample_rate as u64 / 25 {
            let mut scope = [0.0; 1024];
            for (i, x) in scope.iter_mut().enumerate() {
                *x = self.scope[(self.scope_pos + i) % 1024];
            }
            let p = &self.engine.ipc.params;
            let t = Telemetry {
                kernel: self.engine.kernel.last_telemetry,
                scope,
                cpu: self.cpu,
                peak_cpu: self.peak_cpu,
                deadline_misses: self.deadline_misses,
                frames: self.frames,
                sample_rate: self.engine.kernel.sample_rate,
                params: core::array::from_fn(|i| p.get(ParamId::from_index(i).unwrap())),
                trimpots: core::array::from_fn(|i| {
                    [
                        p.voice_vco_a_cents[i].load(),
                        p.voice_vco_b_cents[i].load(),
                        p.voice_vcf_pct[i].load(),
                        p.voice_env_rc_pct[i].load(),
                    ]
                }),
                late_commands: self.late_commands,
                dropped_telemetry: self.dropped_telemetry,
            };
            if self.telemetry.push(t).is_err() {
                self.dropped_telemetry += 1;
            }
            self.last_publish = self.frames;
        }
    }
}
