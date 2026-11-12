// SPDX-License-Identifier: GPL-3.0-or-later
// PROPHET-XII Plugin Wrapper (`prophet-plugin`) — MIDI/MPE Processor, Preset Manager & IPC Integration
// Reference: NIH-plug VST3/CLAP architecture (https://github.com/robbert-vdh/nih-plug)

pub mod ipc_bridge;
pub mod presets;

use core::sync::atomic::Ordering;
use ipc_bridge::{IpcUiEvent, ParamId, WebViewIpcBridge};
use presets::{factory_presets, ProphetPreset};
use prophet_dsp::{ProphetXiiKernel, MAX_BLOCK_SIZE};

/// Real-Time Plugin Instance uniting the `ProphetXiiKernel`, `WebViewIpcBridge`,
/// MIDI/MPE Per-Channel Expressive Router, and Calibrated Factory Presets.
pub struct ProphetXiiPlugin {
    pub kernel: ProphetXiiKernel,
    pub ipc: WebViewIpcBridge,
    pub presets: [ProphetPreset; 10],
    pub current_preset_idx: usize,
    /// MPE channel->voice mapping (`16` MIDI channels)
    mpe_channel_voice: [Option<usize>; 16],
}

impl Default for ProphetXiiPlugin {
    fn default() -> Self {
        Self::new(48_000.0)
    }
}

impl ProphetXiiPlugin {
    pub fn new(sample_rate: f32) -> Self {
        let presets = factory_presets();
        let mut plugin = Self {
            kernel: ProphetXiiKernel::new(sample_rate),
            ipc: WebViewIpcBridge::default(),
            presets,
            current_preset_idx: 0,
            mpe_channel_voice: [None; 16],
        };
        plugin.load_preset(0);
        plugin
    }

    /// Loads one of the 10 calibrated presets into both the atomic parameter bank and the DSP kernel.
    pub fn load_preset(&mut self, index: usize) {
        let idx = index.min(self.presets.len() - 1);
        self.current_preset_idx = idx;
        let p = &self.presets[idx];
        self.ipc.params.apply_struct_snapshot(
            &p.voice,
            &p.reverb,
            p.chorus_mode as f32,
            p.chorus_wet,
            2.0,
        );
        self.kernel.allocator.unison_enabled = p.unison_enabled;
        self.kernel.allocator.unison_voices = p.unison_voices;
        self.kernel.allocator.unison_detune_cents = p.unison_detune_cents;
    }

    /// Processes incoming raw 3-byte MIDI / MPE messages on the Audio Thread with zero allocations.
    pub fn handle_midi_message(&mut self, status: u8, data1: u8, data2: u8) {
        let cmd = status & 0xF0;
        let ch = (status & 0x0F) as usize;

        match cmd {
            0x90 if data2 > 0 => {
                let vel = (data2 as f32) / 127.0;
                self.kernel.allocator.note_on(data1, vel, &self.kernel.voice_params);
                // Record MPE member channel assignment (channels 1..15)
                for (v_idx, v) in self.kernel.allocator.voices.iter().enumerate() {
                    if v.midi_note == data1 && v.gate {
                        self.mpe_channel_voice[ch] = Some(v_idx);
                        break;
                    }
                }
            }
            0x80 | 0x90 => {
                self.kernel.allocator.note_off(data1, &self.kernel.voice_params);
                self.mpe_channel_voice[ch] = None;
            }
            0xB0 => {
                let cc = (data1 & 0x7F) as usize;
                let norm_val = (data2 as f32) / 127.0;

                // Check if MIDI Learn is waiting to bind a CC knob
                if self.ipc.params.midi_learn_active.load(Ordering::Relaxed) {
                    let target = self.ipc.params.midi_learn_target_param.load(Ordering::Relaxed);
                    if target < (ipc_bridge::NUM_PARAMS as u32) {
                        self.ipc.params.cc_map[cc].store(target, Ordering::Relaxed);
                        self.ipc.params.midi_learn_active.store(false, Ordering::Relaxed);
                    }
                }

                // MPE CC 74 (Slide / Timbre) per-voice routing on member channels 1..15
                if cc == 74 && ch > 0 {
                    if let Some(v_idx) = self.mpe_channel_voice[ch] {
                        self.kernel.allocator.voices[v_idx].mpe_timbre = norm_val - 0.5;
                        return;
                    }
                }

                let mapped_param = self.ipc.params.cc_map[cc].load(Ordering::Relaxed);
                if mapped_param == (ParamId::FilterCutoff as u32) {
                    // Exponential MIDI CC to Cutoff mapping (20 Hz .. 18,000 Hz)
                    let hz = 20.0 * (900.0f32).powf(norm_val);
                    self.ipc.params.set(ParamId::FilterCutoff, hz);
                } else if mapped_param < (ipc_bridge::NUM_PARAMS as u32) {
                    let pid: ParamId = unsafe { core::mem::transmute(mapped_param as usize) };
                    self.ipc.params.set(pid, norm_val);
                }
            }
            0xD0 => {
                // Channel Pressure / Aftertouch (Global on Ch 0, Per-Voice MPE on Ch 1..15)
                let pressure = (data1 as f32) / 127.0;
                if ch > 0 {
                    if let Some(v_idx) = self.mpe_channel_voice[ch] {
                        self.kernel.allocator.voices[v_idx].mpe_pressure = pressure;
                        return;
                    }
                }
                self.kernel.voice_params.aftertouch = pressure;
            }
            0xE0 => {
                // 14-bit Pitch Bend
                let raw14 = ((data2 as u16) << 7) | (data1 as u16);
                let norm_bipolar = ((raw14 as f32) - 8192.0) / 8192.0;
                if ch > 0 {
                    if let Some(v_idx) = self.mpe_channel_voice[ch] {
                        // MPE +/- 48 semitone per-note bend
                        self.kernel.allocator.voices[v_idx].mpe_pitch_bend_semis =
                            norm_bipolar * 48.0;
                        return;
                    }
                }
                // Master channel +/- 7 semitone bend
                self.kernel.voice_params.pitch_bend_semis = norm_bipolar * 7.0;
            }
            _ => {}
        }
    }

    /// Drains all lock-free UI events from the SPSC ring buffer and renders an audio block.
    pub fn process_audio_block(&mut self, out_l: &mut [f32], out_r: &mut [f32]) {
        while let Some(ev) = self.ipc.ui_to_audio_queue.pop() {
            match ev {
                IpcUiEvent::NoteOn { note, velocity, channel } => {
                    self.kernel.allocator.note_on(note, velocity, &self.kernel.voice_params);
                    let ch = (channel as usize) & 0x0F;
                    for (v_idx, v) in self.kernel.allocator.voices.iter().enumerate() {
                        if v.midi_note == note && v.gate {
                            self.mpe_channel_voice[ch] = Some(v_idx);
                            break;
                        }
                    }
                }
                IpcUiEvent::NoteOff { note, channel } => {
                    self.kernel.allocator.note_off(note, &self.kernel.voice_params);
                    self.mpe_channel_voice[(channel as usize) & 0x0F] = None;
                }
                IpcUiEvent::MpePitchBend { channel, semitones } => {
                    if let Some(v_idx) = self.mpe_channel_voice[(channel as usize) & 0x0F] {
                        self.kernel.allocator.voices[v_idx].mpe_pitch_bend_semis = semitones;
                    }
                }
                IpcUiEvent::MpePressure { channel, pressure } => {
                    if let Some(v_idx) = self.mpe_channel_voice[(channel as usize) & 0x0F] {
                        self.kernel.allocator.voices[v_idx].mpe_pressure = pressure;
                    }
                }
                IpcUiEvent::MpeTimbre { channel, timbre } => {
                    if let Some(v_idx) = self.mpe_channel_voice[(channel as usize) & 0x0F] {
                        self.kernel.allocator.voices[v_idx].mpe_timbre = timbre;
                    }
                }
                IpcUiEvent::SelectPreset { index } => {
                    self.load_preset(index);
                }
                IpcUiEvent::SetUnison { enabled, voices, detune, spread } => {
                    self.kernel.allocator.unison_enabled = enabled;
                    self.kernel.allocator.unison_voices = voices;
                    self.kernel.allocator.unison_detune_cents = detune;
                    self.kernel.allocator.unison_stereo_spread = spread;
                }
            }
        }

        // Pull atomic parameters into DSP kernel
        let (chorus_mode, chorus_wet, chorus_noise, os_factor, master_gain) = self
            .ipc
            .params
            .read_into_dsp(&mut self.kernel.voice_params, &mut self.kernel.reverb_params);

        self.kernel.chorus.mode = chorus_mode;
        self.kernel.chorus.wet_mix = chorus_wet;
        self.kernel.chorus.bbd_noise_level = chorus_noise;
        self.kernel.master_gain = master_gain;

        if self.kernel.allocator.voices[0].oversampler.factor != os_factor {
            self.kernel.prepare(self.kernel.sample_rate, os_factor);
        }

        // Sync individual voice trimpots from calibration drawer
        for i in 0..prophet_dsp::voice::NUM_VOICES {
            self.kernel.allocator.voices[i].trimpots.vco_a_cents =
                self.ipc.params.voice_vco_a_cents[i].load();
            self.kernel.allocator.voices[i].trimpots.vco_b_cents =
                self.ipc.params.voice_vco_b_cents[i].load();
            self.kernel.allocator.voices[i].trimpots.vcf_cutoff_pct =
                self.ipc.params.voice_vcf_pct[i].load();
            self.kernel.allocator.voices[i].trimpots.env_rc_pct =
                self.ipc.params.voice_env_rc_pct[i].load();
        }

        // Process in fixed-size 64-sample sub-blocks
        let total = out_l.len().min(out_r.len());
        let mut offset = 0;
        while offset < total {
            let chunk = (total - offset).min(MAX_BLOCK_SIZE);
            self.kernel.process_block(
                &mut out_l[offset..offset + chunk],
                &mut out_r[offset..offset + chunk],
                chunk,
            );
            offset += chunk;
        }

        // Push telemetry snapshot to UI ring buffer (non-blocking)
        let _ = self.ipc.audio_to_ui_telemetry.push(self.kernel.last_telemetry);
    }
}
