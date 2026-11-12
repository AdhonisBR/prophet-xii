// SPDX-License-Identifier: GPL-3.0-only
use prophet_dsp::{
    fx::solaris_reverb::*, osc_cem3340::*, oversample::*, vcf_cem3320::*, vcf_moog::*,
};
fn wav(path: &str, sr: u32, a: &[f32]) {
    let mut w = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 1,
            sample_rate: sr,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for &x in a {
        w.write_sample(x).unwrap();
    }
    w.finalize().unwrap();
}
fn main() {
    let dir = std::env::args().nth(1).unwrap_or("evidence/probes".into());
    std::fs::create_dir_all(&dir).unwrap();
    for sr in [48000u32, 96000] {
        for name in ["naive", "blep-a", "blep-b", "triangle", "sync"] {
            let mut osc = Cem3340DualVco::new(sr as f32, 0.0, 0.0);
            let mut phase = 0.0f32;
            let f = sr as f32 * 853.0 / 16384.0;
            let a: Vec<f32> = (0..16384)
                .map(|_| {
                    let naive = 2.0 * phase - 1.0;
                    phase = (phase + f / sr as f32).fract();
                    let (a, b) = osc.process_sample(
                        if name == "sync" { f * 3.71 } else { f },
                        f,
                        0.35,
                        0.5,
                        OscAWaveforms {
                            saw: true,
                            pulse: false,
                        },
                        OscBWaveforms {
                            saw: name != "triangle",
                            triangle: name == "triangle",
                            pulse: false,
                            lo_freq: false,
                            key_tracking: true,
                        },
                        name == "sync",
                    );
                    match name {
                        "naive" => naive,
                        "blep-b" | "triangle" => b,
                        _ => a,
                    }
                })
                .collect();
            wav(&format!("{dir}/{name}-{sr}.wav"), sr, &a);
        }
        for moog in [false, true] {
            for drive in [0.25, 2.0] {
                let mut c = Cem3320Filter::new(sr as f32, 1.0);
                let mut m = MoogLadderFilter::new(sr as f32, 1.0);
                c.set_params(2000.0, 0.25, drive);
                m.set_params(2000.0, 0.25, drive, 0.5);
                let impulse: Vec<_> = (0..16384)
                    .map(|i| {
                        if moog {
                            m.process_sample(if i == 0 { 1e-4 } else { 0.0 }) / 1e-4
                        } else {
                            c.process_sample(if i == 0 { 1e-4 } else { 0.0 }) / 1e-4
                        }
                    })
                    .collect();
                let name = if moog { "moog" } else { "curtis" };
                wav(
                    &format!("{dir}/impulse-{name}-{drive}-{sr}.wav"),
                    sr,
                    &impulse,
                );
                let mut c = Cem3320Filter::new(sr as f32, 1.0);
                let mut m = MoogLadderFilter::new(sr as f32, 1.0);
                c.set_params(8000.0, 0.2, drive);
                m.set_params(8000.0, 0.2, drive, 0.5);
                let thd: Vec<_> = (0..32768)
                    .map(|i| {
                        let x = (std::f32::consts::TAU * 257.0 * i as f32 / 32768.0).sin() * 0.6;
                        if moog {
                            m.process_sample(x)
                        } else {
                            c.process_sample(x)
                        }
                    })
                    .collect();
                wav(&format!("{dir}/thd-{name}-{drive}-{sr}.wav"), sr, &thd);
                let imd: Vec<_> = (0..32768)
                    .map(|i| {
                        let x = ((std::f32::consts::TAU * 257.0 * i as f32 / 32768.0).sin()
                            + (std::f32::consts::TAU * 997.0 * i as f32 / 32768.0).sin())
                            * 0.3;
                        if moog {
                            m.process_sample(x)
                        } else {
                            c.process_sample(x)
                        }
                    })
                    .collect();
                wav(&format!("{dir}/imd-{name}-{drive}-{sr}.wav"), sr, &imd);
            }
        }
        for factor in [
            OversampleFactor::X1,
            OversampleFactor::X2,
            OversampleFactor::X4,
        ] {
            let mut os = VoiceOversampler::new(factor);
            let mut i = 0u64;
            let a: Vec<_> = (0..16384)
                .map(|_| {
                    os.process_voice_sample(|| {
                        let phase = std::f64::consts::TAU * 1973.0 * i as f64
                            / (16384.0 * factor.as_usize() as f64);
                        i += 1;
                        fast_tanh((phase.sin() * 5.0) as f32)
                    })
                })
                .collect();
            wav(
                &format!("{dir}/nonlinear-os{}-{sr}.wav", factor.as_usize()),
                sr,
                &a,
            );
        }
        let mut rev = SolarisCloudburstReverb::new(sr as f32);
        let mut p = SolarisReverbParams::default();
        p.wet_mix = 1.0;
        p.shimmer_amount = 0.0;
        p.ensemble_mode = EnsembleMode::Off;
        p.decay_sec = 1.5;
        let a: Vec<_> = (0..sr * 5)
            .map(|i| {
                let (l, r) = rev.process_stereo(if i == 0 { 1.0 } else { 0.0 }, 0.0, &p);
                (l + r) * 0.5
            })
            .collect();
        wav(&format!("{dir}/reverb-impulse-{sr}.wav"), sr, &a);
    }
}
