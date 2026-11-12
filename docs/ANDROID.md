# Android ARM64 / Moto G8 (rav)

O host é uma aplicação nativa mínima, com serviço foreground `mediaPlayback`, JNI e biblioteca Rust `aarch64-linux-android`. Requer Android 8/API26 ou posterior, ARM64, acesso autorizado para instalar/iniciar a aplicação e uma rede local. Não requer root, firmware modificado ou acesso Hexagon. O sistema Android instalado precisa expor a ABI arm64-v8a: uma CPU de 64 bits sozinha não comprova isso. Com ADB já autorizado, `adb shell getprop ro.product.cpu.abilist` permite conferir; nenhum ABI do Moto G8 foi consultado nesta execução. O DSP é exatamente o crate desktop.

## Instalar e iniciar

1. Instale `binaries/android-arm64/PROPHET-XII.apk` pelo instalador Android autorizado. O APK entregue usa assinatura de desenvolvimento gerada para esta entrega, sem a chave privada no pacote.
2. Abra PROPHET-XII uma vez. Autorize notificações se solicitado. O serviço inicia a saída AAudio, mantém notificação persistente e um wakelock enquanto ativo.
3. A tela inicial mostra o endereço `http://IP_DO_TELEFONE:8787` e um token aleatório exclusivo da instalação. Abra esse endereço em outro dispositivo e informe o token no campo da interface. As notas e parâmetros chegam ao Rust; o som sai pelo telefone/interface de áudio ligada a ele. Não há streaming de áudio para o navegador.
4. O uso normal passa a ser headless. Pare pelo botão da tela inicial, ação Stop da notificação ou comando remoto autenticado `{"type":"stop"}` em `/api/v1/command`. All Notes Off interrompe notas, mantendo o serviço ligado.

Se ADB já estiver autorizado, instalação e lançamento podem ser feitos com:

```sh
adb install -r binaries/android-arm64/PROPHET-XII.apk
adb shell am start -n org.prophetxii/.MainActivity
adb shell am stopservice -n org.prophetxii/.SynthService
```

ADB não é pressuposto. Com tela inutilizável e sem ADB, acessibilidade, gerenciamento ou outro acesso Android previamente autorizado, não há caminho universal para instalar, obter o token e iniciar o serviço. Essa preparação inicial permanece dependente do estado real do Moto G8. Não foram feitos flash, alterações de partições ou acesso físico.

## Recuperação, energia e rede

Desconectar o WebSocket libera notas. Falhas AAudio provocam até três tentativas de reabertura, espaçadas em um segundo, e depois parada. Perda de audio focus encerra o serviço. Não há reinício automático no boot: o Android moderno restringe início de serviço mediaPlayback em background. O serviço é START_NOT_STICKY e exige início autorizado em foreground. Confira restrições de bateria do fabricante e temperatura no aparelho real.

Use uma rede confiável: a interface usa HTTP/WebSocket com token, sem TLS integrado. O token fica no armazenamento privado do app e não está embutido no APK. Não exponha a porta 8787 à internet. O backend Java é deliberadamente mínimo, limitado a quatro conexões e requisições de controle de 32 KiB. O gráfico de resposta medida do filtro só está disponível no host desktop; osciloscópio/FFT, vozes e CPU vêm do Rust em ambas as plataformas. Não há MIDI USB Android nativo; use Web MIDI no computador remoto ou a API MIDI.

Qualidade inicial recomendada para ensaios no Snapdragon 665: 48 kHz, oversampling 1×/2×, Curtis, Ensemble Off/mp, reduzir unison. 4× e Moog não têm garantia de deadline; veja benchmarks x86 e meça CPU/underruns no próprio telefone. Nenhum benchmark x86 constitui evidência de consumo ARM.

## Build

Instale NDK r27c, SDK platform35, build-tools35 e um JDK. Defina ANDROID_NDK_HOME e ANDROID_SDK_ROOT. `scripts/build-android.sh` constrói Rust, Java/Dex, assets e APK. PROPHET_BUILD_TOOLS e PROPHET_ANDROID_JAR ajustam caminhos. PROPHET_ECJ_JAR permite o compilador Eclipse ECJ. Uma chave de assinatura é fornecida apenas por caminho externo em PROPHET_KEYSTORE; sem ela o script produz APK unsigned.

A biblioteca exporta AAudio/JNI e tem min API26. `libprophet_android.so` e o APK foram compilados e inspecionados como ARM64. Não houve emulador ARM nem Moto G8 físico: estado DEVICE_VERIFIED **não alcançado**.

Separadamente, `binaries/linux-aarch64` contém crates compilados para aarch64-unknown-linux-gnu. O backend CPAL/ALSA continua disponível em fonte para Linux ARM; não há executável Linux ARM linkado nem alegação de Linux nativo funcionando no telefone.
