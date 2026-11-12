# PROPHET-XII — ONE-SHOT FULL IMPLEMENTATION

## MISSÃO

Você é o engenheiro principal responsável por entregar o **PROPHET-XII**, um sintetizador Virtual Analog polifônico de 12 vozes, desenvolvido em Rust, com interface React/TypeScript, suporte a desktop e arquitetura compatível com ARM64 Android.

**Esta é uma tarefa de execução integral em uma única interação.**

O usuário não estará disponível para responder perguntas, aprovar checkpoints intermediários ou conduzir sessões adicionais.

Você deve inspecionar os materiais anexados, tomar decisões técnicas fundamentadas, implementar, corrigir, compilar, testar, empacotar e entregar o máximo de funcionalidades verificáveis durante esta execução.

Não encerre sua atividade após produzir apenas um plano, uma auditoria, um README, um esqueleto de projeto ou uma interface demonstrativa.

### ARQUIVOS FORNECIDOS

1. `prophet-xii.zip` — protótipo gerado anteriormente, contendo código Rust e React, ainda não validado.
2. `prompt.md` — especificação funcional e matemática original do instrumento.

Leia integralmente os arquivos antes de iniciar.

O `prompt.md` é a fonte de requisitos. O ZIP é uma implementação preliminar a ser auditada, corrigida e aproveitada quando tecnicamente justificável.

Preserve uma cópia verificável do baseline e seus hashes SHA-256.

## 1. CONDIÇÕES DE EXECUÇÃO AUTÔNOMA

Você está autorizado a:

- Criar, modificar e reorganizar arquivos dentro do workspace.
- Instalar dependências de desenvolvimento necessárias, respeitando as permissões do ambiente.
- Executar comandos de compilação, testes, benchmarks e validação.
- Consultar documentação e repositórios públicos.
- Escolher bibliotecas e dependências compatíveis com os requisitos e licenças.
- Corrigir bugs matemáticos, arquiteturais e de integração.
- Substituir implementações inadequadas por alternativas fundamentadas.
- Criar aplicações, plugins, testes, exemplos e scripts de build.
- Tomar decisões técnicas sem solicitar confirmações intermediárias.

Não está autorizado a:

- Publicar repositórios ou enviar código a contas externas.
- Incorporar firmware ou código proprietário de terceiros.
- Afirmar que algo foi testado sem executar o teste.
- Apresentar mocks como recursos funcionais.
- Modificar dispositivos físicos, executar flash ou alterar partições.
- Incluir credenciais, tokens ou segredos nos artefatos entregues.

**Não peça esclarecimentos.** Resolva ambiguidades usando as informações anexadas, boas práticas de engenharia e decisões documentadas.

Não interrompa a execução em cada etapa para solicitar autorização. Trabalhe continuamente até produzir os entregáveis finais, respeitando os limites reais do ambiente.

## 2. OBJETIVO FUNCIONAL INEGOCIÁVEL

A entrega precisa conter um instrumento que gere áudio real.

A cadeia de processamento obrigatória é:

MIDI NoteOn/NoteOff → Voice Allocator → VCO A/B → Mixer → VCF → VCA → Chorus → Reverb → Saída estéreo.

O processamento deve acontecer efetivamente no código Rust.

A interface não pode gerar áudio fictício em JavaScript, simular notas sem acionar o kernel ou apresentar indicadores de CPU inventados.

O resultado mínimo aceitável é:

1. Um motor Rust funcional.
2. Um renderizador offline capaz de produzir arquivos WAV.
3. Um aplicativo standalone conectado a um backend real de áudio.
4. Controles e eventos MIDI conectados ao motor.
5. Um pacote de código e testes reproduzíveis.

Não sacrifique esses requisitos para adicionar recursos puramente visuais.

## 3. REQUISITOS COMPLETOS DE SÍNTESE

Implemente todos os requisitos especificados em `prompt.md`, com prioridade para sua execução funcional.

### Motor de vozes

- Polifonia de 12 vozes independentes.
- Alocação circular.
- Voice stealing com mitigação de clicks.
- Unison de 1 a 12 vozes.
- Detune e pan spread.
- Glide e portamento.
- Prioridade de notas.
- Dispersão analógica determinística por voz.
- Sustain, velocity, aftertouch, pitch bend e All Notes Off.

### Osciladores

- Dois VCOs por voz inspirados no CEM3340.
- Saw, pulse, triangle e PWM.
- Seleção simultânea de formas de onda conforme o prompt.
- PolyBLEP e PolyBLAMP.
- Hard Sync com correção subamostra.
- Modo Low Frequency para Oscillator B.
- Gerador de ruído branco e rosa.
- Poly-Mod em taxa de áudio.

### Filtros

- Curtis CEM3320 de quatro polos com TPT/ZDF.
- Moog Transistor Ladder não linear de quatro polos com TPT/ZDF.
- Resolução consistente das equações de realimentação.
- Saturação modelada nos pontos apropriados.
- Ressonância, key tracking, envelope modulation e bass compensation.
- Alternância entre topologias e morph quando tecnicamente viável.

### Envelopes e modulação

- Dois ADSRs exponenciais por voz.
- LFO global.
- Mod Wheel.
- Rotas de modulação do Prophet original.
- Suavização de parâmetros.
- Atualização eficiente de coeficientes.

### Oversampling

- 1x, 2x e 4x.
- Interpolação e decimação corretamente implementadas.
- Rejeição de aliasing mensurável.
- Preservação de estados entre blocos.
- Compensação e documentação da latência.

Não aceitar implementações de filtros apenas nominalmente ZDF, nem osciladores sem correção de descontinuidades.

## 4. EFEITOS

### Chorus BBD inspirado no Roland Juno-106

- Modos I, II e I+II.
- Delays estéreo modulados.
- Interpolação de qualidade.
- LFOs com características especificadas.
- Pré/de-ênfase e filtragem.
- Saturação e ruído opcional.
- Mix dry/wet.

### Reverb híbrido Solaris/Cloudburst

- Pre-delay.
- Cascade diffusion.
- FDN de oito canais.
- Mistura ortogonal validada.
- Shimmer.
- Freeze.
- Vowel morph.
- Damping.
- Modulação das linhas de atraso.
- Ensemble harmônico de 48 bandas.
- Modos Off, mp e forte.

Implemente uma interpretação DSP independente dos produtos de referência. Não alegue equivalência de circuitos ou algoritmos proprietários sem evidência experimental.

**Não substitua os efeitos solicitados por Freeverb básico, um único delay com feedback ou efeitos fictícios.**

## 5. ARQUITETURA DE SOFTWARE

Organize o projeto como workspace Rust modular.

Módulos esperados:

- `prophet-dsp`: núcleo DSP independente de plataforma.
- `prophet-params`: parâmetros e presets.
- `prophet-standalone`: host de áudio desktop.
- `prophet-plugin`: integração CLAP/VST3.
- `prophet-render`: renderização MIDI para WAV.
- `prophet-control`: protocolo de controle e telemetria.
- `prophet-android`: backend ARM64 Android.
- `ui`: React/TypeScript.
- `tests`: testes de integração e regressão.
- `benchmarks`: desempenho e processamento em tempo real.

Você pode adaptar a organização se justificar tecnicamente a mudança.

O kernel DSP não deve depender de React, interfaces gráficas, drivers, rede ou sistema operacional específico.

Use APIs versionadas e contratos explícitos entre módulos.

## 6. HARD REAL-TIME SAFETY

A thread de áudio deve seguir as restrições:

- Nenhuma alocação dinâmica durante o processamento normal.
- Nenhum mutex bloqueante.
- Nenhuma operação de rede.
- Nenhuma leitura/escrita de arquivos.
- Nenhum logging síncrono.
- Nenhuma serialização JSON no callback.
- Buffers pré-alocados.
- Comunicação lock-free ou por snapshots adequados.
- Eventos MIDI com timestamps e offsets de amostra.
- Controle de denormais, NaN e Inf.
- Dimensionamento de memória explícito.

Não declare segurança Hard Real-Time apenas por inspeção. Inclua instrumentação e testes que sustentem as garantias alcançadas.

## 7. INTERFACE GRÁFICA FUNCIONAL

Utilize React, TypeScript e tecnologias modernas de renderização.

Estética inspirada no Prophet-10 Rev3: madeira nogueira, alumínio anodizado escuro, botões e indicadores de instrumento profissional.

A interface deve oferecer:

- Oscillator A/B.
- Poly-Mod.
- Mixer.
- Filtros Curtis/Moog.
- Envelopes.
- Chorus.
- Reverb.
- 12 LEDs de diagnóstico de vozes.
- Vintage dispersion.
- Seletor de oversampling.
- Gerenciador de presets.
- Teclado virtual de cinco oitavas.
- Pitch bend e Mod Wheel.
- MIDI Learn.
- Gráficos de envelopes e filtros.
- Osciloscópio e FFT reais.
- Indicadores reais de CPU e underruns.

A UI precisa controlar o processamento Rust.

Não utilizar valores aleatórios ou curvas animadas para falsificar telemetria.

Se houver limitações de interface no ambiente, priorize o backend funcional e entregue os componentes de UI implementados, explicitando o que não foi possível executar.

## 8. DESKTOP AUDIO E PLUGINS

O instrumento deve funcionar em desktop.

Backends:

- Windows: WASAPI.
- Linux: ALSA/PipeWire/JACK, conforme suporte.
- macOS: Core Audio, caso viável no ambiente.

Não utilizar ASIO como dependência obrigatória.

Entregar:

- Standalone.
- CLAP.
- VST3.

Use NIH-plug ou outra implementação apropriada, verificando compatibilidade de licença e dependências.

Plugins devem suportar:

- MIDI.
- Automação de parâmetros.
- State save/restore.
- Presets.
- Processamento em blocos.
- Reporte de latência.

Compile os formatos possíveis no ambiente disponível. Execute testes de carregamento em hosts quando houver hosts instalados.

Não declare que um VST3 foi validado em DAW apenas porque seu binário foi compilado.

## 9. ARM64 — MOTOROLA MOTO G8 HEADLESS

A mesma implementação Rust do DSP deve ser utilizável em ARM64.

Target primário:

`aarch64-linux-android`

Dispositivo de interesse:

- Motorola Moto G8.
- Codename `rav`.
- Qualcomm Snapdragon 665 / SM6125.
- Operação headless.
- Controle remoto pela rede.
- Tela física não utilizável.

### Implementação

Entregar:

1. Build ARM64 Android do núcleo DSP.
2. Backend de áudio AAudio via Android NDK ou biblioteca compatível.
3. Aplicação Android nativa mínima ou serviço apropriado para hospedar o sintetizador.
4. Mecanismo de controle remoto a partir do navegador de outro dispositivo.
5. Interface ou API para envio de notas MIDI e parâmetros.
6. Telemetria de CPU, underruns e estado do motor.
7. Procedimentos documentados de instalação e inicialização.
8. Configurações de qualidade e consumo para ARM.

O sistema deve operar sem exigir uma interface gráfica visível no telefone durante o uso normal.

Não presumir que o dispositivo possua root, ADB habilitado ou Android configurado. Não depender de acesso direto ao Qualcomm Hexagon DSP.

Respeitar as restrições do Android para serviços de áudio, foreground services e inicialização em background.

O modo headless precisa ter um mecanismo de recuperação, parada segura e tratamento de desconexões.

### Linux ARM64

Separadamente, mantenha o kernel compilável para:

`aarch64-unknown-linux-gnu`

Prepare um backend Linux apropriado, sem presumir que Linux nativo já funcione no Moto G8.

### Validação

Diferencie explicitamente:

- Compilação ARM64.
- Execução em emulador.
- Execução em dispositivo físico.

Não declare `DEVICE_VERIFIED` sem execução efetiva no Moto G8.

## 10. TESTES E VALIDAÇÃO MATEMÁTICA

Implemente testes automatizados cobrindo:

- 12 vozes simultâneas.
- Voice stealing.
- ADSR e NoteOff.
- MIDI.
- Determinismo.
- Processamento independente da partição de blocos, dentro da tolerância aplicável.
- Rejeição de aliasing.
- Resposta em frequência e fase.
- THD/IMD.
- Estabilidade dos filtros ZDF.
- Modulação em taxa de áudio.
- Oversampling 2x/4x.
- Chorus estéreo.
- Decaimento do reverb.
- Estabilidade do FDN.
- Ausência de NaN/Inf.
- Ausência de alocações no callback.
- Consumo de CPU.
- Memória.
- Deadline do processamento.

Gere WAVs de teste para diferentes casos:

- Nota isolada.
- Acorde de 12 vozes.
- Hard Sync.
- Filtro Curtis com ressonância.
- Moog Ladder com drive.
- Chorus I/II/I+II.
- Reverb com Ensemble.
- Unison.
- Poly-Mod.

Inclua scripts de análise espectral e visualizações que permitam reproduzir e inspecionar as medições.

Os arquivos WAV devem ser gerados pelo motor Rust, não por um sintetizador externo ou por uma implementação substituta.

Sempre que possível, execute testes em pelo menos 48 kHz e 96 kHz.

## 11. LICENÇA E PROVENIÊNCIA

Objetivo de licenciamento: GNU GPLv3.

Inclua o texto completo e correto da licença adotada.

Audite licenças de todas as dependências e dos trechos de código reutilizados.

Registre referências matemáticas e implementações open source efetivamente consultadas.

Não copie algoritmos proprietários de Sequential, Roland, Strymon ou terceiros.

Se houver incompatibilidade de licença, substitua a dependência ou registre o bloqueio; não apresente conformidade jurídica fictícia.

## 12. ESTRATÉGIA INTERNA DE EXECUÇÃO

Execute autonomamente esta sequência, sem solicitar aprovação entre as etapas:

1. Auditar arquivos e preservar baseline.
2. Compilar e reproduzir defeitos existentes.
3. Corrigir o núcleo DSP.
4. Entregar renderizador MIDI-to-WAV.
5. Validar 12 vozes, osciladores, filtros e envelopes.
6. Completar oversampling e modulações.
7. Implementar e validar chorus e reverb.
8. Integrar backend desktop e MIDI.
9. Conectar a UI ao áudio real.
10. Criar plugins CLAP/VST3.
11. Preparar Android ARM64/AAudio/headless.
12. Executar testes finais e benchmarks.
13. Corrigir falhas encontradas.
14. Empacotar os entregáveis.

Essas etapas são internas. **Não pare após cada etapa para pedir autorização.**

Prefira corrigir defeitos com testes de regressão em vez de substituições amplas e desnecessárias.

Não reescreva partes corretas apenas para produzir mais código.

Se um requisito depender de uma ferramenta ausente, prossiga com as partes independentes e documente a limitação.

Prioridade em caso de restrição de tempo ou contexto:

1. Motor DSP funcional e testado.
2. Renderizador WAV e evidências.
3. Standalone com áudio real.
4. MIDI e interface real.
5. Qualidade e correção DSP.
6. Efeitos completos.
7. CLAP/VST3.
8. ARM64 Android com backend headless.
9. Otimizações e refinamentos visuais.

A prioridade não autoriza declarar requisitos não implementados como concluídos.

## 13. ARTEFATOS OBRIGATÓRIOS

Ao finalizar, entregue um pacote:

`PROPHET-XII-DELIVERY.zip`

Inclua:

- Código-fonte completo.
- Workspaces e manifestos.
- Licença.
- Scripts de compilação.
- Testes automatizados.
- Benchmarks e relatórios.
- WAVs de demonstração gerados pelo DSP.
- Presets de exemplo.
- Interface React.
- Backends e integrações implementados.
- Artefatos binários efetivamente compilados.
- Instruções para Windows, Linux e Android ARM64.
- Manifesto de dependências.
- Matriz de conformidade com o prompt original.
- Evidências e limitações.

Não inclua caches de compilação, diretórios gigantes de dependências, credenciais ou arquivos proprietários.

Organize os binários por plataforma e arquitetura.

Mantenha um relatório `DELIVERY_REPORT.md` com:

| Requirement | Implemented | Built | Tested | Evidence |
|---|---|---|---|---|

Estados permitidos:

- `PASS`
- `FAIL`
- `PARTIAL`
- `BLOCKED`
- `NOT_TESTED`

Registre comandos, versões de ferramentas, ambientes, logs e caminhos dos artefatos.

Não utilize status `PASS` sem evidência correspondente.

## 14. CRITÉRIOS DE ACEITAÇÃO

O projeto somente pode ser considerado funcional quando:

- O DSP gerar áudio real e determinístico.
- A polifonia de 12 vozes funcionar.
- O renderizador produzir WAV válido.
- A entrada MIDI controlar o motor.
- A interface controlar parâmetros reais.
- O standalone reproduzir áudio em um dispositivo de saída.
- Os principais algoritmos DSP passarem em seus testes.
- Os efeitos estiverem conectados à cadeia real.
- Não houver indicadores de performance falsificados.
- As ferramentas de build reproduzirem os resultados registrados.

CLAP, VST3 e Android devem ser avaliados separadamente.

Compilação cruzada Android não constitui validação no dispositivo.

Não classifique o instrumento como comercialmente validado sem ensaios suficientes de estabilidade, desempenho, integração e comportamento musical.

## 15. RESPOSTA FINAL — ÚNICA ENTREGA

Sua resposta final deve conter:

1. Link ou localização do pacote `PROPHET-XII-DELIVERY.zip`.
2. Localização dos executáveis compilados.
3. Instruções diretas de execução.
4. Resumo dos testes realmente executados.
5. Comparação entre os requisitos do `prompt.md` e o que foi entregue.
6. Status do standalone.
7. Status dos plugins CLAP/VST3.
8. Status de ARM64 Android.
9. Limitações comprovadas e requisitos não concluídos.

Não substitua os artefatos solicitados por uma descrição de como implementá-los.

Não peça outra interação para começar ou terminar o trabalho.

Se alguma funcionalidade permanecer incompleta, entregue mesmo assim todo o trabalho funcional e validável realizado, identificando precisamente as limitações.

**EXECUTE AGORA. INSPECIONE OS ANEXOS, IMPLEMENTE O INSTRUMENTO, COMPILE, TESTE, CORRIJA E ENTREGUE OS ARTEFATOS.**