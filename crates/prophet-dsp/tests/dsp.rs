// SPDX-License-Identifier: GPL-3.0-only
use prophet_dsp::{
    fx::{juno_chorus::*, solaris_reverb::*},
    oversample::*,
    vcf_cem3320::*,
    vcf_moog::*,
    voice::*,
    *,
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
struct Count;
thread_local! {static ACTIVE:Cell<bool>=const{Cell::new(false)};static ALLOCS:Cell<usize>=const{Cell::new(0)};}
unsafe impl GlobalAlloc for Count {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ACTIVE.with(|a| {
            if a.get() {
                ALLOCS.with(|n| n.set(n.get() + 1));
            }
        });
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ACTIVE.with(|a| {
            if a.get() {
                ALLOCS.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static ALLOCATOR: Count = Count;
fn block(k: &mut ProphetXiiKernel, n: usize) -> Vec<f32> {
    let mut out = Vec::new();
    let (mut l, mut r) = ([0.0; 64], [0.0; 64]);
    for _ in 0..n {
        k.process_block(&mut l, &mut r, 64);
        out.extend_from_slice(&l);
    }
    out
}
#[test]
fn twelve_voices_steal_and_release() {
    let mut k = ProphetXiiKernel::new(48000.0);
    k.reverb_params.wet_mix = 0.0;
    k.chorus.mode = JunoChorusMode::Off;
    for n in 48..60 {
        k.allocator.note_on(n, 0.9, &k.voice_params);
    }
    let audio = block(&mut k, 300);
    assert_eq!(k.last_telemetry.active_voice_mask.count_ones(), 12);
    assert!(audio.iter().any(|v| v.abs() > 0.01));
    k.allocator.note_on(80, 0.8, &k.voice_params);
    assert_eq!(k.allocator.voices[0].midi_note, 80);
    assert!(k.allocator.voices[0].steal_fade_remaining > 0);
    k.allocator.all_notes_off();
    block(&mut k, 1400);
    assert_eq!(k.last_telemetry.active_voice_mask, 0);
}
#[test]
fn deterministic_and_block_partition_invariant() {
    for sr in [48000.0, 96000.0] {
        let mut a = ProphetXiiKernel::new(sr);
        let mut b = ProphetXiiKernel::new(sr);
        a.allocator.note_on(60, 0.8, &a.voice_params);
        b.allocator.note_on(60, 0.8, &b.voice_params);
        let expected = block(&mut a, 128);
        let mut actual = Vec::new();
        let mut pos = 0;
        while pos < expected.len() {
            let n = (expected.len() - pos).min([1, 7, 31, 64, 13][pos % 5]);
            let (mut l, mut r) = ([0.0; 64], [0.0; 64]);
            b.process_block(&mut l, &mut r, n);
            actual.extend_from_slice(&l[..n]);
            pos += n;
        }
        assert_eq!(expected, actual);
    }
}
#[test]
fn no_callback_allocations() {
    let mut k = ProphetXiiKernel::new(48000.0);
    k.reverb_params.ensemble_mode = EnsembleMode::Forte;
    for n in 48..60 {
        k.allocator.note_on(n, 1.0, &k.voice_params);
    }
    let (mut l, mut r) = ([0.0; 64], [0.0; 64]);
    ALLOCS.with(|n| n.set(0));
    ACTIVE.with(|a| a.set(true));
    for _ in 0..200 {
        k.process_block(&mut l, &mut r, 64);
    }
    k.allocator.note_on(80, 0.8, &k.voice_params);
    k.prepare(48000.0, OversampleFactor::X4);
    k.process_block(&mut l, &mut r, 64);
    ACTIVE.with(|a| a.set(false));
    assert_eq!(ALLOCS.with(|n| n.get()), 0);
}
#[test]
fn hadamard_is_orthogonal() {
    for i in 0..8 {
        let mut v = [0.0; 8];
        v[i] = 1.0;
        let result = hadamard_8x8(hadamard_8x8(v));
        for j in 0..8 {
            assert!((result[j] - v[j]).abs() < 1e-6);
        }
    }
}
#[test]
fn oversampler_dc_latency_stopband() {
    for factor in [OversampleFactor::X2, OversampleFactor::X4] {
        let mut o = VoiceOversampler::new(factor);
        let mut value = 0.0;
        for _ in 0..200 {
            value = o.process_voice_sample(|| 1.0);
        }
        assert!((value - 1.0).abs() < 1e-5);
    }
    let mut stage = HalfBandStage::new();
    let mut energy = 0.0;
    for i in 0..4096 {
        let a = (std::f32::consts::TAU * 0.4 * (2 * i) as f32).sin();
        let b = (std::f32::consts::TAU * 0.4 * (2 * i + 1) as f32).sin();
        let y = stage.downsample_sample(a, b);
        if i > 100 {
            energy += y * y;
        }
    }
    assert!(energy / 3995.0 < 1e-7, "stopband energy {energy}");
    let mut stage = HalfBandStage::new();
    let impulse: Vec<_> = (0..80)
        .map(|i| stage.downsample_sample(if i == 0 { 1.0 } else { 0.0 }, 0.0))
        .collect();
    let peak = impulse
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .unwrap()
        .0;
    assert_eq!(peak, 16);
}
#[test]
fn filter_stress_and_actual_feedback_solution() {
    for sr in [48000.0, 96000.0] {
        let mut c = Cem3320Filter::new(sr, 1.0);
        let mut m = MoogLadderFilter::new(sr, 1.0);
        for i in 0..20000 {
            let fc = 20.0 * (sr * 0.45 / 20.0).powf((i % 2000) as f32 / 1999.0);
            let res = (i % 100) as f32 / 99.0;
            c.set_params(fc, res, 2.0);
            m.set_params(fc, res, 2.0, 0.8);
            let x = (i as f32 * 0.033).sin() * 1.2;
            let a = c.process_sample(x);
            let b = m.process_sample(x);
            assert!(a.is_finite() && a.abs() < 10.0, "Curtis {a}");
            assert!(b.is_finite() && b.abs() < 10.0, "Moog {b}");
        }
    }
}
#[test]
fn chorus_silence_and_stereo_modes() {
    for mode in [
        JunoChorusMode::ModeI,
        JunoChorusMode::ModeII,
        JunoChorusMode::ModeIPlusII,
    ] {
        let mut c = Juno106BbdChorus::new(48000.0);
        c.mode = mode;
        c.bbd_noise_level = 0.0;
        for _ in 0..1000 {
            assert_eq!(c.process_stereo(0.0, 0.0), (0.0, 0.0));
        }
        let mut diff = 0.0;
        for i in 0..48000 {
            let x = (i as f32 * 0.071).sin() * 0.2;
            let (l, r) = c.process_stereo(x, x);
            assert!(l.is_finite() && r.is_finite());
            diff += (l - r) * (l - r);
        }
        assert!(diff > 1.0);
    }
}
#[test]
fn reverb_decay_and_freeze_bounded() {
    let mut r = SolarisCloudburstReverb::new(48000.0);
    let mut p = SolarisReverbParams::default();
    p.wet_mix = 1.0;
    p.pre_delay_ms = 0.0;
    p.shimmer_amount = 0.0;
    p.ensemble_mode = EnsembleMode::Off;
    p.decay_sec = 0.5;
    let (mut early, mut late) = (0.0, 0.0);
    for i in 0..144000 {
        let (l, rr) = r.process_stereo(if i == 0 { 1.0 } else { 0.0 }, 0.0, &p);
        assert!(l.is_finite() && rr.is_finite());
        if (4800..24000).contains(&i) {
            early += l * l + rr * rr;
        }
        if i > 96000 {
            late += l * l + rr * rr;
        }
    }
    assert!(early > 1e-6);
    assert!(late < early * 1e-4, "{early} {late}");
    p.freeze = true;
    p.shimmer_amount = 0.85;
    for _ in 0..48000 {
        let (l, rr) = r.process_stereo(1.0, 1.0, &p);
        assert!(l.is_finite() && rr.is_finite() && l.abs() < 8.0 && rr.abs() < 8.0);
    }
}
#[test]
fn adsr_times_and_noteoff() {
    use prophet_dsp::env_cem3310::*;
    for sr in [48000.0, 96000.0] {
        let mut e = Cem3310Envelope::new(sr, 1.0);
        e.set_adsr(0.01, 0.1, 0.5, 0.1);
        e.note_on(true);
        for _ in 0..(sr * 0.01) as usize {
            e.process_sample();
        }
        assert!(e.level > 0.99);
        e.note_off();
        for _ in 0..(sr * 0.2) as usize {
            e.process_sample();
        }
        assert_eq!(e.stage, EnvStage::Idle);
    }
}
#[test]
fn unison_one_to_twelve_priorities() {
    for count in 1..=12 {
        let mut a = ProphetVoiceAllocator::new(48000.0);
        let p = SynthVoiceParams::default();
        a.unison_enabled = true;
        a.unison_voices = count;
        a.note_priority = NotePriority::Low;
        a.note_on(60, 0.8, &p);
        a.note_on(64, 0.8, &p);
        assert_eq!(a.voices.iter().filter(|v| v.gate).count(), count);
        assert!(a.voices[..count].iter().all(|v| v.midi_note == 60));
        a.note_off(60, &p);
        assert!(a.voices[..count].iter().all(|v| v.midi_note == 64));
        a.note_off(64, &p);
        assert!(a.voices.iter().all(|v| !v.gate));
    }
}

#[test]
fn zdf_equations_have_small_residuals() {
    for sr in [48000.0f32, 96000.0] {
        let mut c = Cem3320Filter::new(sr, 1.0);
        let mut m = MoogLadderFilter::new(sr, 1.0);
        for fc in [80.0, 2000.0, sr * 0.4] {
            c.set_params(fc, 0.85, 1.8);
            m.set_params(fc, 0.85, 1.8, 0.5);
            let g = (std::f32::consts::PI * fc / sr).tan();
            let big_g = g / (1.0 + g);
            let dcg = (std::f32::consts::PI * 5.0 / sr).tan();
            let dcg = dcg / (1.0 + dcg);
            for i in 0..256 {
                let x = (i as f32 * 0.13).sin() * 0.8;
                let old = c.s;
                let old_dc = c.dc_block_state;
                let y = c.process_sample(x);
                let u = fast_tanh(x * 1.8) / 1.8
                    - 4.5 * 0.85 * fast_tanh(1.8 * (1.0 - dcg) * (y - old_dc)) / 1.8;
                let mut expected = u;
                for state in old {
                    expected = big_g * expected + (1.0 - big_g) * state;
                }
                assert!(
                    (expected - y).abs() < 2e-5,
                    "Curtis residual {}",
                    expected - y
                );
                let old = m.s;
                let y = m.process_sample(x);
                let ys = std::array::from_fn::<_, 4, _>(|i| (m.s[i] + old[i]) * 0.5);
                let mut input = x * (1.0 + 4.35 * 0.85 * 0.5 * 0.65) - 4.35 * 0.85 * y;
                for j in 0..4 {
                    let residual = ys[j]
                        - old[j]
                        - g * (fast_tanh(1.8 * input) - fast_tanh(1.8 * ys[j])) / 1.8;
                    assert!(residual.abs() < 2e-5, "Moog residual {residual}, fc {fc}");
                    input = ys[j];
                }
            }
        }
    }
}
