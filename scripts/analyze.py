#!/usr/bin/env python3
"""Analyze WAVs emitted exclusively by prophet-render and prophet-probes."""
import json,pathlib,sys
import numpy as np
from scipy.io import wavfile
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
root=pathlib.Path(sys.argv[1] if len(sys.argv)>1 else 'evidence')
metrics={'demos':{},'oscillators':{},'oversampling':{},'filters':{},'reverb':{}}
def read(path):
 sr,a=wavfile.read(path);return sr,a.astype(float)/(32768 if a.dtype==np.int16 else 1)
for f in sorted((root/'wavs').glob('*.wav')):
 sr,a=read(f);metrics['demos'][f.name]={'rate':sr,'frames':len(a),'finite':bool(np.isfinite(a).all()),'peak':float(np.max(abs(a))),'rms':float(np.sqrt(np.mean(a*a))),'clipped_samples':int(np.count_nonzero(abs(a)>=32767/32768)),'stereo_difference_rms':float(np.sqrt(np.mean((a[:,0]-a[:,1])**2)))}
fig,axs=plt.subplots(2,2,figsize=(12,8))
for sr in [48000,96000]:
 for name in ['naive','blep-a','blep-b','triangle']:
  _,a=read(root/'probes'/f'{name}-{sr}.wav');n=len(a);power=abs(np.fft.rfft(a))**2;valid=np.zeros(len(power),bool);valid[0]=True
  # Exact coherent fundamental: FFT bin 853. Include only unaliased harmonics.
  harmonics=np.arange(1,int((n/2)//853)+1)*853
  if name=='triangle':harmonics=harmonics[np.arange(1,len(harmonics)+1)%2==1]
  valid[harmonics]=True;alias=float(power[~valid].sum()/max(1e-30,power[valid].sum()))
  metrics['oscillators'][f'{name}-{sr}']={'nonharmonic_energy_db':float(10*np.log10(max(alias,1e-30)))}
  if sr==48000:axs[0,0].plot(np.fft.rfftfreq(n,1/sr),10*np.log10(power/max(power)+1e-16),label=name,alpha=.7)
 # Oversampling: nonharmonic energy outside harmonics of the coherent base tone.
 for os in [1,2,4]:
  _,a=read(root/'probes'/f'nonlinear-os{os}-{sr}.wav');# Trim startup; project sine/cos harmonics by least squares on the remaining interval.
  start=256;signal=a[start:];t=np.arange(start,len(a));frequency=1973/16384
  cols=[np.ones(len(t))]
  for h in range(1,int(.5/frequency)+1):cols.extend([np.sin(2*np.pi*h*frequency*t),np.cos(2*np.pi*h*frequency*t)])
  design=np.column_stack(cols);fit=design@np.linalg.lstsq(design,signal,rcond=None)[0];residual=signal-fit
  db=float(10*np.log10(max(1e-30,np.mean(residual**2)/np.mean(signal**2))))
  metrics['oversampling'][f'{os}x-{sr}']={'nonharmonic_energy_db':db}
 for name in ['curtis','moog']:
  for drive in [.25,2.0]:
   _,impulse=read(root/'probes'/f'impulse-{name}-{drive:g}-{sr}.wav');h=np.fft.rfft(impulse);freq=np.fft.rfftfreq(len(impulse),1/sr)
   record={'dc_db':float(20*np.log10(abs(h[0])+1e-20)),'cutoff_db':float(20*np.log10(abs(h[np.argmin(abs(freq-2000))])+1e-20)),'nyquist_db':float(20*np.log10(abs(h[-1])+1e-20))}
   _,tone=read(root/'probes'/f'thd-{name}-{drive:g}-{sr}.wav');# Use last half, exactly coherent at bin 128.5: window spectrum includes half bins. Full acquisition has negligible startup.
   n=len(tone);power=abs(np.fft.rfft(tone))**2;fund=257
   harmonics=np.arange(2,min(20,int(n/2/fund)))*fund
   record['thd_db']=float(10*np.log10(max(1e-30,power[harmonics].sum()/power[fund])))
   _,imd=read(root/'probes'/f'imd-{name}-{drive:g}-{sr}.wav');ip=abs(np.fft.rfft(imd))**2;bins=[abs(2*257-997),2*997-257,257+997,997-257];record['selected_imd_db']=float(10*np.log10(max(1e-30,ip[bins].sum()/(ip[257]+ip[997]))))
   metrics['filters'][f'{name}-{drive}-{sr}']=record
   if sr==48000 and drive==.25:
    axs[0,1].semilogx(freq[1:],20*np.log10(abs(h[1:])+1e-12),label=name);axs[1,0].semilogx(freq[1:],np.unwrap(np.angle(h[1:])),label=name)
 _,tail=read(root/'probes'/f'reverb-impulse-{sr}.wav');energy=np.cumsum((tail*tail)[::-1])[::-1];edc=10*np.log10(np.maximum(1e-30,energy/max(energy[0],1e-30)));t=np.arange(len(tail))/sr;region=(edc<-5)&(edc>-25)
 slope,offset=np.polyfit(t[region],edc[region],1);rt60=-60/slope
 metrics['reverb'][str(sr)]={'requested_rt60':1.5,'measured_t20_extrapolated_rt60':float(rt60),'initial_energy':float(energy[0]),'finite':bool(np.isfinite(tail).all())}
 axs[1,1].plot(t,edc,label=f'{sr} Hz')
axs[0,0].set(xlabel='Frequency Hz',ylabel='Relative dB',title='Oscillator spectrum · coherent note');axs[0,0].set_ylim(-130,5)
axs[0,1].set(xlabel='Frequency Hz',ylabel='Magnitude dB',title='Rust VCF small-signal impulse');axs[0,1].set_ylim(-100,10)
axs[1,0].set(xlabel='Frequency Hz',ylabel='Phase radians',title='Rust VCF phase')
axs[1,1].set(xlabel='Time s',ylabel='Integrated decay dB',title='8-channel FDN decay');axs[1,1].set_ylim(-100,0)
for ax in axs.flat:ax.grid(True,alpha=.2);ax.legend()
fig.tight_layout();fig.savefig(root/'measurements.png',dpi=150);plt.close(fig)
(root/'measurements.json').write_text(json.dumps(metrics,indent=2))
print(json.dumps({k:v for k,v in metrics.items() if k!='demos'},indent=2))
assert all(v['finite'] and v['rms']>1e-5 for v in metrics['demos'].values()),'invalid/silent demo'
for sr in [48000,96000]:
 for name in ['blep-a','blep-b']:assert metrics['oscillators'][f'{name}-{sr}']['nonharmonic_energy_db']<metrics['oscillators'][f'naive-{sr}']['nonharmonic_energy_db']-6,'BLEP rejection regression'
 assert metrics['oversampling'][f'4x-{sr}']['nonharmonic_energy_db']<metrics['oversampling'][f'1x-{sr}']['nonharmonic_energy_db']-15,'oversampling regression'
print('PASS: WAV integrity, BLEP improvement >6 dB, 4x nonlinear alias rejection improvement >15 dB')
