// SPDX-License-Identifier: GPL-3.0-only
use prophet_control::ProphetXiiPlugin;
use prophet_render::*;
use std::path::{Path, PathBuf};
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |key: &str| args.windows(2).find(|v| v[0] == key).map(|v| v[1].clone());
    if args.is_empty() || args.iter().any(|a| a == "--help") {
        println!("prophet-render --demo-all DIR [--sample-rate 48000] [--seconds 5]\nprophet-render --midi INPUT.mid --output OUTPUT.wav [--preset 0] [--tail 4]\nprophet-render --demo single-note --output OUTPUT.wav");
        return Ok(());
    }
    let sr: u32 = value("--sample-rate").unwrap_or("48000".into()).parse()?;
    if !(8000..=192000).contains(&sr) {
        return Err("sample rate must be 8000..192000".into());
    }
    if let Some(dir) = value("--demo-all") {
        std::fs::create_dir_all(&dir)?;
        for name in DEMOS {
            let mut e = ProphetXiiPlugin::new(sr as f32);
            let events = configure_demo(&mut e, name)?;
            let path = PathBuf::from(&dir).join(format!("{name}-{sr}.wav"));
            render(
                &mut e,
                &events,
                (value("--seconds").unwrap_or("5".into()).parse::<f64>()? * sr as f64) as u64,
                &path,
            )?;
            println!("{}", path.display());
        }
        return Ok(());
    }
    let mut e = ProphetXiiPlugin::new(sr as f32);
    e.load_preset(value("--preset").unwrap_or("0".into()).parse()?);
    let events = if let Some(midi) = value("--midi") {
        read_midi(Path::new(&midi), sr)?
    } else {
        configure_demo(&mut e, &value("--demo").unwrap_or("single-note".into()))?
    };
    let secs = value("--seconds")
        .map(|s| s.parse::<f64>())
        .transpose()?
        .unwrap_or_else(|| {
            events
                .last()
                .map(|e| e.frame as f64 / sr as f64)
                .unwrap_or(0.0)
                + value("--tail").and_then(|s| s.parse().ok()).unwrap_or(4.0)
        });
    let out = value("--output").unwrap_or("prophet.wav".into());
    render(
        &mut e,
        &events,
        (secs.max(0.0).min(3600.0) * sr as f64) as u64,
        Path::new(&out),
    )?;
    println!(
        "Rendered {out}: {sr} Hz stereo, {secs:0.3} s, {} MIDI events",
        events.len()
    );
    Ok(())
}
