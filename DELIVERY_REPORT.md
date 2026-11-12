# PROPHET-XII — relatório de entrega

Entrega funcional parcial, com código, binários e evidências reproduzíveis. O instrumento gera áudio real em Rust, renderiza MIDI/WAV e recebe controles da UI React. Não foi alcançada validação comercial, garantia universal Hard Real-Time ou DEVICE_VERIFIED. A fonte original de requisitos está preservada em `docs/prompt.md`; a solicitação de execução integral e o baseline estão em `baseline/`.

## Artefatos e execução

- `binaries/linux-x86_64/prophet-standalone`: host CPAL/ALSA e servidor local. Na raiz do projeto, execute `./binaries/linux-x86_64/prophet-standalone --ui ui/dist` e abra http://127.0.0.1:8787.
- `binaries/linux-x86_64/prophet-render`: `--midi tests/fixtures/tempo.mid --output teste.wav --tail 2`, ou `--demo single-note --output nota.wav`.
- `binaries/linux-x86_64/PROPHET-XII.clap` e bundle `PROPHET-XII.vst3`: plugins reais, com controles genéricos do host.
- `binaries/android-arm64/PROPHET-XII.apk` e `libprophet_android.so`: aplicação/AAudio ARM64 compilados. APK assinado para desenvolvimento, assinatura v2/v3 verificada; chave privada excluída. Não executados em Android.
- `binaries/linux-aarch64/*.rlib`: DSP, parâmetros e controle compilados para Linux ARM64; não são executáveis.
- `evidence/wavs`: 22 demonstrações de cinco segundos em 48/96 kHz e um WAV adicional renderizado do MIDI com mudança de tempo. Nenhum foi gerado por JavaScript ou sintetizador externo.
- `presets`: dez presets JSON importáveis na interface. `ui/dist`: UI compilada; `ui/src`: fontes. Dois arquivos legados não importados estão identificados em `ui/src/LEGACY.md`.
- `scripts`: build, validação, análise espectral e empacotamento. `third_party/rust-sources.tar.gz`: snapshot compacto dos fontes de dependências Rust e seus avisos.

## Resultados efetivamente executados

`cargo build --release --locked --workspace` e `cargo test --release --locked --workspace` passaram no Linux x86_64, Rust 1.99.1. São **18 testes Rust**: três herdados, onze regressões DSP, dois testes de controle/MIDI, um de MIDI/WAV e um do callback nativo. Os logs completos estão em `evidence/logs/final-build.log` e `final-tests.log`. O wrapper NIH-plug emite aviso de compatibilidade futura de macro VST3 com este compilador; não impediu build/testes.

O teste Playwright/Chromium 151 passou para conexão React→Rust, NoteOn gerando amostras reais, alteração de cutoff, NoteOff, unison de 12 vozes, All Notes Off, desktop/mobile e ausência de erros JavaScript. A API validou comandos inválidos, snapshot de 83 parâmetros, MIDI Learn e resposta de filtro calculada no Rust. O teste usa saída null explícita; não prova reprodução física.

O host CLAP de teste carregou o binário final, criou/ativou o plugin, enumerou **83 parâmetros**, salvou/restaurou **1567 bytes de estado** e gerou áudio estéreo com NoteOn no offset 17. Reportou latência de 16 amostras em 2×. O validador Steinberg VST3 executou **47 testes, zero falhas**. Nenhuma DAW comercial, GUI embutida ou automação musical prolongada foi testada.

A saída nativa CPAL→ALSA foi exercitada em um dispositivo **ALSA null virtual**, a 48 kHz, dois canais float32, com quatro notas. O log e snapshot registram frames reais de callback. Não havia `/dev/snd`, interface física ou alto-falante acessível: reprodução audível física permanece **NOT_TESTED**. CPAL conta erros reportados pelo backend; isso não estabelece contagem completa de xruns do hardware.

## Medições e limites DSP

- Saw coerente: energia fora dos harmônicos passou de **−11,66 dB** na referência ingênua para **−27,19 dB** em ambos os VCOs: melhora de **15,53 dB**, nas duas taxas. Triangle: **−51,61 dB** nesse ensaio. Não são limites universais para todo sync/PWM/Poly-Mod.
- Não linearidade isolada: energia residual **−15,99 / −39,50 / −90,40 dB** em 1×/2×/4×, nesse sinal coerente. O teste automático exige melhora >15 dB para 4× e passou. FIR: teste de stopband e pico de latência de 16 amostras em 2× passaram; 4× reporta 24.
- Resíduos das equações implícitas Curtis/Moog abaixo de **2e−5** nos casos exercitados em 48/96 kHz. Sweeps de cutoff/ressonância/drive não produziram NaN/Inf ou saída ilimitada.
- Gráficos de frequência/fase, THD, IMD selecionada e decaimento estão em `measurements.json`/`measurements.png`. São medições digitais, sem comparação experimental com circuitos.
- O controle de decay é nominal antes de perdas do tanque. Com vowel/damping ativos, pedido de 1,5 s resultou em T20 extrapolado de **0,591 s / 0,579 s** em 48/96 kHz. A calibração broadband RT60 não está concluída.
- Todos os WAVs demonstrativos são finitos e não têm amostras clipadas. A análise e os probes são reproduzíveis em `scripts/analyze.py`/`prophet-probes`.
- Instrumentação contou **zero alocações/reallocações** no DSP e no callback do host durante os casos testados, incluindo notas, stealing, mudança de oversampling, presets e publicação de telemetria. Não cobre internals do driver, todas as rotas NIH-plug ou execução ARM física.

O benchmark usa 12 vozes, Ensemble forte, blocos de 256 e 300 medições após warmup. Em 48 kHz, Curtis 2× teve mediana **1,730 ms**, orçamento **5,333 ms**, média **34,46%**, sem violações nas 300 medições. Moog 2× teve mediana **8,461 ms** e **300/300** violações; Moog 4×, 96 kHz e algumas outras configurações também falharam no deadline. **A otimização para 12 vozes Moog de alta qualidade permanece incompleta.** A máquina cloud é compartilhada; não extrapole esses números para PCs ou Snapdragon 665. Veja `benchmarks.csv`, `benchmarks.json`, `benchmarks.png` e `memory.json`.

Delay storage principal: **1.769.472 bytes**, mais chorus, vozes, estado e filas. Kernel inline: **105.568 bytes**. RSS foi medido externamente no cenário indicado em `memory.json`. Nenhuma memória de dispositivo Android ou consumo energético foi medido.

## Matriz com o prompt original e o pedido de execução

Os campos usam PASS, FAIL, PARTIAL, BLOCKED ou NOT_TESTED. PASS exige evidência do aspecto indicado; não significa equivalência ao produto histórico.

| Requirement | Implemented | Built | Tested | Evidence |
|---|---|---|---|---|
| Baseline preservado e SHA-256 | PASS | PASS | PASS | baseline/prophet-xii.zip, SHA256SUMS, EXTRACTED-SHA256SUMS |
| Fonte original prompt.md incorporada | PASS | PASS | PASS | docs/prompt.md; baseline/prompt.md |
| Workspace modular e DSP sem UI/drivers/rede | PASS | PASS | PASS | Cargo.toml; prophet-dsp sem dependências externas |
| 12 vozes simultâneas e alocação circular | PASS | PASS | PASS | twelve_voices_steal_and_release |
| Stealing com fade de 1,5 ms | PASS | PASS | PASS | last_mono/steal state; teste de stealing |
| Unison 1–12, detune e spread | PASS | PASS | PASS | unison_one_to_twelve_priorities; teste UI de 12 vozes |
| Prioridades Last/Low/High | PASS | PASS | PARTIAL | implementação; regressão Low e fallback, High/Last sem ensaio musical |
| Glide taxa/tempo constantes | PASS | PASS | PARTIAL | voice.rs; controles reais; tempo musical não calibrado |
| Dispersão determinística VCO/VCF/RC/VCA | PASS | PASS | PASS | sementes fixas; teste de determinismo/partição |
| NoteOn/Off, velocity, sustain, bend, aftertouch | PASS | PASS | PARTIAL | controle MIDI; sustain_cc_bend_and_panic; hosts |
| MPE/multicanal completo | PARTIAL | PASS | PARTIAL | mapeamento limitado; notas iguais entre canais não independentes |
| Osc A saw/pulse e Osc B saw/triangle/pulse simultâneos | PASS | PASS | PASS | osc_cem3340; probes e WAVs |
| PWM 1–99% e B LO FREQ/key tracking | PASS | PASS | PARTIAL | controles/código; sem varredura espectral completa de PWM |
| PolyBLEP/PolyBLAMP, sem osciladores ingênuos | PASS | PASS | PASS | alias medido; erro dos pesos temporais corrigido |
| Hard sync subamostra | PASS | PASS | PARTIAL | teste de bounds e WAV sync; sem meta completa de alias em todos os ratios |
| Ruído branco/rosa | PASS | PASS | PARTIAL | gerador determinístico e mixer; espectro rosa não calibrado em todas as taxas |
| Poly-Mod em taxa de áudio | PASS | PASS | PASS | loop oversampled; preset/WAV poly-mod |
| Curtis TPT/ZDF e feedback consistente | PASS | PASS | PASS | equações/resíduo/sweep; vcf_cem3320.rs |
| Moog não linear TPT/ZDF por estágio | PASS | PASS | PASS | equações/resíduo/sweep; interpretação diferencial documentada |
| Saturação, ressonância, tracking, EG, bass compensation | PASS | PASS | PARTIAL | controles reais, THD/IMD; sem equivalência/calibração de hardware |
| Morph Curtis/Moog | PASS | PASS | PARTIAL | blend linear; estado inativo e loudness não plenamente compensados |
| Dois ADSRs exponenciais por voz | PASS | PASS | PASS | adsr_times_and_noteoff; teste em 48/96 kHz |
| LFO e Mod Wheel com destinos conectados | PASS | PASS | PASS | flags conectadas e teste CC1 |
| Smoothing e cache de coeficientes | PASS | PASS | PARTIAL | cutoff/res + 24 valores/gain; modos/parte dos efeitos discretos |
| Oversampling 1×/2×/4×, estados contínuos | PASS | PASS | PASS | FIR, DC/stopband/latência; partição independente |
| Interpolação e decimação | PASS | PASS | PASS | HalfBandStage; geração nativa em alta taxa |
| Latência reportada/compensação | PARTIAL | PASS | PASS | 0/16/24 samples reportados; sem equalização interna entre modos |
| Chorus I, II, I+II estéreo | PASS | PASS | PASS | teste de silêncio/estéreo; seis WAVs 48/96 kHz |
| Hermite, filtros, ênfase, saturação, hiss opcional | PASS | PASS | PARTIAL | código independente; sem comparação com MN3009 físico |
| Reverb predelay tempo/ms e 4 allpasses modulados | PASS | PASS | PASS | pipeline, parâmetros, WAVs e estabilidade |
| Ensemble 48 bandas, mp/forte, followers, parciais | PASS | PASS | PARTIAL | implementado/processado; afinação perceptual e anti-alias global não exaustivos |
| FDN de 8 canais e mistura ortogonal | PASS | PASS | PASS | hadamard_is_orthogonal; reverb_decay_and_freeze_bounded |
| Shimmer ±12/+7, formantes no loop, damping | PASS | PASS | PARTIAL | controles/algoritmos reais; ratios e perdas não calibrados musicalmente |
| Modulação senoidal/aleatória determinística | PASS | PASS | PASS | estados fixos, teste de determinismo e partição |
| Freeze | PASS | PASS | PARTIAL | teste de boundedness; perda zero infinita não demonstrada |
| Decay/RT60 calibrado | PARTIAL | PASS | PARTIAL | ganho nominal; 1,5 s solicitado mediu ~0,59 s com damping/vowel |
| Sem Freeverb/delay fictício | PASS | PASS | PASS | FDN, difusores e banco espectral reais |
| UI React/TS com controles reais | PASS | PASS | PASS | npm build; Playwright; /api/v1/command |
| Estética madeira/alumínio, 12 LEDs/trimpots | PASS | PASS | PASS | screenshots desktop/mobile; dados reais |
| Presets, busca/categorias, JSON, A/B, undo/redo | PASS | PASS | PARTIAL | UI implementada; teste factory/unison; fluxo completo de todas as funções não automatizado |
| Teclado de cinco oitavas, bend/mod, MIDI Learn | PASS | PASS | PASS | 61 teclas, teste de notas, API MIDI Learn/CC |
| Gráficos de envelopes editáveis | PASS | PASS | PARTIAL | SVG editável e níveis reais; arraste não coberto por teste automatizado |
| Resposta VCF medida, scope e FFT reais | PASS | PASS | PASS | worker Rust e amostras reais; teste API/UI; filtro gráfico desktop somente |
| CPU e contagem de falhas reais | PASS | PASS | PARTIAL | tempo do callback; CPAL stream errors / AAudio xrun API |
| Buffers pré-alocados e comunicação SPSC | PASS | PASS | PASS | rtrb split; testes de alocação |
| Zero alloc no callback exercitado | PASS | PASS | PASS | no_callback_allocations; native_callback_has_no_allocations... |
| Garantia universal Hard Real-Time | PARTIAL | PASS | FAIL | violações medidas em Moog/alta qualidade; driver/SO não garantidos |
| MIDI com offsets de amostra | PARTIAL | PASS | PASS | renderer/core/plugins exatos; UI/hardware desktop quantizados ao callback |
| Tratamento de NaN/Inf/denormais | PARTIAL | PASS | PARTIAL | fronteiras finitas/sweeps; FTZ global dos efeitos incompleto |
| SIMD explícito de vozes | FAIL | FAIL | NOT_TESTED | alinhamento e blocos existem; vetorização manual não implementada |
| Standalone CPAL→ALSA real | PASS | PASS | PARTIAL | dispositivo ALSA null virtual; nenhuma saída física disponível |
| Reprodução em dispositivo físico | PASS | PASS | NOT_TESTED | /dev/snd ausente; aceitação física pendente |
| WASAPI/CoreAudio | PASS | BLOCKED | NOT_TESTED | caminhos CPAL em fonte; Windows/macOS indisponíveis |
| PipeWire via ALSA / JACK direto | PARTIAL | PASS | PARTIAL | ALSA possível; PipeWire físico não testado; JACK direto ausente |
| CLAP MIDI/automation/state/latência | PASS | PASS | PASS | host C local, 83 params, state, áudio com offset |
| VST3 processamento/automação/estado | PASS | PASS | PASS | validator: 47 passed, 0 failed |
| Presets/editor React embutidos em plugin | PARTIAL | PARTIAL | NOT_TESTED | presets JSON para standalone; plugin usa editor genérico do host |
| Plugins em DAW comercial | PASS | PASS | NOT_TESTED | somente hosts de teste/validador |
| Android ARM64 DSP e AAudio/JNI | PASS | PASS | NOT_TESTED | libprophet_android.so ELF AArch64; NDK r27c/API26 |
| APK/foreground service/notificação/wakelock | PASS | PASS | NOT_TESTED | SDK35, Dex, assinatura verificada; sem runtime Android |
| Controle remoto Android, recuperação e parada | PASS | PASS | NOT_TESTED | HTTP/WS com token, três reaberturas, stop; sem aparelho/emulador |
| Instalação sem presumir root/ADB | PASS | PASS | NOT_TESTED | docs/ANDROID.md; acesso inicial autorizado ainda necessário |
| DEVICE_VERIFIED Moto G8 rav | BLOCKED | PASS | NOT_TESTED | telefone físico e ABI do firmware indisponíveis |
| Linux ARM64 kernel | PASS | PASS | NOT_TESTED | .rlib aarch64-unknown-linux-gnu; nenhum backend linkado/executado ARM |
| GPLv3 completa e inventário/proveniência | PASS | PASS | PARTIAL | LICENSE; 250 packages Rust, 145 npm entries; fontes/avisos; não certificação jurídica |
| Benchmarks CPU/memória/deadline | PASS | PASS | PASS | benchmarks.csv/json/png, memory.json; cloud x86 somente |
| Pacote sem caches/credenciais | PASS | PASS | PASS | scripts/package.py; manifesto/hash; chaves privadas fora do projeto |

## Limitações materiais restantes

O mínimo de áudio físico exigido na aceitação não pôde ser demonstrado no ambiente sem dispositivo. O standalone foi integrado ao backend e exercitado virtualmente, portanto não é declarado plenamente validado em hardware. Android foi compilado e empacotado, sem emulação ou execução no Moto G8. Windows/macOS não foram compilados. O aparelho pode exigir preparação inicial/ABI compatível; não se presume root, ADB habilitado ou capacidade de instalar com a tela inutilizável.

Moog em polifonia/oversampling altos precisa otimização. SIMD explícito, mapeamento temporal do MIDI de hardware, multicanal/MPE completo, compensação de morph e calibração RT60 permanecem incompletos. Hiss/ensemble/filters são interpretações independentes, não cópias comprovadas de circuitos/algoritmos proprietários. Plugins têm estado e automação validados em host de teste; presets/React editor não estão integrados ao editor do plugin. Não há promessa de estabilidade comercial ou uso seguro em apresentação sem ensaio no hardware/host de destino.

## Reprodução e integridade

`README.md`, `docs/DSP.md`, `docs/CONTROL.md`, `docs/ANDROID.md`, `docs/WINDOWS-MACOS.md` e `docs/PROVENANCE.md` detalham decisões, contratos e builds. `evidence/tool-versions.txt` registra ferramentas, SO, revisões dos validadores e NDK. `ARTIFACTS-SHA256SUMS` cobre os arquivos do pacote; o ZIP final tem checksum externo `.zip.sha256`. Não foram publicados repositórios, enviados códigos a contas externas ou modificados dispositivos físicos.
