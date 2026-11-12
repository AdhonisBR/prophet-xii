use prophet_control::*;
use prophet_params::ParamId::*;
#[test]
fn timestamped_midi_sustain_cc_bend_and_panic() {
    let mut p = ProphetXiiPlugin::new(48000.0);
    p.ipc.params.set(ReverbWet, 0.0);
    p.ipc.params.set(ChorusMode, 0.0);
    let (mut l, mut r) = ([0.0; 64], [0.0; 64]);
    p.process_events(
        &mut l,
        &mut r,
        &[MidiEvent {
            offset: 32,
            status: 0x90,
            data1: 60,
            data2: 100,
        }],
    )
    .unwrap();
    assert!(l[..32].iter().all(|&v| v == 0.0));
    assert_eq!(p.kernel.last_telemetry.active_voice_mask.count_ones(), 1);
    p.handle_midi_message(0xb0, 64, 127);
    p.handle_midi_message(0x80, 60, 0);
    assert!(p.kernel.allocator.voices[0].gate);
    p.handle_midi_message(0xb0, 64, 0);
    assert!(!p.kernel.allocator.voices[0].gate);
    p.handle_midi_message(0xe0, 127, 127);
    p.sync_parameters();
    assert!(p.kernel.voice_params.pitch_bend_semis > 6.9);
    p.handle_midi_message(0xb0, 1, 100);
    p.sync_parameters();
    assert!(p.kernel.global_lfo.mod_wheel > 0.7);
    p.handle_midi_message(0x90, 67, 127);
    p.handle_midi_message(0xb0, 123, 0);
    assert!(p.kernel.allocator.voices.iter().all(|v| !v.gate));
}
#[test]
fn invalid_events_and_nonfinite_params_are_rejected() {
    let mut p = ProphetXiiPlugin::new(48000.0);
    let (mut l, mut r) = ([0.0; 8], [0.0; 8]);
    assert!(p
        .process_events(
            &mut l,
            &mut r,
            &[MidiEvent {
                offset: 8,
                status: 0x90,
                data1: 60,
                data2: 127
            }]
        )
        .is_err());
    let v = p.ipc.params.get(FilterCutoff);
    p.ipc.params.set(FilterCutoff, f32::NAN);
    assert_eq!(p.ipc.params.get(FilterCutoff), v);
    p.ipc.params.set(FilterCutoff, -100.0);
    assert_eq!(p.ipc.params.get(FilterCutoff), 20.0);
}
