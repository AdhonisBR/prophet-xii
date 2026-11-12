# License and provenance

Project license: GNU GPL version 3 only (`GPL-3.0-only`), selecting version 3 from the baseline's GPL-3.0-or-later grant. The full unmodified GNU GPLv3 text is in LICENSE. The uploaded source is preserved as supplied, with exact ZIP and extracted-file hashes. Its third-party attribution comments were not treated as proof that it was a faithful port or manufacturer-calibrated instrument.

The core DSP has no external crates. Existing user-supplied modules were audited and modified. New filter equations, FIR construction, interpolation/modulation plumbing and test hosts were implemented here; no proprietary algorithm, product firmware, binary asset or factory calibration was imported. Preset manufacturer-author claims were removed. Names describe design inspirations, not endorsements.

## Material actually consulted

- The full original prompt in `docs/prompt.md`, added after the initial audit, and the execution request in `baseline/REQUEST.md`.
- Zavalishin TPT one-pole/cascade algebra as specified in the prompt, independently derived in `docs/DSP.md` and checked by residual tests. The original linked VAFilterDesign book was not copied into this package.
- Surge `sst-filters/include/sst/filters/VintageLadders.h`: nonlinear ladder differential equations, alternative RK implementation and license notice. The delivered solver is a separate trapezoidal implicit solution, not a port of that header. `CytomicSVF.h` was consulted for TPT state-variable structure. Upstream GPLv3-or-later, with specific permissive notices in the headers.
- OB-Xd ObxdVoice.h architecture/tolerance fields, YKChorus ChorusEngine.h structure and CloudSeed project documentation and GhostNoteAudio/CloudSeedCore ModulatedAllpass.h (MIT) were consulted as contextual comparisons. No code was copied from them in this implementation pass. The inspected legacy OB-Xd header and YKChorus source/license state GPLv2, so its availability alone was not treated as permission to incorporate GPLv2-only code into GPLv3. The delivered chorus derives from the user's baseline and independently implemented delay/filter mathematics.
- NIH-plug README, example sine plugin, MIDI/buffer/parameter/context definitions and exporter sources at commit `de421011f41a6d10fc8c7a6084e4f4dee0143683` were used for the actual wrapper. NIH-plug declares ISC. Its vst3-sys dependency is GPLv3 compatible; exact revisions and license declarations are in Cargo.lock/DEPENDENCIES.json.
- Official CLAP headers (MIT) used only to compile the local C verification host; the host source is in tests/clap_smoke.c. The framework's clap-sys revision is separately pinned in Cargo.lock.
- Official Steinberg VST3 SDK validator was built from the checked-out SDK and executed. The SDK is a validation tool, not incorporated into the synthesizer or redistributable archive. Its exact revision is recorded in evidence/tool-versions.txt.
- Android NDK r27c AAudio public API and Android SDK35 service/notification/JNI API contracts were used to build the Android implementation. SDK/NDK/compiler distributions are not packaged. D8 desugars Java8 lambdas for API26. ECJ is a build tool, not a runtime dependency.

## Dependency inventory and source

DEPENDENCIES.json records resolved Rust and npm packages with declared licenses and versions. Cargo.lock pins sources/checksums; package-lock.json pins npm integrity hashes. `third_party/rust-sources.tar.gz` contains a compact source-only Cargo vendor snapshot, including applicable upstream license and notice files, not build caches. Extract it under `third_party/`, then copy `third_party/cargo-vendor-config.toml` into `.cargo/config.toml` to build Rust offline. The archive is included to make the linked Rust dependency sources reviewable without a giant unpacked dependency directory in the delivery.

React and ReactDOM's MIT notices are preserved in `third_party/NOTICES.txt`; the Vite output retains dependency license banners. Build-only npm tools remain reconstructible from the lockfile and their public source locations. System libraries such as libc/ALSA and platform drivers are dynamically linked and not distributed here. Generated declarations are not a legal certification of every upstream source file; source notices remain authoritative. No detected license conflict was knowingly incorporated.
