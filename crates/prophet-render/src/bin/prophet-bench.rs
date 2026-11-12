// SPDX-License-Identifier: GPL-3.0-only
use prophet_control::ProphetXiiPlugin;
use prophet_params::ParamId::*;
use std::time::Instant;
fn main() {
    println!("sample_rate,oversampling,filter,ensemble,block,median_us,p99_us,max_us,budget_us,deadline_misses,realtime_percent,kernel_inline_bytes");
    for sr in [48000.0, 96000.0] {
        for factor in [1.0, 2.0, 4.0] {
            for moog in [false, true] {
                let mut p = ProphetXiiPlugin::new(sr);
                p.ipc.params.set(OversamplingMode, factor);
                p.ipc.params.set(FilterMorph, if moog { 1.0 } else { 0.0 });
                p.ipc.params.set(ReverbEnsembleMode, 2.0);
                p.sync_parameters();
                for note in 48..60 {
                    p.handle_midi_message(0x90, note, 100);
                }
                let (mut l, mut r) = ([0.0; 256], [0.0; 256]);
                let mut times = Vec::with_capacity(300);
                for i in 0..330 {
                    let start = Instant::now();
                    p.process_audio_block(&mut l, &mut r);
                    let us = start.elapsed().as_secs_f64() * 1e6;
                    if i >= 30 {
                        times.push(us);
                    }
                    std::hint::black_box(&l);
                }
                let budget = 256.0 / sr as f64 * 1e6;
                let misses = times.iter().filter(|&&v| v > budget).count();
                let percent = times.iter().sum::<f64>() / times.len() as f64 / budget * 100.0;
                times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                println!("{sr},{factor},{},forte,256,{:0.3},{:0.3},{:0.3},{budget:0.3},{misses},{percent:0.2},{}",if moog{"moog"}else{"curtis"},times[150],times[297],times[299],std::mem::size_of::<prophet_dsp::ProphetXiiKernel>());
            }
        }
    }
}
