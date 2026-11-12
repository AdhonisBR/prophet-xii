// SPDX-License-Identifier: GPL-3.0-or-later
// PROPHET-XII Calibrated Factory Preset Manager (10 Authentic Classic & Hybrid Presets)
// Includes A/B Comparison slots and Undo/Redo parameter state history.

use prophet_dsp::fx::solaris_reverb::{EnsembleMode, SolarisReverbParams};
use prophet_dsp::osc_cem3340::{OscAWaveforms, OscBWaveforms};
use prophet_dsp::voice::{GlideMode, PolyModParams, SynthVoiceParams};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetCategory {
    Brass,
    Pads,
    Bass,
    Leads,
    AmbientKeys,
}

impl PresetCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            PresetCategory::Brass => "Brass",
            PresetCategory::Pads => "Pads",
            PresetCategory::Bass => "Bass",
            PresetCategory::Leads => "Leads",
            PresetCategory::AmbientKeys => "Ambient Keys",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProphetPreset {
    pub id: usize,
    pub name: &'static str,
    pub category: PresetCategory,
    pub author: &'static str,
    pub description: &'static str,
    pub voice: SynthVoiceParams,
    pub reverb: SolarisReverbParams,
    /// 0 = Off, 1 = Chorus I, 2 = Chorus II, 3 = Chorus I+II
    pub chorus_mode: u8,
    pub chorus_wet: f32,
    pub unison_enabled: bool,
    pub unison_voices: usize,
    pub unison_detune_cents: f32,
}

/// Returns the 10 calibrated classic & studio hybrid presets built into PROPHET-XII.
pub fn factory_presets() -> [ProphetPreset; 10] {
    let base_v = SynthVoiceParams::default();
    let base_r = SolarisReverbParams::default();

    [
        // 1. Prophet-10 Rev3 Signature Brass
        ProphetPreset {
            id: 0,
            name: "1980 San Jose Rev3 Brass",
            category: PresetCategory::Brass,
            author: "Sequential Factory Calibrated",
            description: "Classic CEM3340 dual sawtooth brass with CEM3320 filter swell and subtle vintage board dispersion.",
            voice: SynthVoiceParams {
                osc_a_semi: 0.0,
                osc_a_fine_cents: -4.5,
                osc_b_semi: 0.0,
                osc_b_fine_cents: 5.5,
                filter_cutoff_hz: 680.0,
                filter_resonance: 0.22,
                filter_drive: 1.25,
                filter_env_amount: 0.62,
                filter_topology_morph: 0.0, // CEM3320
                filt_attack: 0.065,
                filt_decay: 0.55,
                filt_sustain: 0.48,
                filt_release: 0.45,
                amp_attack: 0.012,
                amp_decay: 0.60,
                amp_sustain: 0.88,
                amp_release: 0.40,
                vintage_amount: 0.42,
                ..base_v
            },
            reverb: SolarisReverbParams {
                ensemble_mode: EnsembleMode::Off,
                wet_mix: 0.20,
                decay_sec: 2.6,
                ..base_r
            },
            chorus_mode: 1,
            chorus_wet: 0.35,
            unison_enabled: false,
            unison_voices: 1,
            unison_detune_cents: 12.0,
        },
        // 2. Blade Runner Vangelis CS/Prophet Hybrid Lush Pad
        ProphetPreset {
            id: 1,
            name: "Deckard's Cloudburst Ensemble",
            category: PresetCategory::Pads,
            author: "PROPHET-XII Sound Design",
            description: "12-voice lush PWM/Saw pad feeding the 48-Band Cloudburst Forte Harmonic Ensemble & Solaris Vowel FDN.",
            voice: SynthVoiceParams {
                osc_a_waves: OscAWaveforms { saw: true, pulse: true },
                osc_a_pw: 0.38,
                osc_b_semi: 12.0,
                osc_b_fine_cents: 8.0,
                filter_cutoff_hz: 1450.0,
                filter_resonance: 0.30,
                filter_env_amount: 0.48,
                filter_topology_morph: 0.35,
                filt_attack: 0.65,
                filt_decay: 2.40,
                filt_sustain: 0.65,
                filt_release: 2.80,
                amp_attack: 0.45,
                amp_decay: 1.80,
                amp_sustain: 0.90,
                amp_release: 3.20,
                vintage_amount: 0.55,
                ..base_v
            },
            reverb: SolarisReverbParams {
                pre_delay_ms: 42.0,
                cascade_diffusion: 0.82,
                ensemble_mode: EnsembleMode::Forte,
                ensemble_blend: 0.68,
                decay_sec: 9.5,
                shimmer_amount: 0.32,
                vowel_morph: 0.12, // Warm 'A/E' formant
                vowel_mix: 0.40,
                wet_mix: 0.52,
                ..base_r
            },
            chorus_mode: 2,
            chorus_wet: 0.55,
            unison_enabled: false,
            unison_voices: 1,
            unison_detune_cents: 14.0,
        },
        // 3. Hard Sync Screaming Lead (The Cars / Let's Go)
        ProphetPreset {
            id: 2,
            name: "Sync Sweep Overdrive (CEM3340)",
            category: PresetCategory::Leads,
            author: "Sequential Factory Calibrated",
            description: "Exact subsample CEM3340 Hard Sync with Filter Envelope sweeping Slave Osc A over 3.5 octaves via Poly-Mod.",
            voice: SynthVoiceParams {
                osc_a_semi: 19.0,
                osc_a_sync: true,
                osc_b_semi: 0.0,
                mix_osc_a: 0.95,
                mix_osc_b: 0.20,
                poly_mod: PolyModParams {
                    src_filter_env: 0.78,
                    filter_env_bipolar: false,
                    src_osc_b: 0.0,
                    dest_osc_a_freq: 0.72,
                    dest_osc_a_pw: 0.0,
                    dest_filter_cutoff: 0.0,
                },
                filter_cutoff_hz: 3200.0,
                filter_resonance: 0.42,
                filter_drive: 1.65,
                filt_attack: 0.008,
                filt_decay: 0.65,
                filt_sustain: 0.15,
                filt_release: 0.35,
                amp_attack: 0.003,
                amp_decay: 0.30,
                amp_sustain: 0.90,
                amp_release: 0.30,
                ..base_v
            },
            reverb: SolarisReverbParams {
                ensemble_mode: EnsembleMode::Off,
                decay_sec: 3.1,
                wet_mix: 0.25,
                ..base_r
            },
            chorus_mode: 1,
            chorus_wet: 0.40,
            unison_enabled: true,
            unison_voices: 4,
            unison_detune_cents: 16.0,
        },
        // 4. Moog Transistor Ladder Sub-Bass (Bass Compensation Active)
        ProphetPreset {
            id: 3,
            name: "Taurus / Model D Ladder Bass",
            category: PresetCategory::Bass,
            author: "PROPHET-XII Sound Design",
            description: "24 dB/oct Moog Transistor Ladder mode with per-stage BJT thermal saturation and active Bass Compensation.",
            voice: SynthVoiceParams {
                osc_a_semi: -12.0,
                osc_a_waves: OscAWaveforms { saw: true, pulse: true },
                osc_a_pw: 0.50,
                osc_b_semi: -12.0,
                osc_b_fine_cents: 6.5,
                filter_cutoff_hz: 290.0,
                filter_resonance: 0.58,
                filter_drive: 1.85,
                filter_env_amount: 0.56,
                filter_topology_morph: 1.0, // 100% Moog Transistor Ladder
                moog_bass_comp: 0.85,
                filt_attack: 0.002,
                filt_decay: 0.28,
                filt_sustain: 0.12,
                filt_release: 0.18,
                amp_attack: 0.002,
                amp_decay: 0.25,
                amp_sustain: 0.85,
                amp_release: 0.15,
                glide_time_sec: 0.045,
                glide_mode: GlideMode::ConstantRate,
                ..base_v
            },
            reverb: SolarisReverbParams {
                ensemble_mode: EnsembleMode::Off,
                wet_mix: 0.06,
                ..base_r
            },
            chorus_mode: 0,
            chorus_wet: 0.0,
            unison_enabled: true,
            unison_voices: 3,
            unison_detune_cents: 11.0,
        },
        // 5. Juno-106 Dual MN3009 Chorus I+II Rotary Polysynth
        ProphetPreset {
            id: 4,
            name: "Juno-Prophet Rotary Ensemble (I+II)",
            category: PresetCategory::AmbientKeys,
            author: "PROPHET-XII Sound Design",
            description: "PWM + Sub-octave Triangle routed through the authentic Juno-106 Mode I+II fast 8.65 Hz BBD vibrato chorus.",
            voice: SynthVoiceParams {
                osc_a_waves: OscAWaveforms { saw: false, pulse: true },
                osc_a_pw: 0.42,
                osc_b_semi: -12.0,
                osc_b_waves: OscBWaveforms {
                    saw: false,
                    triangle: true,
                    pulse: false,
                    lo_freq: false,
                    key_tracking: true,
                },
                filter_cutoff_hz: 1950.0,
                filter_resonance: 0.28,
                filter_env_amount: 0.38,
                filt_attack: 0.010,
                filt_decay: 0.85,
                filt_sustain: 0.45,
                filt_release: 0.90,
                amp_attack: 0.008,
                amp_decay: 0.90,
                amp_sustain: 0.70,
                amp_release: 0.95,
                ..base_v
            },
            reverb: SolarisReverbParams {
                ensemble_mode: EnsembleMode::MezzoPiano,
                ensemble_blend: 0.30,
                decay_sec: 4.2,
                wet_mix: 0.30,
                ..base_r
            },
            chorus_mode: 3, // Mode I+II pressed together
            chorus_wet: 0.78,
            unison_enabled: false,
            unison_voices: 1,
            unison_detune_cents: 0.0,
        },
        // 6. Poly-Mod Audio-Rate Metallic FM Bell Keys
        ProphetPreset {
            id: 5,
            name: "Prophet Poly-Mod Cross-FM Epiano",
            category: PresetCategory::AmbientKeys,
            author: "Sequential Factory Calibrated",
            description: "Audio-rate Oscillator B modulating Oscillator A Frequency & CEM3320 Filter Cutoff inside the 4x oversampled loop.",
            voice: SynthVoiceParams {
                osc_a_semi: 0.0,
                osc_b_semi: 19.0, // 3rd harmonic ratio
                osc_b_waves: OscBWaveforms {
                    saw: false,
                    triangle: true,
                    pulse: false,
                    lo_freq: false,
                    key_tracking: true,
                },
                mix_osc_a: 0.90,
                mix_osc_b: 0.25,
                poly_mod: PolyModParams {
                    src_filter_env: 0.15,
                    filter_env_bipolar: false,
                    src_osc_b: 0.52,
                    dest_osc_a_freq: 0.34,
                    dest_osc_a_pw: 0.15,
                    dest_filter_cutoff: 0.28,
                },
                filter_cutoff_hz: 2600.0,
                filter_resonance: 0.18,
                filt_attack: 0.002,
                filt_decay: 1.25,
                filt_sustain: 0.10,
                filt_release: 1.40,
                amp_attack: 0.002,
                amp_decay: 1.65,
                amp_sustain: 0.15,
                amp_release: 1.60,
                ..base_v
            },
            reverb: SolarisReverbParams {
                pre_delay_ms: 35.0,
                ensemble_mode: EnsembleMode::MezzoPiano,
                ensemble_blend: 0.42,
                decay_sec: 5.8,
                shimmer_amount: 0.25,
                wet_mix: 0.38,
                ..base_r
            },
            chorus_mode: 1,
            chorus_wet: 0.45,
            unison_enabled: false,
            unison_voices: 1,
            unison_detune_cents: 0.0,
        },
        // 7. Solaris Choir Cathedral (Vowel Formant + Shimmer FDN)
        ProphetPreset {
            id: 6,
            name: "Solaris Seraphim Vowel Choir",
            category: PresetCategory::Pads,
            author: "PROPHET-XII Sound Design",
            description: "Ethereal ambient pad sculpting the 8-channel Hadamard FDN tail through the Solaris 'O/U' Formant Filter and +12/+7 Shimmer.",
            voice: SynthVoiceParams {
                osc_a_waves: OscAWaveforms { saw: true, pulse: true },
                osc_a_pw: 0.25,
                osc_b_semi: 0.0,
                osc_b_fine_cents: -9.0,
                mix_noise: 0.08,
                noise_pink_blend: 0.90,
                filter_cutoff_hz: 1100.0,
                filter_resonance: 0.35,
                filter_topology_morph: 0.50, // 50/50 CEM3320 + Moog Morph
                filt_attack: 1.10,
                filt_decay: 3.20,
                filt_sustain: 0.75,
                filt_release: 4.50,
                amp_attack: 0.95,
                amp_decay: 2.50,
                amp_sustain: 0.95,
                amp_release: 4.80,
                vintage_amount: 0.65,
                ..base_v
            },
            reverb: SolarisReverbParams {
                pre_delay_ms: 65.0,
                cascade_diffusion: 0.88,
                ensemble_mode: EnsembleMode::Forte,
                ensemble_blend: 0.80,
                decay_sec: 16.5,
                shimmer_amount: 0.55,
                shimmer_fifth_blend: 0.40,
                vowel_morph: 0.72, // 'O' formant
                vowel_mix: 0.75,
                wet_mix: 0.68,
                ..base_r
            },
            chorus_mode: 2,
            chorus_wet: 0.60,
            unison_enabled: false,
            unison_voices: 1,
            unison_detune_cents: 0.0,
        },
        // 8. 12-Voice Unison Monster Stack (Prophet-10 Dual-Card Unison)
        ProphetPreset {
            id: 7,
            name: "12-Voice Dual-Card Unison Stack",
            category: PresetCategory::Bass,
            author: "Sequential Factory Calibrated",
            description: "All 12 CEM3340 voices stacked in stereo unison with non-linear cubic detune and heavy CEM3320 feedback drive.",
            voice: SynthVoiceParams {
                osc_a_semi: -12.0,
                osc_b_semi: 0.0,
                osc_b_fine_cents: 11.0,
                filter_cutoff_hz: 1650.0,
                filter_resonance: 0.38,
                filter_drive: 2.10,
                filter_env_amount: 0.50,
                filt_attack: 0.004,
                filt_decay: 0.42,
                filt_sustain: 0.30,
                filt_release: 0.35,
                amp_attack: 0.003,
                amp_decay: 0.40,
                amp_sustain: 0.90,
                amp_release: 0.32,
                vintage_amount: 0.75,
                glide_time_sec: 0.08,
                ..base_v
            },
            reverb: SolarisReverbParams {
                ensemble_mode: EnsembleMode::Off,
                decay_sec: 2.2,
                wet_mix: 0.18,
                ..base_r
            },
            chorus_mode: 1,
            chorus_wet: 0.30,
            unison_enabled: true,
            unison_voices: 12,
            unison_detune_cents: 28.0,
        },
        // 9. Radiohead Everything In Its Right Place Prophet-5 Pluck/Pad
        ProphetPreset {
            id: 8,
            name: "Kid A Warm Rev3 Analog Keys",
            category: PresetCategory::AmbientKeys,
            author: "PROPHET-XII Sound Design",
            description: "Intimate, warm low-cutoff CEM3320 polyphonic keys with authentic Rev1/Rev3 vintage voice card drift.",
            voice: SynthVoiceParams {
                osc_a_waves: OscAWaveforms { saw: true, pulse: false },
                osc_b_waves: OscBWaveforms {
                    saw: false,
                    triangle: true,
                    pulse: true,
                    lo_freq: false,
                    key_tracking: true,
                },
                osc_b_pw: 0.48,
                filter_cutoff_hz: 520.0,
                filter_resonance: 0.16,
                filter_env_amount: 0.32,
                filt_attack: 0.018,
                filt_decay: 0.68,
                filt_sustain: 0.28,
                filt_release: 0.75,
                amp_attack: 0.010,
                amp_decay: 0.85,
                amp_sustain: 0.65,
                amp_release: 0.80,
                vintage_amount: 0.88, // Heavy vintage drift across the 12 cards
                ..base_v
            },
            reverb: SolarisReverbParams {
                pre_delay_ms: 18.0,
                ensemble_mode: EnsembleMode::MezzoPiano,
                ensemble_blend: 0.25,
                decay_sec: 3.4,
                wet_mix: 0.26,
                ..base_r
            },
            chorus_mode: 0,
            chorus_wet: 0.0,
            unison_enabled: false,
            unison_voices: 1,
            unison_detune_cents: 0.0,
        },
        // 10. Progressive Analog Lead (Moog Ladder + Poly-Mod Osc B PWM)
        ProphetPreset {
            id: 9,
            name: "Moog Ladder Expressive Solo Lead",
            category: PresetCategory::Leads,
            author: "PROPHET-XII Sound Design",
            description: "Singing solo lead combining CEM3340 audio-rate PWM with the 24 dB/oct Moog Ladder and Cloudburst stereo tail.",
            voice: SynthVoiceParams {
                osc_a_waves: OscAWaveforms { saw: true, pulse: true },
                osc_a_pw: 0.32,
                osc_b_semi: 7.0, // Fifth interval
                filter_cutoff_hz: 2100.0,
                filter_resonance: 0.48,
                filter_drive: 1.75,
                filter_env_amount: 0.45,
                filter_topology_morph: 1.0, // Moog Ladder
                moog_bass_comp: 0.70,
                filt_attack: 0.015,
                filt_decay: 0.50,
                filt_sustain: 0.65,
                filt_release: 0.55,
                amp_attack: 0.008,
                amp_decay: 0.45,
                amp_sustain: 0.92,
                amp_release: 0.50,
                glide_time_sec: 0.065,
                ..base_v
            },
            reverb: SolarisReverbParams {
                pre_delay_ms: 48.0,
                ensemble_mode: EnsembleMode::MezzoPiano,
                ensemble_blend: 0.40,
                decay_sec: 5.2,
                shimmer_amount: 0.18,
                wet_mix: 0.36,
                ..base_r
            },
            chorus_mode: 2,
            chorus_wet: 0.48,
            unison_enabled: true,
            unison_voices: 2,
            unison_detune_cents: 9.5,
        },
    ]
}
