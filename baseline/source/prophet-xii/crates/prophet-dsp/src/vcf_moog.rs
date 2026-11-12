// SPDX-License-Identifier: GPL-3.0-or-later
// PROPHET-XII DSP Kernel — Moog Transistor Ladder 24 dB/oct ZDF Filter & Dual VCF Stage
// Mathematical grounding:
// - Antti Huovilainen & Vesa Välimäki non-linear BJT differential pair model
// - Surge XT sst-filters VintageLadder (https://github.com/surge-synthesizer/sst-filters)
// - Vadim Zavalishin TPT + Mystran local pivot linearization + 2-step Newton-Raphson

use core::f32::consts::PI;
use crate::vcf_cem3320::{Cem3320Filter, fast_sech2, fast_tanh};

/// Moog Transistor Ladder 24 dB/oct ZDF Filter with per-stage BJT thermal voltage saturation
/// and optional Passband Bass Compensation.
#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct MoogLadderFilter {
    /// TPT integrator states s1..s4
    pub s: [f32; 4],
    /// Previous output y4 for initial Newton-Raphson seed
    pub prev_y4: f32,
    pub sample_rate: f32,
    pub cutoff_hz: f32,
    pub resonance: f32,
    /// Thermal saturation drive (models 1 / (2 * V_T) scaling)
    pub drive: f32,
    /// Bass compensation amount in [0.0, 1.0] (restores 6..12 dB passband loss at high Q)
    pub bass_compensation: f32,
    /// Deterministic per-voice cutoff tolerance multiplier
    pub cutoff_dispersion: f32,
    g: f32,
    big_g: f32,
}

impl MoogLadderFilter {
    pub fn new(sample_rate: f32, cutoff_dispersion: f32) -> Self {
        let mut flt = Self {
            s: [0.0; 4],
            prev_y4: 0.0,
            sample_rate: sample_rate.max(8000.0),
            cutoff_hz: 2000.0,
            resonance: 0.0,
            drive: 1.15,
            bass_compensation: 0.5,
            cutoff_dispersion: cutoff_dispersion.clamp(0.95, 1.05),
            g: 0.1,
            big_g: 0.09,
        };
        flt.set_params(2000.0, 0.0, 1.15, 0.5);
        flt
    }

    #[inline]
    pub fn reset(&mut self) {
        self.s = [0.0; 4];
        self.prev_y4 = 0.0;
    }

    #[inline]
    pub fn update_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.set_params(
            self.cutoff_hz,
            self.resonance,
            self.drive,
            self.bass_compensation,
        );
    }

    #[inline(always)]
    pub fn set_params(
        &mut self,
        cutoff_hz: f32,
        resonance: f32,
        drive: f32,
        bass_compensation: f32,
    ) {
        let fc = (cutoff_hz * self.cutoff_dispersion).clamp(15.0, 0.46 * self.sample_rate);
        self.cutoff_hz = fc;
        self.resonance = resonance.clamp(0.0, 1.0);
        self.drive = drive.clamp(0.25, 4.0);
        self.bass_compensation = bass_compensation.clamp(0.0, 1.0);

        let g = (PI * fc / self.sample_rate).tan();
        self.g = g;
        self.big_g = g / (1.0 + g);
    }

    /// Evaluates the 4-stage non-linear BJT transistor ladder forward path for a trial feedback `y4_est`
    /// without mutating state, returning `(y4_actual, dy4_dy4est)` for Newton-Raphson ZDF polishing.
    ///
    /// In the Huovilainen / Välimäki transistor ladder model:
    /// Each BJT differential pair saturates according to `tanh(x / (2 * V_T))`.
    #[inline(always)]
    fn evaluate_non_linear_stages(
        &self,
        input: f32,
        y4_est: f32,
        k: f32,
        passband_comp: f32,
        vt_scale: f32,
        inv_vt: f32,
    ) -> (f32, f32) {
        let g = self.big_g;

        // Input differential pair: u = tanh((drive * input * comp - k * y4_est) * vt_scale)
        let u_arg = (input * passband_comp - k * y4_est) * vt_scale;
        let u = fast_tanh(u_arg) * inv_vt;
        let du_dy4 = -k * fast_sech2(u_arg);

        // Stage 1 BJT pair
        let y0_lin = g * (u - self.s[0]) + self.s[0];
        let y0 = fast_tanh(y0_lin * vt_scale) * inv_vt;
        let dy0_du = g * fast_sech2(y0_lin * vt_scale);

        // Stage 2 BJT pair
        let y1_lin = g * (y0 - self.s[1]) + self.s[1];
        let y1 = fast_tanh(y1_lin * vt_scale) * inv_vt;
        let dy1_dy0 = g * fast_sech2(y1_lin * vt_scale);

        // Stage 3 BJT pair
        let y2_lin = g * (y1 - self.s[2]) + self.s[2];
        let y2 = fast_tanh(y2_lin * vt_scale) * inv_vt;
        let dy2_dy1 = g * fast_sech2(y2_lin * vt_scale);

        // Stage 4 BJT pair
        let y3_lin = g * (y2 - self.s[3]) + self.s[3];
        let y3 = fast_tanh(y3_lin * vt_scale) * inv_vt;
        let dy3_dy2 = g * fast_sech2(y3_lin * vt_scale);

        let dy3_dy4est = dy3_dy2 * dy2_dy1 * dy1_dy0 * dy0_du * du_dy4;
        (y3, dy3_dy4est)
    }

    /// Processes one sample through the 4-pole ZDF Moog Transistor Ladder filter.
    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Resonance gain k in [0.0, 4.35]
        let k = 4.35 * self.resonance;
        // Bass compensation restores passband gain (1 + k * bass_comp) lost at low frequencies
        let passband_comp = 1.0 + k * self.bass_compensation * 0.65;

        // Effective thermal scale 1 / (2 * V_T) modulated by user drive
        let vt_scale = (0.85 * self.drive).clamp(0.25, 3.5);
        let inv_vt = 1.0 / vt_scale;

        // Step 1: Mystran local pivot ZDF predictor using current integrator states
        let g1 = self.big_g;
        let g2 = g1 * g1;
        let g3 = g2 * g1;
        let g4 = g2 * g2;
        let big_s = g3 * self.s[0] + g2 * self.s[1] + g1 * self.s[2] + self.s[3];
        let mut y4_est = (g4 * input * passband_comp + big_s) / (1.0 + k * g4);

        // Step 2: 2-iteration Newton-Raphson ZDF root polishing across all 4 BJT stages
        for _ in 0..2 {
            let (y4_actual, dy4_d_est) = self.evaluate_non_linear_stages(
                input,
                y4_est,
                k,
                passband_comp,
                vt_scale,
                inv_vt,
            );
            let f_err = y4_est - y4_actual;
            let denom = (1.0 - dy4_d_est).max(0.05);
            y4_est -= f_err / denom;
        }

        // Step 3: Commit TPT trapezoidal integrator state updates with per-stage BJT saturation
        let u = fast_tanh((input * passband_comp - k * y4_est) * vt_scale) * inv_vt;

        let v0 = g1 * (u - self.s[0]);
        let y0_raw = v0 + self.s[0];
        let y0 = fast_tanh(y0_raw * vt_scale) * inv_vt;
        self.s[0] = y0 + v0;

        let v1 = g1 * (y0 - self.s[1]);
        let y1_raw = v1 + self.s[1];
        let y1 = fast_tanh(y1_raw * vt_scale) * inv_vt;
        self.s[1] = y1 + v1;

        let v2 = g1 * (y1 - self.s[2]);
        let y2_raw = v2 + self.s[2];
        let y2 = fast_tanh(y2_raw * vt_scale) * inv_vt;
        self.s[2] = y2 + v2;

        let v3 = g1 * (y2 - self.s[3]);
        let y3_raw = v3 + self.s[3];
        let y3 = fast_tanh(y3_raw * vt_scale) * inv_vt;
        self.s[3] = y3 + v3;

        self.prev_y4 = y3;
        y3
    }
}

/// Dual VCF Stage allowing continuous morphing (`morph in [0.0, 1.0]`) or discrete switching
/// between Mode 1 (Prophet-10 Rev3 Curtis CEM3320) and Mode 2 (Moog 24 dB/oct Transistor Ladder).
#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct DualVcfStage {
    pub cem3320: Cem3320Filter,
    pub moog_ladder: MoogLadderFilter,
    /// `0.0` = 100% CEM3320, `1.0` = 100% Moog Ladder, intermediate = phase-coherent equal-power morph
    pub topology_morph: f32,
}

impl DualVcfStage {
    pub fn new(sample_rate: f32, cutoff_dispersion: f32) -> Self {
        Self {
            cem3320: Cem3320Filter::new(sample_rate, cutoff_dispersion),
            moog_ladder: MoogLadderFilter::new(sample_rate, cutoff_dispersion),
            topology_morph: 0.0,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.cem3320.reset();
        self.moog_ladder.reset();
    }

    #[inline]
    pub fn update_sample_rate(&mut self, sample_rate: f32) {
        self.cem3320.update_sample_rate(sample_rate);
        self.moog_ladder.update_sample_rate(sample_rate);
    }

    #[inline(always)]
    pub fn set_params(
        &mut self,
        cutoff_hz: f32,
        resonance: f32,
        drive: f32,
        bass_compensation: f32,
        topology_morph: f32,
    ) {
        self.topology_morph = topology_morph.clamp(0.0, 1.0);
        self.cem3320.set_params(cutoff_hz, resonance, drive);
        self.moog_ladder
            .set_params(cutoff_hz, resonance, drive, bass_compensation);
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let m = self.topology_morph;
        if m <= 0.0001 {
            self.cem3320.process_sample(input)
        } else if m >= 0.9999 {
            self.moog_ladder.process_sample(input)
        } else {
            let out_cem = self.cem3320.process_sample(input);
            let out_moog = self.moog_ladder.process_sample(input);
            // Phase-coherent linear/equal-energy blend (both filters share identical TPT pre-warping)
            out_cem * (1.0 - m) + out_moog * m
        }
    }
}
