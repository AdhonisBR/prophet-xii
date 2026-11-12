// SPDX-License-Identifier: GPL-3.0-only
use prophet_control::ProphetXiiPlugin;
use prophet_params::{ParamId, NUM_PARAMS};
use std::{error::Error, path::Path};
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[derive(Clone, Copy, Debug)]
pub struct TimedMidi {
    pub frame: u64,
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
}
/// Merge tracks before integrating tempo; supports metrical SMF formats 0 and 1.
pub fn read_midi(path: &Path, sample_rate: u32) -> Result<Vec<TimedMidi>> {
    use midly::{MetaMessage, Timing, TrackEventKind};
    let bytes = std::fs::read(path)?;
    let smf = midly::Smf::parse(&bytes)?;
    if smf.header.format == midly::Format::Sequential {
        return Err("SMF format 2 is not supported".into());
    }
    let ppq = match smf.header.timing {
        Timing::Metrical(v) => v.as_int() as f64,
        _ => return Err("SMPTE timing is not supported".into()),
    };
    let mut all = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u64;
        for event in track {
            tick += event.delta.as_int() as u64;
            all.push((tick, event.kind));
        }
    }
    all.sort_by_key(|v| v.0);
    let mut out = Vec::new();
    let (mut tick, mut seconds, mut tempo) = (0u64, 0.0f64, 500000.0);
    for (t, kind) in all {
        seconds += (t - tick) as f64 * tempo / (ppq * 1_000_000.0);
        tick = t;
        match kind {
            TrackEventKind::Meta(MetaMessage::Tempo(v)) => tempo = v.as_int() as f64,
            TrackEventKind::Midi { channel, message } => {
                let mut bytes = Vec::new();
                midly::live::LiveEvent::Midi { channel, message }.write(&mut bytes)?;
                if bytes.len() >= 2 {
                    out.push(TimedMidi {
                        frame: (seconds * sample_rate as f64).round() as u64,
                        status: bytes[0],
                        data1: bytes[1],
                        data2: *bytes.get(2).unwrap_or(&0),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(out)
}
pub fn configure_demo(engine: &mut ProphetXiiPlugin, name: &str) -> Result<Vec<TimedMidi>> {
    use prophet_params::ParamId::*;
    engine.load_preset(match name {
        "hard-sync" => 2,
        "moog-drive" => 3,
        "reverb-ensemble" => 6,
        "unison" => 7,
        "poly-mod" => 5,
        _ => 0,
    });
    engine.ipc.params.set(ChorusBbdNoise, 0.0);
    let mut notes = vec![60];
    match name {
        "single-note" => {
            engine.ipc.params.set(ReverbWet, 0.0);
            engine.ipc.params.set(ChorusMode, 0.0);
        }
        "twelve-voices" => notes = (48..60).collect(),
        "curtis-resonance" => {
            engine.ipc.params.set(FilterResonance, 0.9);
            engine.ipc.params.set(FilterEnvAmt, 0.1);
        }
        "chorus-i" | "chorus-ii" | "chorus-i-ii" => {
            engine.ipc.params.set(
                ChorusMode,
                if name == "chorus-i" {
                    1.0
                } else if name == "chorus-ii" {
                    2.0
                } else {
                    3.0
                },
            );
            engine.ipc.params.set(ChorusWet, 0.8);
            engine.ipc.params.set(ReverbWet, 0.0);
            notes = vec![48, 55, 60, 64];
        }
        "reverb-ensemble" => {
            notes = vec![48, 55, 60, 64];
            engine.ipc.params.set(AmpAttack, 0.02);
            engine.ipc.params.set(AmpRelease, 0.15);
        }
        "hard-sync" | "moog-drive" | "unison" | "poly-mod" => {}
        _ => return Err(format!("unknown demo {name}").into()),
    }
    engine.sync_parameters();
    let sr = engine.kernel.sample_rate as u64;
    let mut ev = Vec::new();
    for n in notes {
        ev.push(TimedMidi {
            frame: 0,
            status: 0x90,
            data1: n,
            data2: 100,
        });
        ev.push(TimedMidi {
            frame: sr * 2,
            status: 0x80,
            data1: n,
            data2: 0,
        });
    }
    ev.sort_by_key(|e| e.frame);
    Ok(ev)
}
pub const DEMOS: [&str; 11] = [
    "single-note",
    "twelve-voices",
    "hard-sync",
    "curtis-resonance",
    "moog-drive",
    "chorus-i",
    "chorus-ii",
    "chorus-i-ii",
    "reverb-ensemble",
    "unison",
    "poly-mod",
];
pub fn render(
    engine: &mut ProphetXiiPlugin,
    events: &[TimedMidi],
    frames: u64,
    path: &Path,
) -> Result<()> {
    let sr = engine.kernel.sample_rate as u32;
    let mut wav = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: sr,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )?;
    let (mut pos, mut idx) = (0u64, 0usize);
    let (mut l, mut r) = ([0.0; 64], [0.0; 64]);
    while pos < frames {
        while idx < events.len() && events[idx].frame <= pos {
            let e = events[idx];
            engine.handle_midi_message(e.status, e.data1, e.data2);
            idx += 1;
        }
        let next = events
            .get(idx)
            .map(|e| e.frame)
            .unwrap_or(frames)
            .min(frames);
        let n = (next - pos).min(64) as usize;
        if n == 0 {
            continue;
        }
        engine.process_audio_block(&mut l[..n], &mut r[..n]);
        for i in 0..n {
            if !l[i].is_finite() || !r[i].is_finite() {
                return Err(format!("nonfinite output at {}", pos + i as u64).into());
            }
            wav.write_sample((l[i].clamp(-1.0, 1.0) * 32767.0).round() as i16)?;
            wav.write_sample((r[i].clamp(-1.0, 1.0) * 32767.0).round() as i16)?;
        }
        pos += n as u64;
    }
    wav.finalize()?;
    Ok(())
}
pub fn save_state(engine: &ProphetXiiPlugin, path: &Path) -> Result<()> {
    // Portable bounded text format, separate from network protocol and plugin host state.
    let mut s = String::from("PROPHET-XII-STATE 1\n");
    for i in 0..NUM_PARAMS {
        s.push_str(&format!(
            "{} {}\n",
            i,
            engine.ipc.params.get(ParamId::from_index(i).unwrap())
        ));
    }
    std::fs::write(path, s)?;
    Ok(())
}
