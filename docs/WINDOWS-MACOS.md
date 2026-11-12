# Windows and macOS builds

Source paths use CPAL WASAPI (Windows) or CoreAudio (macOS), and midir's platform MIDI API. ASIO is not enabled. These operating systems/toolchains were not available for compilation or device tests in the delivery environment.

On Windows, install Rust stable MSVC, Microsoft C++ build tools and Node. Run `cargo build --release --locked -p prophet-standalone -p prophet-render -p prophet-plugin`, then `cd ui`, `npm ci`, `npm run build`. From the project root run `target\release\prophet-standalone.exe --ui ui/dist`. The plugin DLL exports CLAP/VST3 entry points; packaging a VST3 bundle and installing it in a host must be validated on Windows.

On macOS, install Rust, Xcode Command Line Tools and Node, then the same Cargo/npm commands. Run `target/release/prophet-standalone --ui ui/dist`. CLAP/VST3 bundle layout, signing, architecture coverage and host loading must be completed/tested on macOS. No Windows/macOS binary, notarization, DAW validation or device performance is claimed.
