use prophet_render::*;
#[test]
fn smf_tempo_changes_and_wav() {
    use midly::{
        Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
    };
    let path = std::env::temp_dir().join(format!("prophet-{}.mid", std::process::id()));
    let smf = Smf {
        header: Header::new(Format::SingleTrack, Timing::Metrical(480.into())),
        tracks: vec![vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn {
                        key: 60.into(),
                        vel: 100.into(),
                    },
                },
            },
            TrackEvent {
                delta: 480.into(),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(1000000.into())),
            },
            TrackEvent {
                delta: 480.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOff {
                        key: 60.into(),
                        vel: 0.into(),
                    },
                },
            },
        ]],
    };
    smf.save(&path).unwrap();
    let events = read_midi(&path, 48000).unwrap();
    assert_eq!(events[1].frame, 72000);
    let wav = path.with_extension("wav");
    let mut e = prophet_control::ProphetXiiPlugin::new(48000.0);
    render(&mut e, &events, 96000, &wav).unwrap();
    let mut reader = hound::WavReader::open(&wav).unwrap();
    assert_eq!(reader.spec().channels, 2);
    assert_eq!(reader.duration(), 96000);
    assert!(reader.samples::<i16>().any(|v| v.unwrap().abs() > 100));
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(wav).unwrap();
}
