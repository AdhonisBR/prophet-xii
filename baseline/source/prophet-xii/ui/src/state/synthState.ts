// SPDX-License-Identifier: GPL-3.0-or-later
// PROPHET-XII Lock-Free UI Parameter State, 12-Voice Trimpot Calibration & Preset Manager

export type PresetCategory = 'All' | 'Brass' | 'Pads' | 'Bass' | 'Leads' | 'Ambient Keys';

export interface VoiceTrimpotState {
  voiceIndex: number;
  vcoACents: number;
  vcoBCents: number;
  vcfCutoffPct: number;
  envRcPct: number;
}

export interface ProphetParamState {
  // Oscillator A (CEM3340)
  oscASemi: number;
  oscAFine: number;
  oscAPw: number;
  oscASaw: boolean;
  oscAPulse: boolean;
  oscASync: boolean;

  // Oscillator B (CEM3340)
  oscBSemi: number;
  oscBFine: number;
  oscBPw: number;
  oscBSaw: boolean;
  oscBTri: boolean;
  oscBPulse: boolean;
  oscBLoFreq: boolean;
  oscBKeyTrack: boolean;

  // Mixer
  mixOscA: number;
  mixOscB: number;
  mixNoise: number;
  noisePinkBlend: number;

  // Poly-Mod Matrix
  polyModFiltEnv: number;
  polyModOscB: number;
  polyModDestFreqA: number;
  polyModDestPwA: number;
  polyModDestFilt: number;

  // Dual ZDF Filter (CEM3320 <-> Moog Transistor Ladder)
  filterCutoff: number;
  filterResonance: number;
  filterDrive: number;
  filterEnvAmt: number;
  filterKeyTrack: 0 | 0.5 | 1.0;
  filterMorph: number; // 0.0 = CEM3320 (Rev3), 1.0 = Moog Ladder (24dB)
  moogBassComp: number;

  // Envelopes (Dual CEM3310)
  filtAttack: number;
  filtDecay: number;
  filtSustain: number;
  filtRelease: number;

  ampAttack: number;
  ampDecay: number;
  ampSustain: number;
  ampRelease: number;

  // Global LFO & Performance
  lfoFreq: number;
  lfoInitialAmt: number;
  modWheel: number;
  pitchBendSemis: number;
  vintageDispersion: number;
  glideTime: number;
  unisonEnabled: boolean;
  unisonVoices: number;
  unisonDetune: number;
  unisonSpread: number;
  notePriority: 'Last' | 'Low' | 'High';

  // Roland Juno-106 Dual MN3009 BBD Chorus
  chorusButtonI: boolean;
  chorusButtonII: boolean;
  chorusWet: number;
  chorusBbdNoise: number;

  // Solaris-Cloudburst Hybrid Ambient Reverb
  reverbPreDelay: number;
  reverbCascade: number;
  reverbEnsembleMode: 'Off' | 'mp' | 'forte';
  reverbEnsembleBlend: number;
  reverbDecay: number;
  reverbShimmer: number;
  reverbVowelMorph: number; // 0=A, 0.25=E, 0.5=I, 0.75=O, 1.0=U
  reverbVowelMix: number;
  reverbHighCut: number;
  reverbLowCut: number;
  reverbModDepth: number;
  reverbModSpeed: number;
  reverbFreeze: boolean;
  reverbWet: number;

  // System
  oversampling: 1 | 2 | 4;
  masterGain: number;
}

export interface PresetItem {
  id: number;
  name: string;
  category: Exclude<PresetCategory, 'All'>;
  description: string;
  params: ProphetParamState;
}

export interface TelemetryFrame {
  activeVoiceMask: number;
  voiceVcaLevels: number[];
  voiceFiltEnvLevels: number[];
  voiceMidiNotes: number[];
  scopeSamplesL: number[];
  scopeSamplesR: number[];
  dspLoadPercent: number;
}

export const INITIAL_TRIMPOTS: VoiceTrimpotState[] = Array.from({ length: 12 }, (_, idx) => {
  const s = Math.sin((idx + 1) * 12.9898) * 43758.5453;
  const frac = s - Math.floor(s);
  const bipolar = frac * 2.0 - 1.0;
  return {
    voiceIndex: idx,
    vcoACents: Number((bipolar * 2.6).toFixed(2)),
    vcoBCents: Number((-bipolar * 2.3).toFixed(2)),
    vcfCutoffPct: Number((bipolar * 1.8).toFixed(2)),
    envRcPct: Number((bipolar * 3.5).toFixed(2)),
  };
});

export const DEFAULT_PARAMS: ProphetParamState = {
  oscASemi: 0,
  oscAFine: -4.5,
  oscAPw: 0.5,
  oscASaw: true,
  oscAPulse: false,
  oscASync: false,

  oscBSemi: 0,
  oscBFine: 5.5,
  oscBPw: 0.5,
  oscBSaw: true,
  oscBTri: false,
  oscBPulse: false,
  oscBLoFreq: false,
  oscBKeyTrack: true,

  mixOscA: 0.85,
  mixOscB: 0.65,
  mixNoise: 0.0,
  noisePinkBlend: 0.75,

  polyModFiltEnv: 0.0,
  polyModOscB: 0.0,
  polyModDestFreqA: 0.0,
  polyModDestPwA: 0.0,
  polyModDestFilt: 0.0,

  filterCutoff: 1850,
  filterResonance: 0.25,
  filterDrive: 1.2,
  filterEnvAmt: 0.55,
  filterKeyTrack: 0.5,
  filterMorph: 0.0,
  moogBassComp: 0.5,

  filtAttack: 0.045,
  filtDecay: 0.55,
  filtSustain: 0.45,
  filtRelease: 0.50,

  ampAttack: 0.010,
  ampDecay: 0.50,
  ampSustain: 0.85,
  ampRelease: 0.45,

  lfoFreq: 2.4,
  lfoInitialAmt: 0.0,
  modWheel: 0.0,
  pitchBendSemis: 0.0,
  vintageDispersion: 0.38,
  glideTime: 0.0,
  unisonEnabled: false,
  unisonVoices: 4,
  unisonDetune: 16.0,
  unisonSpread: 0.8,
  notePriority: 'Last',

  chorusButtonI: true,
  chorusButtonII: false,
  chorusWet: 0.4,
  chorusBbdNoise: 0.15,

  reverbPreDelay: 28,
  reverbCascade: 0.72,
  reverbEnsembleMode: 'mp',
  reverbEnsembleBlend: 0.35,
  reverbDecay: 4.8,
  reverbShimmer: 0.22,
  reverbVowelMorph: 0.15,
  reverbVowelMix: 0.30,
  reverbHighCut: 7800,
  reverbLowCut: 110,
  reverbModDepth: 0.45,
  reverbModSpeed: 0.65,
  reverbFreeze: false,
  reverbWet: 0.32,

  oversampling: 2,
  masterGain: 0.85,
};

export const FACTORY_PRESETS: PresetItem[] = [
  {
    id: 0,
    name: '1980 San Jose Rev3 Brass',
    category: 'Brass',
    description: 'Classic CEM3340 dual sawtooth brass with CEM3320 filter swell and Rev3 board dispersion.',
    params: { ...DEFAULT_PARAMS, filterCutoff: 680, filterEnvAmt: 0.62, filtAttack: 0.065, vintageDispersion: 0.42 },
  },
  {
    id: 1,
    name: "Deckard's Cloudburst Ensemble",
    category: 'Pads',
    description: '12-voice PWM/Saw pad feeding the 48-Band Cloudburst Forte Harmonic Ensemble & Solaris Vowel FDN.',
    params: {
      ...DEFAULT_PARAMS,
      oscAPulse: true,
      oscAPw: 0.38,
      oscBSemi: 12,
      filterCutoff: 1450,
      filterMorph: 0.35,
      filtAttack: 0.65,
      filtRelease: 2.8,
      ampAttack: 0.45,
      ampRelease: 3.2,
      chorusButtonI: false,
      chorusButtonII: true,
      reverbEnsembleMode: 'forte',
      reverbEnsembleBlend: 0.68,
      reverbDecay: 9.5,
      reverbShimmer: 0.32,
      reverbWet: 0.52,
    },
  },
  {
    id: 2,
    name: 'Sync Sweep Overdrive (CEM3340)',
    category: 'Leads',
    description: 'Exact subsample CEM3340 Hard Sync with Filter Envelope sweeping Slave Osc A over 3.5 octaves.',
    params: {
      ...DEFAULT_PARAMS,
      oscASemi: 19,
      oscASync: true,
      polyModFiltEnv: 0.78,
      polyModDestFreqA: 0.72,
      filterCutoff: 3200,
      filterResonance: 0.42,
      filterDrive: 1.65,
      unisonEnabled: true,
      unisonVoices: 4,
    },
  },
  {
    id: 3,
    name: 'Taurus / Model D Ladder Bass',
    category: 'Bass',
    description: '24 dB/oct Moog Transistor Ladder mode with BJT thermal saturation and active Bass Compensation.',
    params: {
      ...DEFAULT_PARAMS,
      oscASemi: -12,
      oscAPulse: true,
      oscBSemi: -12,
      filterCutoff: 290,
      filterResonance: 0.58,
      filterDrive: 1.85,
      filterMorph: 1.0,
      moogBassComp: 0.85,
      filtAttack: 0.002,
      filtDecay: 0.28,
      filtSustain: 0.12,
      chorusButtonI: false,
      reverbWet: 0.06,
      unisonEnabled: true,
      unisonVoices: 3,
    },
  },
  {
    id: 4,
    name: 'Juno-Prophet Rotary Ensemble (I+II)',
    category: 'Ambient Keys',
    description: 'PWM + Sub-octave Triangle routed through the authentic Juno-106 Mode I+II fast 8.65 Hz BBD chorus.',
    params: {
      ...DEFAULT_PARAMS,
      oscASaw: false,
      oscAPulse: true,
      oscAPw: 0.42,
      oscBSemi: -12,
      oscBSaw: false,
      oscBTri: true,
      chorusButtonI: true,
      chorusButtonII: true,
      chorusWet: 0.78,
    },
  },
  {
    id: 5,
    name: 'Prophet Poly-Mod Cross-FM Epiano',
    category: 'Ambient Keys',
    description: 'Audio-rate Oscillator B modulating Oscillator A Frequency & CEM3320 Filter Cutoff at 4x oversampling.',
    params: {
      ...DEFAULT_PARAMS,
      oscBSemi: 19,
      oscBSaw: false,
      oscBTri: true,
      polyModOscB: 0.52,
      polyModDestFreqA: 0.34,
      polyModDestFilt: 0.28,
      filterCutoff: 2600,
      oversampling: 4,
    },
  },
  {
    id: 6,
    name: 'Solaris Seraphim Vowel Choir',
    category: 'Pads',
    description: "8-channel Hadamard FDN tail sculpted through the Solaris 'O/U' Formant Filter and +12/+7 Shimmer.",
    params: {
      ...DEFAULT_PARAMS,
      filterCutoff: 1100,
      filterMorph: 0.5,
      ampAttack: 0.95,
      ampRelease: 4.8,
      reverbEnsembleMode: 'forte',
      reverbEnsembleBlend: 0.80,
      reverbDecay: 16.5,
      reverbShimmer: 0.55,
      reverbVowelMorph: 0.72,
      reverbVowelMix: 0.75,
      reverbWet: 0.68,
    },
  },
  {
    id: 7,
    name: '12-Voice Dual-Card Unison Stack',
    category: 'Bass',
    description: 'All 12 CEM3340 voices stacked in stereo unison with non-linear cubic detune and heavy CEM3320 drive.',
    params: {
      ...DEFAULT_PARAMS,
      oscASemi: -12,
      filterCutoff: 1650,
      filterDrive: 2.1,
      vintageDispersion: 0.75,
      unisonEnabled: true,
      unisonVoices: 12,
      unisonDetune: 28.0,
    },
  },
  {
    id: 8,
    name: 'Kid A Warm Rev3 Analog Keys',
    category: 'Ambient Keys',
    description: 'Intimate low-cutoff CEM3320 polyphonic keys with authentic Rev1/Rev3 vintage voice card drift.',
    params: {
      ...DEFAULT_PARAMS,
      oscBTri: true,
      oscBPulse: true,
      filterCutoff: 520,
      filterResonance: 0.16,
      vintageDispersion: 0.88,
      chorusButtonI: false,
    },
  },
  {
    id: 9,
    name: 'Moog Ladder Expressive Solo Lead',
    category: 'Leads',
    description: 'Singing solo lead combining CEM3340 audio-rate PWM with the 24 dB/oct Moog Ladder and Cloudburst tail.',
    params: {
      ...DEFAULT_PARAMS,
      oscAPulse: true,
      oscBSemi: 7,
      filterCutoff: 2100,
      filterResonance: 0.48,
      filterMorph: 1.0,
      moogBassComp: 0.70,
      glideTime: 0.065,
      unisonEnabled: true,
      unisonVoices: 2,
    },
  },
];
