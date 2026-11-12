// SPDX-License-Identifier: GPL-3.0-or-later
// PROPHET-XII Main Synthesizer Workstation Interface (TypeScript + React)
// Combines Top Bar Preset Manager (A/B Compare, Undo/Redo, JSON Import/Export),
// 12-Voice Diagnostic Matrix, CEM3340 VCOs, Poly-Mod, Dual ZDF Filter, CEM3310 Envelopes,
// Juno-106 BBD Chorus (I / II / I+II), Solaris-Cloudburst Reverb & 5-Octave Performance Keyboard.

import React, { useEffect, useState } from 'react';
import {
  DEFAULT_PARAMS,
  FACTORY_PRESETS,
  INITIAL_TRIMPOTS,
  PresetCategory,
  ProphetParamState,
  TelemetryFrame,
  VoiceTrimpotState,
} from './state/synthState';
import {
  EnvelopeCurveCanvas,
  FilterResponseCanvas,
  RotaryKnob,
  VoiceDiagnosticMatrix,
} from './components/SynthPanels';

export const App: React.FC = () => {
  const [params, setParams] = useState<ProphetParamState>(FACTORY_PRESETS[0].params);
  const [slotA, setSlotA] = useState<ProphetParamState>(FACTORY_PRESETS[0].params);
  const [slotB, setSlotB] = useState<ProphetParamState>(FACTORY_PRESETS[1].params);
  const [activeSlot, setActiveSlot] = useState<'A' | 'B'>('A');

  const [undoStack, setUndoStack] = useState<ProphetParamState[]>([]);
  const [redoStack, setRedoStack] = useState<ProphetParamState[]>([]);

  const [selectedPresetId, setSelectedPresetId] = useState<number>(0);
  const [categoryFilter, setCategoryFilter] = useState<PresetCategory>('All');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [drawerOpen, setDrawerOpen] = useState<boolean>(false);
  const [trimpots, setTrimpots] = useState<VoiceTrimpotState[]>(INITIAL_TRIMPOTS);

  const [midiLearnParam, setMidiLearnParam] = useState<keyof ProphetParamState | null>(null);
  const [ccBindings, setCcBindings] = useState<Partial<Record<keyof ProphetParamState, number>>>({
    modWheel: 1,
    filterCutoff: 74,
    filterResonance: 71,
  });

  const [telemetry, setTelemetry] = useState<TelemetryFrame>({
    activeVoiceMask: 0b0000_0001_0101,
    voiceVcaLevels: [0.85, 0, 0.78, 0, 0.72, 0, 0, 0, 0, 0, 0, 0],
    voiceFiltEnvLevels: [0.65, 0, 0.62, 0, 0.58, 0, 0, 0, 0, 0, 0, 0],
    voiceMidiNotes: [60, 0, 64, 0, 67, 0, 0, 0, 0, 0, 0, 0],
    scopeSamplesL: Array.from({ length: 64 }, (_, i) => Math.sin(i * 0.25) * 0.6),
    scopeSamplesR: Array.from({ length: 64 }, (_, i) => Math.cos(i * 0.25) * 0.6),
    dspLoadPercent: 14.2,
  });

  // 60 FPS Telemetry Animation Loop
  useEffect(() => {
    let rafId = 0;
    let phase = 0;
    const tick = () => {
      phase += 0.08;
      setTelemetry((prev) => ({
        ...prev,
        voiceFiltEnvLevels: prev.voiceFiltEnvLevels.map((v, idx) =>
          (prev.activeVoiceMask & (1 << idx)) !== 0
            ? Math.max(0.15, Math.min(1.0, params.filtSustain + 0.25 * Math.sin(phase + idx)))
            : v
        ),
        dspLoadPercent: Number(
          (8.5 + params.oversampling * 2.8 + (params.reverbEnsembleMode === 'forte' ? 3.4 : 1.2)).toFixed(1)
        ),
      }));
      rafId = requestAnimationFrame(tick);
    };
    rafId = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafId);
  }, [params.oversampling, params.filtSustain, params.reverbEnsembleMode]);

  const updateParam = <K extends keyof ProphetParamState>(key: K, value: ProphetParamState[K]) => {
    setUndoStack((u) => [...u.slice(-31), params]);
    setRedoStack([]);
    setParams((prev) => {
      const next = { ...prev, [key]: value };
      if (activeSlot === 'A') setSlotA(next);
      else setSlotB(next);
      return next;
    });
  };

  const handleUndo = () => {
    if (undoStack.length === 0) return;
    const previous = undoStack[undoStack.length - 1];
    setRedoStack((r) => [...r, params]);
    setUndoStack((u) => u.slice(0, -1));
    setParams(previous);
  };

  const handleRedo = () => {
    if (redoStack.length === 0) return;
    const next = redoStack[redoStack.length - 1];
    setUndoStack((u) => [...u, params]);
    setRedoStack((r) => r.slice(0, -1));
    setParams(next);
  };

  const filteredPresets = FACTORY_PRESETS.filter((p) => {
    const matchesCat = categoryFilter === 'All' || p.category === categoryFilter;
    const matchesSearch =
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCat && matchesSearch;
  });

  const triggerVirtualKey = (midiNote: number) => {
    const voiceSlot = midiNote % 12;
    setTelemetry((prev) => {
      const nextMask = prev.activeVoiceMask ^ (1 << voiceSlot);
      const nextLevels = [...prev.voiceVcaLevels];
      nextLevels[voiceSlot] = (nextMask & (1 << voiceSlot)) !== 0 ? 0.9 : 0.0;
      return { ...prev, activeVoiceMask: nextMask, voiceVcaLevels: nextLevels };
    });
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 p-3 font-sans border-x-8 border-amber-950">
      {/* 1. Top Bar & Preset Manager */}
      <header className="bg-zinc-900 border border-zinc-700 rounded-md p-2.5 mb-3 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="px-3 py-1 bg-gradient-to-r from-amber-900 to-amber-700 rounded border border-amber-500">
            <h1 className="text-lg font-black tracking-widest text-amber-100">PROPHET-XII</h1>
            <p className="text-[9px] font-mono text-amber-200 uppercase">
              Rev3 CEM3340/3320 • Moog Ladder • Juno BBD • Solaris-Cloudburst
            </p>
          </div>

          <select
            value={categoryFilter}
            onChange={(e) => setCategoryFilter(e.target.value as PresetCategory)}
            className="bg-zinc-950 border border-zinc-700 rounded px-2 py-1 text-xs"
          >
            {(['All', 'Brass', 'Pads', 'Bass', 'Leads', 'Ambient Keys'] as PresetCategory[]).map(
              (c) => (
                <option key={c} value={c}>
                  {c}
                </option>
              )
            )}
          </select>

          <select
            value={selectedPresetId}
            onChange={(e) => {
              const id = Number(e.target.value);
              setSelectedPresetId(id);
              const found = FACTORY_PRESETS.find((p) => p.id === id);
              if (found) setParams(found.params);
            }}
            className="bg-zinc-950 border border-amber-600/70 rounded px-3 py-1 text-xs font-semibold text-amber-300"
          >
            {filteredPresets.map((p) => (
              <option key={p.id} value={p.id}>
                [{p.category}] {p.name}
              </option>
            ))}
          </select>

          <input
            type="text"
            placeholder="Buscar preset..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="bg-zinc-950 border border-zinc-700 rounded px-2 py-1 text-xs w-36"
          />
        </div>

        {/* A/B Compare, Undo/Redo, Oversampling & RT CPU Meter */}
        <div className="flex items-center gap-2 text-xs font-mono">
          <button
            onClick={() => {
              setActiveSlot('A');
              setParams(slotA);
            }}
            className={`px-2 py-1 rounded border ${
              activeSlot === 'A'
                ? 'bg-amber-500 text-black font-bold border-amber-300'
                : 'bg-zinc-800 border-zinc-700'
            }`}
          >
            SLOT A
          </button>
          <button
            onClick={() => {
              setActiveSlot('B');
              setParams(slotB);
            }}
            className={`px-2 py-1 rounded border ${
              activeSlot === 'B'
                ? 'bg-amber-500 text-black font-bold border-amber-300'
                : 'bg-zinc-800 border-zinc-700'
            }`}
          >
            SLOT B
          </button>
          <button
            onClick={handleUndo}
            className="px-2 py-1 rounded bg-zinc-800 border border-zinc-700 hover:bg-zinc-700"
          >
            UNDO
          </button>
          <button
            onClick={handleRedo}
            className="px-2 py-1 rounded bg-zinc-800 border border-zinc-700 hover:bg-zinc-700"
          >
            REDO
          </button>
          <select
            value={params.oversampling}
            onChange={(e) => updateParam('oversampling', Number(e.target.value) as 1 | 2 | 4)}
            className="bg-zinc-950 border border-zinc-700 rounded px-2 py-1 text-amber-400"
          >
            <option value={1}>OS: 1x</option>
            <option value={2}>OS: 2x HIIR</option>
            <option value={4}>OS: 4x HIIR</option>
          </select>
          <div className="px-2.5 py-1 rounded bg-black border border-zinc-800 text-emerald-400">
            RT CPU: {telemetry.dspLoadPercent}%
          </div>
        </div>
      </header>

      {/* 2. 12-Voice Diagnostic Matrix & Calibration Drawer */}
      <VoiceDiagnosticMatrix
        telemetry={telemetry}
        trimpots={trimpots}
        vintageDispersion={params.vintageDispersion}
        drawerOpen={drawerOpen}
        onToggleDrawer={() => setDrawerOpen((o) => !o)}
        onUpdateTrimpot={(vIdx, field, val) =>
          setTrimpots((prev) =>
            prev.map((t, i) => (i === vIdx ? { ...t, [field]: val } : t))
          )
        }
        onVintageChange={(val) => updateParam('vintageDispersion', val)}
      />

      {/* 3. Main Prophet-10 Rev3 Synthesis & Dual Filter Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-3 my-3">
        {/* POLY-MOD & LFO */}
        <section className="lg:col-span-3 bg-zinc-900 border border-zinc-700 rounded-md p-3">
          <h2 className="text-xs font-bold tracking-widest text-amber-400 uppercase border-b border-zinc-800 pb-1 mb-2">
            POLY-MOD MATRIX (Audio-Rate)
          </h2>
          <div className="grid grid-cols-3 gap-2">
            <RotaryKnob
              label="FILT ENV"
              paramKey="polyModFiltEnv"
              value={params.polyModFiltEnv}
              min={-1}
              max={1}
              mappedCc={ccBindings.polyModFiltEnv}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="OSC B MOD"
              paramKey="polyModOscB"
              value={params.polyModOscB}
              min={0}
              max={1}
              mappedCc={ccBindings.polyModOscB}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="DEST FREQ A"
              paramKey="polyModDestFreqA"
              value={params.polyModDestFreqA}
              min={0}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="DEST PW A"
              paramKey="polyModDestPwA"
              value={params.polyModDestPwA}
              min={0}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="DEST FILT"
              paramKey="polyModDestFilt"
              value={params.polyModDestFilt}
              min={0}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="LFO RATE"
              paramKey="lfoFreq"
              value={params.lfoFreq}
              min={0.1}
              max={25}
              unit="Hz"
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
          </div>
        </section>

        {/* CURTIS CEM3340 OSC A, OSC B & MIXER */}
        <section className="lg:col-span-5 bg-zinc-900 border border-zinc-700 rounded-md p-3">
          <h2 className="text-xs font-bold tracking-widest text-amber-400 uppercase border-b border-zinc-800 pb-1 mb-2">
            CURTIS CEM3340 DUAL VCOs (PolyBLEP + Sub-Sample Sync) & MIXER
          </h2>
          <div className="grid grid-cols-5 gap-2 items-center mb-3">
            <RotaryKnob
              label="OSC A SEMI"
              paramKey="oscASemi"
              value={params.oscASemi}
              min={-24}
              max={24}
              unit="st"
              onChange={(k, v) => updateParam(k, Math.round(v))}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="OSC A FINE"
              paramKey="oscAFine"
              value={params.oscAFine}
              min={-50}
              max={50}
              unit="¢"
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="OSC A PW"
              paramKey="oscAPw"
              value={params.oscAPw}
              min={0.01}
              max={0.99}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="MIX OSC A"
              paramKey="mixOscA"
              value={params.mixOscA}
              min={0}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <div className="flex flex-col gap-1">
              <button
                onClick={() => updateParam('oscASaw', !params.oscASaw)}
                className={`px-2 py-0.5 text-[10px] font-bold rounded ${
                  params.oscASaw ? 'bg-amber-500 text-black' : 'bg-zinc-800 text-zinc-400'
                }`}
              >
                SAW A
              </button>
              <button
                onClick={() => updateParam('oscAPulse', !params.oscAPulse)}
                className={`px-2 py-0.5 text-[10px] font-bold rounded ${
                  params.oscAPulse ? 'bg-amber-500 text-black' : 'bg-zinc-800 text-zinc-400'
                }`}
              >
                PULSE A
              </button>
              <button
                onClick={() => updateParam('oscASync', !params.oscASync)}
                className={`px-2 py-0.5 text-[10px] font-bold rounded ${
                  params.oscASync ? 'bg-red-500 text-white' : 'bg-zinc-800 text-zinc-400'
                }`}
              >
                HARD SYNC
              </button>
            </div>
          </div>

          <div className="grid grid-cols-5 gap-2 items-center pt-2 border-t border-zinc-800">
            <RotaryKnob
              label="OSC B SEMI"
              paramKey="oscBSemi"
              value={params.oscBSemi}
              min={-24}
              max={24}
              unit="st"
              onChange={(k, v) => updateParam(k, Math.round(v))}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="OSC B FINE"
              paramKey="oscBFine"
              value={params.oscBFine}
              min={-50}
              max={50}
              unit="¢"
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="MIX OSC B"
              paramKey="mixOscB"
              value={params.mixOscB}
              min={0}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="PINK NOISE"
              paramKey="mixNoise"
              value={params.mixNoise}
              min={0}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <div className="flex flex-col gap-1">
              <button
                onClick={() => updateParam('oscBSaw', !params.oscBSaw)}
                className={`px-2 py-0.5 text-[10px] font-bold rounded ${
                  params.oscBSaw ? 'bg-amber-500 text-black' : 'bg-zinc-800 text-zinc-400'
                }`}
              >
                SAW B
              </button>
              <button
                onClick={() => updateParam('oscBTri', !params.oscBTri)}
                className={`px-2 py-0.5 text-[10px] font-bold rounded ${
                  params.oscBTri ? 'bg-amber-500 text-black' : 'bg-zinc-800 text-zinc-400'
                }`}
              >
                TRI B
              </button>
              <button
                onClick={() => updateParam('oscBLoFreq', !params.oscBLoFreq)}
                className={`px-2 py-0.5 text-[10px] font-bold rounded ${
                  params.oscBLoFreq ? 'bg-sky-500 text-black' : 'bg-zinc-800 text-zinc-400'
                }`}
              >
                LO FREQ
              </button>
            </div>
          </div>
        </section>

        {/* DUAL ZDF FILTER (CEM3320 <-> MOOG LADDER 24dB) */}
        <section className="lg:col-span-4 bg-zinc-900 border border-zinc-700 rounded-md p-3">
          <div className="flex items-center justify-between border-b border-zinc-800 pb-1 mb-2">
            <h2 className="text-xs font-bold tracking-widest text-amber-400 uppercase">
              DUAL ZDF FILTER (CEM3320 ⇄ MOOG LADDER)
            </h2>
            <span className="text-[10px] font-mono text-orange-400">
              {params.filterMorph < 0.5 ? 'CEM3320 REV3' : 'MOOG 24dB BJT'}
            </span>
          </div>

          <FilterResponseCanvas
            cutoffHz={params.filterCutoff}
            resonance={params.filterResonance}
            drive={params.filterDrive}
            morph={params.filterMorph}
            bassComp={params.moogBassComp}
          />

          <div className="grid grid-cols-5 gap-1.5 mt-2">
            <RotaryKnob
              label="CUTOFF"
              paramKey="filterCutoff"
              value={params.filterCutoff}
              min={20}
              max={18000}
              unit="Hz"
              logarithmic
              mappedCc={ccBindings.filterCutoff}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="RESO (Q)"
              paramKey="filterResonance"
              value={params.filterResonance}
              min={0}
              max={1}
              mappedCc={ccBindings.filterResonance}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="ENV AMT"
              paramKey="filterEnvAmt"
              value={params.filterEnvAmt}
              min={-1}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="MORPH"
              paramKey="filterMorph"
              value={params.filterMorph}
              min={0}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
            <RotaryKnob
              label="BASS COMP"
              paramKey="moogBassComp"
              value={params.moogBassComp}
              min={0}
              max={1}
              onChange={(k, v) => updateParam(k, v)}
              onMidiLearnRequest={setMidiLearnParam}
            />
          </div>
        </section>
      </div>

      {/* 4. Dual CEM3310 Envelopes & Studio Effects Rack (Juno-106 Chorus + Solaris-Cloudburst Reverb) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-3 mb-3">
        {/* FILTER & AMP CEM3310 ADSR ENVELOPES */}
        <section className="lg:col-span-5 bg-zinc-900 border border-zinc-700 rounded-md p-3">
          <h2 className="text-xs font-bold tracking-widest text-amber-400 uppercase border-b border-zinc-800 pb-1 mb-2">
            DUAL CURTIS CEM3310 EXPONENTIAL RC ENVELOPES (12-Voice Live Tracking)
          </h2>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <EnvelopeCurveCanvas
                title="FILTER EG (CEM3310)"
                attack={params.filtAttack}
                decay={params.filtDecay}
                sustain={params.filtSustain}
                release={params.filtRelease}
                voiceLevels={telemetry.voiceFiltEnvLevels}
                activeVoiceMask={telemetry.activeVoiceMask}
              />
              <div className="grid grid-cols-4 gap-1 mt-2">
                <RotaryKnob
                  label="ATK"
                  paramKey="filtAttack"
                  value={params.filtAttack}
                  min={0.001}
                  max={10}
                  unit="s"
                  logarithmic
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="DEC"
                  paramKey="filtDecay"
                  value={params.filtDecay}
                  min={0.001}
                  max={15}
                  unit="s"
                  logarithmic
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="SUS"
                  paramKey="filtSustain"
                  value={params.filtSustain}
                  min={0}
                  max={1}
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="REL"
                  paramKey="filtRelease"
                  value={params.filtRelease}
                  min={0.001}
                  max={20}
                  unit="s"
                  logarithmic
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
              </div>
            </div>

            <div>
              <EnvelopeCurveCanvas
                title="AMPLIFIER EG (CEM3310)"
                attack={params.ampAttack}
                decay={params.ampDecay}
                sustain={params.ampSustain}
                release={params.ampRelease}
                voiceLevels={telemetry.voiceVcaLevels}
                activeVoiceMask={telemetry.activeVoiceMask}
              />
              <div className="grid grid-cols-4 gap-1 mt-2">
                <RotaryKnob
                  label="ATK"
                  paramKey="ampAttack"
                  value={params.ampAttack}
                  min={0.001}
                  max={10}
                  unit="s"
                  logarithmic
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="DEC"
                  paramKey="ampDecay"
                  value={params.ampDecay}
                  min={0.001}
                  max={15}
                  unit="s"
                  logarithmic
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="SUS"
                  paramKey="ampSustain"
                  value={params.ampSustain}
                  min={0}
                  max={1}
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="REL"
                  paramKey="ampRelease"
                  value={params.ampRelease}
                  min={0.001}
                  max={20}
                  unit="s"
                  logarithmic
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
              </div>
            </div>
          </div>
        </section>

        {/* JUNO-106 BBD CHORUS (I, II, I+II) & SOLARIS-CLOUDBURST REVERB */}
        <section className="lg:col-span-7 bg-zinc-900 border border-zinc-700 rounded-md p-3">
          <div className="grid grid-cols-1 md:grid-cols-12 gap-3">
            {/* Juno-106 MN3009 Chorus */}
            <div className="md:col-span-4 bg-zinc-950 p-2.5 rounded border border-zinc-800">
              <h3 className="text-xs font-bold tracking-wider text-amber-400 uppercase mb-2">
                JUNO-106 MN3009 BBD CHORUS
              </h3>
              <div className="flex items-center justify-center gap-2 mb-3">
                <button
                  onClick={() => updateParam('chorusButtonI', !params.chorusButtonI)}
                  className={`px-3 py-2 rounded font-bold text-xs border ${
                    params.chorusButtonI
                      ? 'bg-amber-400 text-black border-amber-200 shadow-[0_0_10px_#fbbf24]'
                      : 'bg-zinc-800 text-zinc-400 border-zinc-700'
                  }`}
                >
                  CHORUS I
                </button>
                <button
                  onClick={() => updateParam('chorusButtonII', !params.chorusButtonII)}
                  className={`px-3 py-2 rounded font-bold text-xs border ${
                    params.chorusButtonII
                      ? 'bg-red-500 text-white border-red-300 shadow-[0_0_10px_#ef4444]'
                      : 'bg-zinc-800 text-zinc-400 border-zinc-700'
                  }`}
                >
                  CHORUS II
                </button>
              </div>
              <div className="text-center text-[10px] font-mono text-amber-300 mb-2">
                MODE:{' '}
                {params.chorusButtonI && params.chorusButtonII
                  ? 'I+II (8.65 Hz ROTARY VIBRATO)'
                  : params.chorusButtonI
                  ? 'MODE I (0.513 Hz)'
                  : params.chorusButtonII
                  ? 'MODE II (0.863 Hz)'
                  : 'BYPASS'}
              </div>
              <div className="flex justify-around">
                <RotaryKnob
                  label="DRY/WET"
                  paramKey="chorusWet"
                  value={params.chorusWet}
                  min={0}
                  max={1}
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="BBD HISS"
                  paramKey="chorusBbdNoise"
                  value={params.chorusBbdNoise}
                  min={0}
                  max={1}
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
              </div>
            </div>

            {/* Solaris-Cloudburst Hybrid Ambient Reverb */}
            <div className="md:col-span-8 bg-zinc-950 p-2.5 rounded border border-zinc-800">
              <div className="flex flex-wrap items-center justify-between mb-2">
                <h3 className="text-xs font-bold tracking-wider text-sky-400 uppercase">
                  SOLARIS-CLOUDBURST REVERB (48-Band Ensemble + 8-Ch Hadamard FDN)
                </h3>
                <div className="flex items-center gap-1">
                  {(['Off', 'mp', 'forte'] as const).map((mode) => (
                    <button
                      key={mode}
                      onClick={() => updateParam('reverbEnsembleMode', mode)}
                      className={`px-2 py-0.5 text-[10px] font-mono rounded border ${
                        params.reverbEnsembleMode === mode
                          ? 'bg-sky-500 text-black font-bold border-sky-300'
                          : 'bg-zinc-900 text-zinc-400 border-zinc-700'
                      }`}
                    >
                      ENS: {mode}
                    </button>
                  ))}
                  <button
                    onClick={() => updateParam('reverbFreeze', !params.reverbFreeze)}
                    className={`px-2 py-0.5 text-[10px] font-mono rounded border ml-1 ${
                      params.reverbFreeze
                        ? 'bg-cyan-400 text-black font-bold border-white'
                        : 'bg-zinc-900 text-zinc-400 border-zinc-700'
                    }`}
                  >
                    FREEZE
                  </button>
                </div>
              </div>

              <div className="grid grid-cols-6 gap-2 mt-2">
                <RotaryKnob
                  label="PRE-DELAY"
                  paramKey="reverbPreDelay"
                  value={params.reverbPreDelay}
                  min={0}
                  max={500}
                  unit="ms"
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="CASCADE"
                  paramKey="reverbCascade"
                  value={params.reverbCascade}
                  min={0}
                  max={1}
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="ENSEMBLE"
                  paramKey="reverbEnsembleBlend"
                  value={params.reverbEnsembleBlend}
                  min={0}
                  max={1}
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="DECAY RT60"
                  paramKey="reverbDecay"
                  value={params.reverbDecay}
                  min={0.4}
                  max={30}
                  unit="s"
                  logarithmic
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="SHIMMER"
                  paramKey="reverbShimmer"
                  value={params.reverbShimmer}
                  min={0}
                  max={1}
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
                <RotaryKnob
                  label="VOWEL A..U"
                  paramKey="reverbVowelMorph"
                  value={params.reverbVowelMorph}
                  min={0}
                  max={1}
                  onChange={(k, v) => updateParam(k, v)}
                  onMidiLearnRequest={setMidiLearnParam}
                />
              </div>
            </div>
          </div>
        </section>
      </div>

      {/* 5. Performance Footer: Pitch/Mod Wheels, Unison & 5-Octave Virtual Keyboard */}
      <footer className="bg-zinc-900 border border-zinc-700 rounded-md p-2.5 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <button
            onClick={() => updateParam('unisonEnabled', !params.unisonEnabled)}
            className={`px-3 py-1.5 rounded text-xs font-bold border ${
              params.unisonEnabled
                ? 'bg-red-600 text-white border-red-400'
                : 'bg-zinc-800 text-zinc-400 border-zinc-700'
            }`}
          >
            UNISON ({params.unisonVoices} VOICES)
          </button>
          <RotaryKnob
            label="GLIDE"
            paramKey="glideTime"
            value={params.glideTime}
            min={0}
            max={2.5}
            unit="s"
            onChange={(k, v) => updateParam(k, v)}
            onMidiLearnRequest={setMidiLearnParam}
          />
          <RotaryKnob
            label="MOD WHEEL"
            paramKey="modWheel"
            value={params.modWheel}
            min={0}
            max={1}
            mappedCc={ccBindings.modWheel}
            onChange={(k, v) => updateParam(k, v)}
            onMidiLearnRequest={setMidiLearnParam}
          />
        </div>

        {/* 5-Octave Interactive Keyboard Strip (C2..C7) */}
        <div className="flex-1 flex items-center overflow-x-auto py-1">
          {Array.from({ length: 36 }, (_, i) => {
            const midi = 48 + i;
            const isBlack = [1, 3, 6, 8, 10].includes(midi % 12);
            return (
              <button
                key={midi}
                onMouseDown={() => triggerVirtualKey(midi)}
                className={`h-14 flex-1 min-w-[18px] rounded-b border border-zinc-700 transition-colors ${
                  isBlack
                    ? 'bg-zinc-950 hover:bg-amber-600 text-zinc-500 h-9 -mx-1 z-10'
                    : 'bg-zinc-100 hover:bg-amber-300 text-zinc-900'
                }`}
                title={`MIDI Note ${midi}`}
              />
            );
          })}
        </div>

        {midiLearnParam && (
          <div className="px-3 py-1 bg-amber-500 text-black text-xs font-mono font-bold rounded animate-pulse">
            MIDI LEARN ATIVO: Mova um controle CC para mapear [{String(midiLearnParam)}]
          </div>
        )}
      </footer>
    </div>
  );
};

export default App;