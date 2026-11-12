# Public export

This repository exports the complete experimental PROPHET-XII delivery: source code, recorded validation evidence, compiled binaries, Android APK, WAVs, presets and the source-only Rust dependency snapshot. It does not add DSP features or claim new hardware validation.

- Repository: https://github.com/AdhonisBR/prophet-xii
- Download: https://github.com/AdhonisBR/prophet-xii/archive/refs/heads/main.zip
- The delivery report is a historical test record from before this export; its statement that no repository was published describes that earlier session. This public export was explicitly requested afterward.
- The GitHub integration could push Git commits, but uploads.github.com rejected Release attachment upload with HTTP 401. Deliverables are therefore versioned directly in the repository, with no missing binary/dependency files behind Release-only links.
- No build caches, node_modules, SDK/NDK distributions or private signing keys are included. The dependency archive contains upstream source and license/notice files.
- Original compiled deliverables are unchanged; standalone physical audio, Android emulator/Moto G8 operation, Windows/macOS builds and commercial DAW use remain unverified.
