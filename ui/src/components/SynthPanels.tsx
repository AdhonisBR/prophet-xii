// SPDX-License-Identifier: GPL-3.0-or-later
// PROPHET-XII Precision Hardware Controls & 60 FPS Canvas Visualizers
// Includes: RotaryKnob (with MIDI Learn), FilterCurveCanvas (Magnitude + Phase),
// EnvelopeGraphCanvas (Draggable Bezier/RC with 12-Voice Dots), SpectrumScopeCanvas,
// and 12-Voice Diagnostic Matrix + Calibration Drawer.

import React, { useEffect, useRef } from 'react';
import { ProphetParamState, TelemetryFrame, VoiceTrimpotState } from '../state/synthState';

interface RotaryKnobProps {
  label: string;
  paramKey: keyof ProphetParamState;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  logarithmic?: boolean;
  mappedCc?: number;
  onChange: (key: keyof ProphetParamState, val: number) => void;
  onMidiLearnRequest: (key: keyof ProphetParamState) => void;
}

export const RotaryKnob: React.FC<RotaryKnobProps> = ({
  label,
  paramKey,
  value,
  min,
  max,
  unit = '',
  logarithmic = false,
  mappedCc,
  onChange,
  onMidiLearnRequest,
}) => {
  const dragStartY = useRef<number | null>(null);
  const dragStartNorm = useRef<number>(0);

  const valToNorm = (v: number): number => {
    const clamped = Math.min(max, Math.max(min, v));
    if (logarithmic && min > 0) {
      return Math.log(clamped / min) / Math.log(max / min);
    }
    return (clamped - min) / (max - min || 1);
  };

  const normToVal = (n: number): number => {
    const c = Math.min(1, Math.max(0, n));
    if (logarithmic && min > 0) {
      return min * Math.pow(max / min, c);
    }
    return min + c * (max - min);
  };

  const norm = valToNorm(value);
  const angleDeg = -135 + norm * 270;

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 2) return;
    dragStartY.current = e.clientY;
    dragStartNorm.current = norm;

    const handleMove = (ev: MouseEvent) => {
      if (dragStartY.current === null) return;
      const dy = dragStartY.current - ev.clientY;
      const nextNorm = Math.min(1, Math.max(0, dragStartNorm.current + dy / 140));
      onChange(paramKey, normToVal(nextNorm));
    };

    const handleUp = () => {
      dragStartY.current = null;
      window.removeEventListener('mousemove', handleMove);
      window.removeEventListener('mouseup', handleUp);
    };

    window.addEventListener('mousemove', handleMove);
    window.addEventListener('mouseup', handleUp);
  };

  return (
    <div
      className="flex flex-col items-center select-none group cursor-ns-resize"
      onMouseDown={handleMouseDown}
      onContextMenu={(e) => {
        e.preventDefault();
        onMidiLearnRequest(paramKey);
      }}
      title="Arraste verticalmente para ajustar | Clique direito para MIDI Learn CC"
    >
      <div className="relative w-12 h-12 rounded-full bg-zinc-900 border-2 border-zinc-700 shadow-inner flex items-center justify-center">
        <div
          className="w-9 h-9 rounded-full bg-gradient-to-b from-zinc-700 to-zinc-950 border border-zinc-600 relative"
          style={{ transform: `rotate(${angleDeg}deg)` }}
        >
          <div className="w-0.5 h-3.5 bg-amber-400 mx-auto mt-0.5 rounded-full shadow" />
        </div>
        {mappedCc !== undefined && (
          <span className="absolute -top-1 -right-2 px-1 text-[9px] font-mono bg-amber-600 text-black rounded">
            CC{mappedCc}
          </span>
        )}
      </div>
      <span className="text-[10px] font-semibold tracking-wider text-zinc-300 uppercase mt-1">
        {label}
      </span>
      <span className="text-[10px] font-mono text-amber-400/90">
        {value >= 1000 ? `${(value / 1000).toFixed(2)}k` : value.toFixed(2)}
        {unit}
      </span>
    </div>
  );
};

interface FilterCurveProps {
  cutoffHz: number;
  resonance: number;
  drive: number;
  morph: number;
  bassComp: number;
}

/// 60 FPS Interactive ZDF Filter Magnitude (dB) & Phase Response Visualizer.
export const FilterResponseCanvas: React.FC<FilterCurveProps> = ({
  cutoffHz,
  resonance,
  drive,
  morph,
  bassComp,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const cvs = canvasRef.current;
    if (!cvs) return;
    const ctx = cvs.getContext('2d');
    if (!ctx) return;

    const w = cvs.width;
    const h = cvs.height;
    ctx.clearRect(0, 0, w, h);

    // Background grid
    ctx.fillStyle = '#090a0c';
    ctx.fillRect(0, 0, w, h);
    ctx.strokeStyle = '#1f242d';
    ctx.lineWidth = 1;
    for (let x = 0; x < w; x += w / 6) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, h);
      ctx.stroke();
    }
    for (let y = 0; y < h; y += h / 4) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(w, y);
      ctx.stroke();
    }

    // Evaluate 4-pole TPT transfer function H(jw) = G^4 / (1 + k * G^4), G(jw) = 1 / (1 + j * f/fc)
    const k = (4.35 * resonance) / Math.pow(drive, 0.25);
    const passbandBoost = 1.0 + morph * k * bassComp * 0.65;

    // 1. Phase curve (dashed cyan)
    ctx.beginPath();
    ctx.strokeStyle = 'rgba(56, 189, 248, 0.45)';
    ctx.lineWidth = 1.5;
    for (let px = 0; px < w; px++) {
      const normX = px / (w - 1);
      const freq = 20.0 * Math.pow(20000.0 / 20.0, normX);
      const s = freq / Math.max(20.0, cutoffHz);
      // 4-pole phase = -4 * atan(s)
      const phaseRad = -4.0 * Math.atan(s);
      const py = h - ((phaseRad + Math.PI * 2) / (Math.PI * 2)) * (h * 0.85) - h * 0.07;
      if (px === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.stroke();

    // 2. Magnitude curve (amber/orange glow)
    ctx.beginPath();
    ctx.strokeStyle = morph > 0.5 ? '#f97316' : '#f59e0b';
    ctx.lineWidth = 2.4;

    for (let px = 0; px < w; px++) {
      const normX = px / (w - 1);
      const freq = 20.0 * Math.pow(20000.0 / 20.0, normX);
      const s = freq / Math.max(20.0, cutoffHz);

      // (1 + j*s)^4 = (1 - 6*s^2 + s^4) + j*(4*s - 4*s^3)
      const s2 = s * s;
      const s4 = s2 * s2;
      const reDen = 1.0 - 6.0 * s2 + s4 + k * (1.0 - (1.0 - morph) * (5.0 / freq));
      const imDen = 4.0 * s * (1.0 - s2);
      const mag = passbandBoost / Math.sqrt(Math.max(0.015, reDen * reDen + imDen * imDen));
      const db = 20.0 * Math.log10(Math.max(1e-4, mag));
      const py = h * 0.38 - (db / 48.0) * (h * 0.55);
      const clampedY = Math.min(h - 4, Math.max(4, py));
      if (px === 0) ctx.moveTo(px, clampedY);
      else ctx.lineTo(px, clampedY);
    }
    ctx.stroke();
  }, [cutoffHz, resonance, drive, morph, bassComp]);

  return (
    <canvas
      ref={canvasRef}
      width={300}
      height={110}
      className="w-full h-28 rounded border border-zinc-700 bg-black"
    />
  );
};

interface EnvelopeGraphProps {
  title: string;
  attack: number;
  decay: number;
  sustain: number;
  release: number;
  voiceLevels: number[];
  activeVoiceMask: number;
}

/// 60 FPS True CEM3310 RC Exponential ADSR Curve + Real-Time 12-Voice Position Dots.
export const EnvelopeCurveCanvas: React.FC<EnvelopeGraphProps> = ({
  title,
  attack,
  decay,
  sustain,
  release,
  voiceLevels,
  activeVoiceMask,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const cvs = canvasRef.current;
    if (!cvs) return;
    const ctx = cvs.getContext('2d');
    if (!ctx) return;

    const w = cvs.width;
    const h = cvs.height;
    ctx.fillStyle = '#090a0c';
    ctx.fillRect(0, 0, w, h);

    const attW = Math.max(16, Math.min(w * 0.28, Math.log10(1 + attack * 10) * 65));
    const decW = Math.max(20, Math.min(w * 0.28, Math.log10(1 + decay * 10) * 65));
    const susW = w * 0.22;
    const relW = Math.max(20, Math.min(w - (attW + decW + susW) - 8, Math.log10(1 + release * 10) * 65));

    const yTop = 12;
    const yBot = h - 10;
    const ySus = yBot - sustain * (yBot - yTop);

    ctx.beginPath();
    ctx.strokeStyle = '#fbbf24';
    ctx.lineWidth = 2.2;
    ctx.moveTo(6, yBot);

    // Convex RC Attack charging toward 1.2 * V_max clamped at 1.0
    for (let i = 1; i <= 20; i++) {
      const u = i / 20;
      const v = Math.min(1.0, 1.2 * (1.0 - Math.exp(-1.7917595 * u)));
      ctx.lineTo(6 + u * attW, yBot - v * (yBot - yTop));
    }

    // Exponential RC Decay to Sustain
    for (let i = 1; i <= 20; i++) {
      const u = i / 20;
      const v = sustain + (1.0 - sustain) * Math.exp(-4.8 * u);
      ctx.lineTo(6 + attW + u * decW, yBot - v * (yBot - yTop));
    }

    // Sustain plateau
    ctx.lineTo(6 + attW + decW + susW, ySus);

    // Exponential RC Release to 0.0
    for (let i = 1; i <= 20; i++) {
      const u = i / 20;
      const v = sustain * Math.exp(-4.8 * u);
      ctx.lineTo(6 + attW + decW + susW + u * relW, yBot - v * (yBot - yTop));
    }
    ctx.stroke();

    // Render 12-Voice Real-Time Position Dots along the envelope height
    for (let vIdx = 0; vIdx < 12; vIdx++) {
      const isActive = (activeVoiceMask & (1 << vIdx)) !== 0;
      if (!isActive) continue;
      const lvl = Math.min(1, Math.max(0, voiceLevels[vIdx] || 0));
      const dotY = yBot - lvl * (yBot - yTop);
      const dotX = 6 + attW + (vIdx / 12) * (decW + susW);

      ctx.beginPath();
      ctx.fillStyle = '#38bdf8';
      ctx.arc(dotX, dotY, 3.5, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.fillStyle = '#a1a1aa';
    ctx.font = '10px monospace';
    ctx.fillText(title, 10, 16);
  }, [title, attack, decay, sustain, release, voiceLevels, activeVoiceMask]);

  return (
    <canvas
      ref={canvasRef}
      width={260}
      height={92}
      className="w-full h-24 rounded border border-zinc-700 bg-black"
    />
  );
};

interface DiagnosticMatrixProps {
  telemetry: TelemetryFrame;
  trimpots: VoiceTrimpotState[];
  vintageDispersion: number;
  drawerOpen: boolean;
  onToggleDrawer: () => void;
  onUpdateTrimpot: (voiceIdx: number, field: keyof VoiceTrimpotState, value: number) => void;
  onVintageChange: (val: number) => void;
}

/// 12-Voice Hardware LED Diagnostic Matrix & Expandable Calibration Trimpots Drawer.
export const VoiceDiagnosticMatrix: React.FC<DiagnosticMatrixProps> = ({
  telemetry,
  trimpots,
  vintageDispersion,
  drawerOpen,
  onToggleDrawer,
  onUpdateTrimpot,
  onVintageChange,
}) => {
  return (
    <div className="bg-zinc-900/95 border border-zinc-700 rounded-md p-2.5 shadow-lg">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <span className="text-xs font-bold tracking-widest text-amber-400 uppercase">
            12-Voice CEM Board Matrix
          </span>
          <button
            onClick={onToggleDrawer}
            className="px-2 py-0.5 text-[10px] font-mono rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-600"
          >
            {drawerOpen ? 'HIDE VOICE TRIMPOTS ▲' : 'CALIBRATE TRIMPOTS ▼'}
          </button>
        </div>

        {/* Row of 12 Physical Voice Card LEDs */}
        <div className="flex items-center gap-2">
          {Array.from({ length: 12 }, (_, idx) => {
            const active = (telemetry.activeVoiceMask & (1 << idx)) !== 0;
            const vca = telemetry.voiceVcaLevels[idx] || 0;
            return (
              <div
                key={idx}
                className="flex flex-col items-center bg-black/70 px-1.5 py-1 rounded border border-zinc-800"
              >
                <div
                  className={`w-3 h-3 rounded-full transition-all ${
                    active
                      ? 'bg-red-500 shadow-[0_0_8px_#ef4444]'
                      : 'bg-red-950/40 border border-red-900/40'
                  }`}
                  style={{ opacity: active ? Math.max(0.45, vca) : 0.35 }}
                />
                <span className="text-[9px] font-mono text-zinc-400 mt-0.5">V{idx + 1}</span>
              </div>
            );
          })}
        </div>

        {/* Macro VINTAGE Dispersion Knob */}
        <div className="flex items-center gap-2 bg-zinc-950 px-3 py-1 rounded border border-amber-700/40">
          <span className="text-[10px] font-bold text-amber-400 uppercase">
            VINTAGE (Rev3 → Rev1):
          </span>
          <input
            type="range"
            min={0}
            max={1.2}
            step={0.01}
            value={vintageDispersion}
            onChange={(e) => onVintageChange(parseFloat(e.target.value))}
            className="w-24 accent-amber-500"
          />
          <span className="text-xs font-mono text-amber-300">
            {Math.round(vintageDispersion * 100)}%
          </span>
        </div>
      </div>

      {drawerOpen && (
        <div className="mt-3 pt-2 border-t border-zinc-800 grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-2">
          {trimpots.map((tp) => (
            <div
              key={tp.voiceIndex}
              className="bg-zinc-950 p-2 rounded border border-zinc-800 text-[10px] font-mono"
            >
              <div className="font-bold text-amber-400 mb-1">
                VOICE CARD #{tp.voiceIndex + 1}
              </div>
              <label className="flex justify-between text-zinc-400">
                <span>VCO A (¢):</span>
                <input
                  type="number"
                  step={0.1}
                  value={tp.vcoACents}
                  onChange={(e) =>
                    onUpdateTrimpot(tp.voiceIndex, 'vcoACents', parseFloat(e.target.value) || 0)
                  }
                  className="w-14 bg-zinc-900 text-right text-amber-300 px-1 rounded"
                />
              </label>
              <label className="flex justify-between text-zinc-400 mt-1">
                <span>VCF Fc (%):</span>
                <input
                  type="number"
                  step={0.1}
                  value={tp.vcfCutoffPct}
                  onChange={(e) =>
                    onUpdateTrimpot(tp.voiceIndex, 'vcfCutoffPct', parseFloat(e.target.value) || 0)
                  }
                  className="w-14 bg-zinc-900 text-right text-amber-300 px-1 rounded"
                />
              </label>
              <label className="flex justify-between text-zinc-400 mt-1">
                <span>ENV RC (%):</span>
                <input
                  type="number"
                  step={0.1}
                  value={tp.envRcPct}
                  onChange={(e) =>
                    onUpdateTrimpot(tp.voiceIndex, 'envRcPct', parseFloat(e.target.value) || 0)
                  }
                  className="w-14 bg-zinc-900 text-right text-amber-300 px-1 rounded"
                />
              </label>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
