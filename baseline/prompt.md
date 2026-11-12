ROLE & OBJECTIVE
Você é um Engenheiro Sênior de DSP (Processamento Digital de Sinais), Arquiteto de Sistemas de Áudio em Tempo Real (Rust) e Desenvolvedor Full-Stack de Interfaces de Áudio (TypeScript/React). Sua tarefa é projetar e implementar um sintetizador polifônico Virtual Analog de 12 vozes completo, de nível comercial/estúdio, chamado "PROPHET-XII", baseado na arquitetura eletrônica do Sequential Circuits Prophet-10 Rev3 (geração de chips Curtis CEM), expandido com um filtro Moog Transistor Ladder 24 dB/oct, o circuito BBD Stereo Chorus do Roland Juno-106 e um motor de Reverb Ambiental Híbrido inspirado no Adam Szabo Solaris e no Strymon Cloudburst.

Todo o projeto deve ser licenciado estritamente sob GNU GPLv3, permitindo portar e adaptar algoritmos matematicamente validados dos repositórios de referência listados abaixo.

1. REGRAS RÍGIDAS DE ANTI-ALUCINAÇÃO E REFERÊNCIAS OPEN SOURCE (GROUNDING)
É ESTRITAMENTE PROIBIDO usar aproximações ingênuas de DSP (como osciladores sin/saw com aliasing, filtros Biquad RBJ para o VCF, ou reverbs Freeverb/Schroeder básicos). Você DEVE fundamentar toda a matemática nas seguintes referências open source reais:

Topologia de Vozes e Dispersão Analógica (CEM3340 / CEM3320 / CEM3310):
Referência: OB-Xd (2DaT/Obxd / discoDSP, GPLv3) e Surge XT (surge-synthesizer/sst-filters, GPLv3).
Use a abordagem de modelagem de tolerância por voz (Voice Dispersion / Vintage Knob): cada uma das 12 vozes possui desvios estocásticos determinísticos (gerados por semente fixa por voz) para offsets de frequência dos VCOs (\pm 3\text{ cents}), cutoff do VCF (\pm 2\%), taxas RC dos envelopes CEM3310 (\pm 4\%) e ganho dos VCAs.
Filtros Zero-Delay Feedback (ZDF / TPT):
Referência Matemática: Livro "The Art of VA Filter Design" (Vadim Zavalishin) e repositório sst-filters (VintageLadder.h, CytomicSVF.h).
Discretização obrigatória via Transformada Bilinear Preservadora de Topologia (TPT) usando integradores trapezoidais (g = \tan(\pi f_c / f_s)) com resolução algébrica da malha de realimentação instantânea (sem atraso unitário z^{-1} no loop de feedback).
Chorus BBD (Roland Juno-106):
Referência: YKChorus (SpotlightKid/ykchorus / TAL-NoiseMaker Chorus engine, GPLv2/v3).
Modelagem fiel de linha de atraso modulada estilo Bucket Brigade Device (Panasonic MN3009 de 256 estágios).
Reverb Híbrido (Solaris + Cloudburst Ensemble):
Referência: Topologia de tanque Dattorro Plate / FDN de 8 canais (CloudSeed / Signalsmith Reverb / chowdsp_wdf).
2. ARQUITETURA DO KERNEL RT E BACKEND (RUST)
O motor de áudio em Rust deve operar como um Kernel de DSP determinístico e Hard Real-Time Safe:

Framework Base: nih-plug (suporte a VST3, CLAP e Standalone) com ponte IPC de baixa latência (nih_plug_webview / WebSockets / SharedArrayBuffer) para o frontend React, OU arquitetura modular dsp-core desacoplada da camada de I/O.
Leis da Audio Thread (Invioláveis):
Zero Alocações Dinâmicas: Proibido usar Box::new, Vec::push, String, Arc::new ou malloc dentro do callback process(). Todos os buffers de delay (BBD Chorus, Pre-Delay, FDN Reverb, Oversampling FIR) devem ser pré-alocados na inicialização (initialize() / prepare()) para suportar até f_s = 192\text{ kHz}.
Sincronização Lock-Free: Proibido usar Mutex ou RwLock bloqueantes na thread de áudio. Comunicação UI <-> Kernel exclusivamente via atomic_float (Ordering::Relaxed para parâmetros com suavização exponencial de 1 polo) e filas Single-Producer Single-Consumer (rtrb ou ringbuf) para eventos MIDI, trocas de Preset e telemetria FFT/osciloscópio.
Vetorização e Performance: Processamento em blocos fixos de 32 ou 64 amostras (MAX_BLOCK_SIZE = 64) alinhados em memória (#[repr(align(32))]) otimizados para SIMD (wide crate ou std::simd f32x4 / f32x8).
Oversampling Interno: Estágio de 2x ou 4x Polyphase Half-band FIR/IIR Oversampling por voz envolvendo os osciladores (quando em Hard Sync / FM Poly-Mod) e o estágio não-linear dos filtros VCF.
2.1 Gerenciador de 12 Vozes (Voice Allocator Kernel)
Polifonia real de 12 vozes (expansão da arquitetura dual-card do Prophet-10 Rev3).
Modos de alocação:
Poly Circular (Prophet Hardware Logic): Rotaciona entre as 12 placas de voz sequencialmente, mantendo a identidade térmica de cada voz física.
Voice Stealing Inteligente: Prioridade para vozes em estágio de Release com menor amplitude residual no VCA; suavização anti-click (crossfade de 1.5\text{ ms}) ao roubar uma voz ativa.
Unison Mode (1 a 12 vozes empilhadas): Com Unison Detune não-linear, espalhamento estéreo (Pan Spread) e prioridade de nota selecionável (Low, High, Last) com Polyphonic Glide / Portamento a taxa constante ou tempo constante.
3. ESPECIFICAÇÃO DO MOTOR DE SÍNTESE (PROPHET-10 REV3 + MOOG LADDER)
3.1 Osciladores Duplos por Voz (Curtis CEM3340 VCO Model)
Oscilador A: Formas de onda simultaneamente selecionáveis e somáveis: Sawtooth e Pulse (com PWM contínuo de 1\% a 99\%).
Oscilador B: Formas de onda simultaneamente selecionáveis e somáveis: Sawtooth, Triangle e Pulse (com PWM contínuo).
Modo LO FREQ: Converte o Osc B em um LFO por voz (sem anti-aliasing excessivo para preservar bordas afiadas de modulação), com chave de Keyboard Tracking (ON/OFF).
Anti-Aliasing: Implementação obrigatória via PolyBLEP de 2ª/4ª ordem para Saw/Pulse e PolyBLAMP para Triangle e Hard Sync.
Hard Sync Exato (Osc A sincronizado ao Osc B): Resolução analítica da fração de sub-amostra (\tau \in [0, 1)) no instante exato em que o acumulador mestre (Osc B) cruza 1.0, aplicando correção PolyBLEP proporcional à descontinuidade de amplitude no escravo (Osc A).
Gerador de Ruído: Ruído Branco / Rosa modelado via LFSR + filtro de coloração no Mixer de entrada do filtro.
3.2 Matriz Poly-Mod (Modulação Polifônica em Taxa de Áudio)
Fiel ao Prophet-5/10 Rev3, calculada amostra por amostra dentro do loop de oversampling de cada voz:

Fontes (Sources):
Filter Envelope (bipolar/unipolar ajustável)
Oscillator B (áudio-rate ou sub-áudio, permitindo síntese FM Exponencial/Linear analógica e timbres metálicos de Cross-Modulation)
Destinos (Destinations):
Oscillator A Frequency (Exponencial em V/Oct)
Oscillator A Pulse Width (PW)
Filter Cutoff Frequency
3.3 Estágio Duplo de Filtragem VCF (24 dB/oct ZDF)
O usuário pode alternar ou fazer morph entre duas topologias históricas de 4 polos:

Modo 1 — Prophet Rev3 Authentic (Curtis CEM3320 4-Pole OTA Cascade):

Cascata de 4 integradores TPT de 1 polo acoplados via ganho de realimentação global k \in [0, 4.5].
No CEM3320 real, os estágios internos possuem amplificadores operacionais de transcondutância (OTAs) com linearidade estendida, enquanto a limitação de auto-oscilação ocorre predominantemente na malha de realimentação via par diferencial/diodos.
Equação por integrador: v = (x - s) \cdot \frac{g}{1 + g}, y = v + s, s_{next} = y + v.
Não-linearidade na malha de feedback: u = x_{in} - k \cdot \tanh(\text{drive} \cdot y_4) com filtro passa-altas de acoplamento DC (5\text{ Hz}) na realimentação para modelar o bloqueio DC dos capacitores do circuito da Sequential.
Modo 2 — Moog Transistor Ladder 24 dB/oct (Huovilainen / Välimäki ZDF):

Baseado em sst-filters::VintageLadder.
Modela a saturação térmica dos pares de transistores BJT em cada um dos 4 estágios individualmente: y_i = \tanh\left(\frac{y_{i-1} - y_i}{2 V_T}\right)
Resolução instantânea ZDF com estimativa de linearização local (mystran's cheap non-linear ZDF method ou iteração de Newton-Raphson em 2 passos).
Parâmetro extra Bass Compensation: Compensa opcionalmente a perda clássica de 6\text{ dB} a 12\text{ dB} nos graves da escada de Moog quando a ressonância (Q) sobe.
3.4 Envelopes (Dual Curtis CEM3310 ADSR) & LFO Global
2x ADSR por voz (Filter EG e Amp EG): Curvas RC verdadeiramente exponenciais (ataque convexo assintótico em direção a 1.2 \times V_{max} com clamp em 1.0; Decay e Release exponenciais com constante de tempo ajustável de 1\text{ ms} a 25\text{ s}).
LFO Global + Mod Wheel: Formas Saw, Triangle e Square (combináveis simultaneamente como no hardware original), com controle de Initial Amount + Mod Wheel Depth, roteável para Osc A, Osc B, PW A+B e Filter Cutoff.
4. PIPELINE DE EFEITOS INTEGRADOS DE ESTÚDIO (CHORUS & REVERB)
4.1 Roland Juno-106 Stereo BBD Chorus (Modos I, II e I+II)
Deve reproduzir fielmente o circuito analógico estéreo baseado em dois chips Panasonic MN3009 (256 estágios BBD):

Pré/De-ênfase e Anti-Aliasing: Filtro Butterworth Passa-Baixas de 4 polos antes e depois da linha BBD com cutoff em \approx 7.2\text{ kHz} a 10\text{ kHz} (evitando aspereza digital nos agudos da cauda modulada).
Linhas de Atraso Estéreo: Duas linhas de delay interpoladas (interpolação Hermite cúbica de 4 pontos ou Allpass) com inversão de fase de 180^\circ do LFO entre o canal Esquerdo (L) e Direito (R):
Chorus I: Onda Triangular, frequência f_{LFO} = 0.513\text{ Hz}, atraso variando entre 1.66\text{ ms} e 5.35\text{ ms}.
Chorus II: Onda Triangular, frequência f_{LFO} = 0.863\text{ Hz}, atraso variando entre 1.66\text{ ms} e 5.35\text{ ms}.
Chorus I + II (Pressionados Juntos): No Juno-106 real, pressionar I e II altera a rede de resistores do integrador do LFO para gerar uma modulação mais rápida e rasa (f_{LFO} \approx 8\text{ Hz} a 9.75\text{ Hz}, profundidade reduzida de \approx 0.2\text{ ms} em torno de 3.5\text{ ms}), criando um efeito de vibrato/rotary ensemble característico.
Saturação BBD & Ruído Analógico: Saturação suave assimétrica dentro da linha de delay + gerador de ruído rosa filtrado opcional (BBD Hisss Knob, de -\infty\text{ dB} a -55\text{ dB}).
4.2 Reverb Ambiental Híbrido "SOLARIS-CLOUDBURST"
Um processador espacial algorítmico projetado para dispensar qualquer plugin externo, unindo a arquitetura do Adam Szabo Solaris e o motor harmônico do Strymon Cloudburst:

Estágio 1 — Pre-Delay & Cascade Diffuser (Inspiração Solaris):

Pre-Delay sincronizável ao tempo (0\text{ ms} a 500\text{ ms}).
Bloco Cascade: Série de 4 filtros Allpass modulados com realimentação difusa ajustável, capaz de transformar transientes secos em uma nuvem contínua antes mesmo de entrar no tanque principal.
Estágio 2 — Motor Harmônico "Ensemble" de 48 Bandas (Inspiração Strymon Cloudburst):

Chave seletora de 3 estados (OFF, mp [Mezzo-Piano], forte) + controle contínuo de Ensemble Blend.
Como implementar sem alucinar: Não é um simples pitch-shifter de oitava (shimmer comum). Divida o sinal de entrada em um banco de filtros passa-faixa de alta resolução (ou análise espectral STFT/ Constant-Q de 48 bandas distribuídas logaritmicamente de 80\text{ Hz} a 12\text{ kHz}).
Em cada banda, um seguidor de envelope (Envelope Follower com ataque suave de 15\text{ ms} e release de 120\text{ ms}) detecta a energia instantânea da fundamental tocada e excita geradores de parciais harmônicos sintetizados (2ª e 3ª harmônicas com micro-modulação de fase estilo Solina String Ensemble / chorus trifásico a 0.6\text{ Hz} e 5.8\text{ Hz}).
Resultado: Resposta dinâmica de orquestra de cordas analógica que segue polifonicamente qualquer acorde ou filtro varrido no sintetizador, injetada diretamente na entrada do tanque de reverb.
Estágio 3 — Tanque FDN de 8 Canais + Shimmer & Vowel Filter (Inspiração Solaris):

Body (Tanque Principal): Feedback Delay Network (FDN) de 8 linhas de atraso com comprimentos baseados em números primos mutuamente exclusivos e matriz de mistura ortogonal de Hadamard / Householder.
Modulação Dupla: Modulação senoidal/aleatória suave nas linhas de delay do tanque (Depth e Speed independentes).
Shimmer Pitch-Shifter no Loop: Deslocador de pitch dual (\pm 12\text{ semitons} / +7\text{ semitons}) com janela de crossfade granular suave inserido dentro da malha de realimentação do FDN.
Vowel & Damping Filters: Filtros High-Cut / Low-Cut + um filtro de formantes (Vowel Filter selecionável entre A, E, I, O, U ou morph contínuo, assinatura do Solaris) dentro do tanque para esculpir caudas vocais/etéreas, além de modo Freeze / Infinite.
5. ARQUITETURA DO FRONTEND (TYPESCRIPT + REACT)
Interface gráfica moderna de alta taxa de quadros (60\text{ FPS} via HTML5 Canvas / WebGL + React UI declarativa):

Design Visual: Estética industrial inspirada no painel de madeira nogueira (Walnut) e alumínio escovado anodizado preto do Prophet-10 Rev3, combinada com displays vetoriais modernos de alta precisão.
Componentes Obrigatórios da UI:
Top Bar & Preset Manager: Navegador de Presets categorizados (Bass, Brass, Pads, Leads, Ambient Keys), busca instantânea, importação/exportação JSON, A/B Compare, Undo/Redo, indicador de uso de CPU RT e seletor de Oversampling (1x, 2x, 4x).
12-Voice Diagnostic Matrix & Calibration Drawer: Fileira de 12 LEDs virtuais que acendem conforme cada voz física é acionada, com painel expansível (Voice Trimpots) permitindo visualizar e ajustar o desvio individual de tuning/cutoff de cada uma das 12 placas ou girar o macro knob VINTAGE (que escala a dispersão do Rev3 estável até o Rev1 instável).
Seções de Painel Interativas:
POLY-MOD (Filter Env, Osc B -> Freq A, PW A, Filt).
OSC A & OSC B (Knobs de frequência em semitons, Fine Tune, Pulse Width, botões iluminados de Waveform e Hard Sync).
MIXER (Osc A, Osc B, White/Pink Noise).
DUAL FILTER (VCF): Chave seletora/Morph entre CEM3320 (Rev3) e MOOG LADDER (24dB), Cutoff, Resonance, Env Amount, Key Tracking (Off, Half, Full) e gráfico dinâmico interativo de resposta em frequência/fase em tempo real.
ENVELOPES (ADSR): Duas seções com knobs + gráficos interativos Bezier/Exponenciais arrastáveis que mostram a posição em tempo real das 12 vozes sobre a curva.
EFFECTS RACK (Juno-106 Chorus & Solaris-Cloudburst Reverb):
Painel Juno Chorus com botões mecânicos iluminados amarelos/vermelhos I, II (suportando clique simultâneo I+II), Dry/Wet e BBD Noise.
Painel Reverb com controles de Pre-Delay, Cascade, Decay, Shimmer, Vowel Morph, Tone Tilt, Modulation e chave de 3 posições Cloudburst Ensemble (Off / mp / forte).
Rodapé de Performance & MIDI: Teclado virtual de 5 oitavas com suporte a Pitch Bend, Mod Wheel, Unison Config, MIDI Learn (clique direito em qualquer knob para mapear CC) e suporte a Aftertouch/Velocity.
6. ESTRUTURA DE DIRETÓRIOS E PLANO DE ENTREGA
Estruture o projeto como um workspace limpo:

prophet-xii/
├── LICENSE                      # GNU GPLv3
├── Cargo.toml                   # Workspace Rust
├── crates/
│   ├── prophet-dsp/             # Kernel DSP puro (sem dependências de UI, 100% RT-safe)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── voice.rs         # 12-Voice Allocator, Glide, Unison & Vintage Dispersion
│   │   │   ├── osc_cem3340.rs   # PolyBLEP/PolyBLAMP VCO + Sub-sample Hard Sync
│   │   │   ├── vcf_cem3320.rs   # ZDF/TPT 4-Pole Curtis Filter
│   │   │   ├── vcf_moog.rs      # ZDF/TPT 4-Pole Non-linear Transistor Ladder
│   │   │   ├── env_cem3310.rs   # True RC Exponential ADSR
│   │   │   ├── oversample.rs    # Polyphase Half-band FIR/IIR 2x/4x
│   │   │   └── fx/
│   │   │       ├── juno_chorus.rs     # MN3009 Dual BBD Stereo Chorus (I, II, I+II)
│   │   │       └── solaris_reverb.rs  # 48-Band Ensemble + Cascade + 8-Ch Hadamard FDN
│   └── prophet-plugin/          # Wrapper nih-plug (VST3/CLAP/Standalone) + IPC Bridge
└── ui/                          # Frontend TypeScript + React + Vite + Tailwind/Canvas
    ├── package.json
    └── src/
        ├── components/          # Knobs, EnvelopeGraph, FilterCurve, VoiceLEDs, FXRack
        ├── state/               # Lock-free parameter sync & Preset Manager
        └── App.tsx