# PROPHET-XII

> Repositório público de fontes e evidências. A [Release v1.0.0-experimental](https://github.com/AdhonisBR/prophet-xii/releases/tag/v1.0.0-experimental) contém o ZIP completo com binários, APK, WAVs e fontes das dependências. Para executar os binários abaixo, extraia primeiro esse pacote. O estado experimental e as limitações estão no [relatório de entrega](DELIVERY_REPORT.md).

Instrumento virtual de 12 vozes com motor Rust, renderizador MIDI/WAV, host desktop CPAL, interface React remota, CLAP/VST3 NIH-plug e host Android AAudio. Esta entrega é uma implementação experimental mensurada; consulte `DELIVERY_REPORT.md` para a matriz de conformidade, evidências e limites.

## Executar no Linux x86_64

A partir desta pasta:

```sh
./binaries/linux-x86_64/prophet-standalone --ui ui/dist
```

Abra http://127.0.0.1:8787. Escolha um preset e toque no teclado. O áudio é gerado no processo Rust e sai pelo dispositivo nativo. `--list-devices` enumera saídas; `--device NOME` seleciona uma. `--midi-port 0` abre a primeira entrada MIDI do sistema; o botão Web MIDI permite usar MIDI ligado ao computador que executa o navegador. No Linux, CPAL usa ALSA; o dispositivo ALSA padrão pode encaminhar para PipeWire/PulseAudio. JACK direto não foi implementado. Windows usa WASAPI e macOS CoreAudio pelo CPAL, mas esses builds não foram executados nesta entrega.

O host falha com uma mensagem se não houver saída. `--null` é exclusivamente um sumidouro de teste temporizado, sem reprodução. Use `--duration 10 --demo` para um teste com encerramento automático. Ctrl+C solicita All Notes Off e encerra o host.

## Renderizar WAV sem dispositivo

```sh
./binaries/linux-x86_64/prophet-render --demo single-note --output nota.wav
./binaries/linux-x86_64/prophet-render --midi musica.mid --output musica.wav --sample-rate 48000 --preset 0 --tail 5
./binaries/linux-x86_64/prophet-render --demo-all demos --sample-rate 96000
```

SMF tipos 0/1 com tempo métrico e alterações de andamento são suportados. SMF tipo 2 e SMPTE são rejeitados. Os eventos são aplicados no frame exato. WAV PCM16 estéreo. As demonstrações entregues em `evidence/wavs` foram geradas por este motor.

## Plugins

Copie `binaries/linux-x86_64/PROPHET-XII.clap` para o diretório CLAP do host ou o bundle `PROPHET-XII.vst3` para o diretório VST3. São instrumentos estéreo com MIDI, automação dos parâmetros, estado e reporte de latência. A edição usa os controles genéricos do host; a interface React é do standalone/Android, sem editor embutido no plugin. Nenhuma DAW comercial foi testada. Há um host CLAP de teste e resultados do validador oficial VST3 em `evidence/logs`.

## Compilar e testar

Rust stable, Node >=18, C/C++ linker, pkg-config e headers ALSA (`libasound2-dev` em Debian/Ubuntu). Recomendado 4 GB livres para dependências e builds.

```sh
./scripts/build-linux.sh
./scripts/validate.sh
```

O `Cargo.lock` fixa todas as revisões, incluindo NIH-plug. `npm ci` usa `ui/package-lock.json`. Análise Python requer numpy, scipy e matplotlib; teste da UI requer Playwright/Chromium. `tests/ui_smoke.py` espera um host de teste em 127.0.0.1:8787.

## Android ARM64

Veja `docs/ANDROID.md`. O APK e a biblioteca AAudio ARM64 estão em `binaries/android-arm64`. Compilar não demonstra execução em telefone; o Moto G8 não esteve disponível.

Licença do projeto: GNU GPL versão 3 somente. Textos, inventário e proveniência em `LICENSE`, `DEPENDENCIES.json` e `docs/PROVENANCE.md`. O ZIP original e seus hashes permanecem em `baseline/`.

## Obter os artefatos publicados

```sh
gh release download v1.0.0-experimental --repo AdhonisBR/prophet-xii --pattern 'PROPHET-XII-DELIVERY.zip*'
sha256sum -c PROPHET-XII-DELIVERY.zip.sha256
unzip PROPHET-XII-DELIVERY.zip
```

O histórico Git contém os fontes, presets, testes, documentação e relatórios. Binários, WAVs/probes e o snapshot Cargo de fontes de dependências estão no ZIP da Release para manter o clone pequeno. O manifesto do pacote original está em `evidence/DELIVERY-ARTIFACTS-SHA256SUMS`.
