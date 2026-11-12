# Control API v1

The standalone serves the React assets and these endpoints on 127.0.0.1:8787 by default. Audio stays on the host device; WebSocket carries telemetry only.

- `GET /api/v1/meta`: factory preset names/categories and parameter metadata.
- `GET /api/v1/state`: measured latest audio snapshot.
- `GET /api/v1/ws`: WebSocket JSON snapshots, nominal 25 Hz. Socket disconnect requests All Notes Off.
- `POST /api/v1/command`: bounded JSON command. Returns 202 on queue acceptance, 400 on validation failure, 429 on queue saturation. Acceptance is not a sample timestamp guarantee.
- `POST /api/v1/filter`: desktop-only, array of parameter values; returns a measured small-signal Rust impulse-response magnitude/phase, computed on a worker outside audio.

Examples:

```json
{"type":"midi","status":144,"data1":60,"data2":100}
{"type":"midi","status":128,"data1":60,"data2":0}
{"type":"param","id":23,"value":1200}
{"type":"preset","index":0}
{"type":"learn","id":23}
{"type":"trimpot","voice":0,"field":2,"value":0.015}
{"type":"panic"}
```

A snapshot command is `{"type":"snapshot","values":[...]}` with exactly 83 finite real values. `ui/src/param-specs.json` is the index contract. Field order is versioned and must remain stable. Trimpot fields are A cents, B cents, cutoff fraction, RC fraction. Cutoff 0.015 means +1.5%, not 1.5 Hz. Numerical values clamp to declared bounds; NaN/Inf controls are rejected/ignored at the boundary.

Desktop hardware/UI messages are quantized to the next callback; driver timestamps are not mapped to the device sample clock. The core `process_events` API, MIDI file renderer and plugin host paths support exact sample offsets. Network jitter and quantization are not reported as device latency. Desktop `underruns` is the CPAL stream-error counter, also labeled device errors in the UI; it does not prove every ALSA/PipeWire xrun was observed. Android uses AAudioStream_getXRunCount. MIDI queue overflow may release all notes to recover safely; there is no unlimited command queue.

Binding a desktop host beyond loopback requires environment variable PROPHET_CONTROL_TOKEN with at least 16 characters. Authenticated endpoints accept `Authorization: Bearer ...` or the `token` query field used by the browser WebSocket. No secret is committed or included in artifacts. HTTP is intended for a trusted LAN and is not an encrypted transport. Android uses a random per-install token and the same command/state protocol; Android lacks the desktop measured-filter endpoint and native USB MIDI enumeration. Android also accepts `{"type":"stop"}` to end the foreground service safely; a new authorized app launch is then needed to restart it.
