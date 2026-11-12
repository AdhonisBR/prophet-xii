// SPDX-License-Identifier: GPL-3.0-only
use axum::{
    extract::{
        ws::{Message, WebSocketUpgrade},
        Query, State,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use prophet_params::{NUM_PARAMS, PARAM_SPECS};
use prophet_standalone::{AudioProcessor, Command, Telemetry};
use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::{mpsc, watch};
use tower_http::services::ServeDir;
#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<Command>,
    telemetry: watch::Receiver<String>,
    token: Option<Arc<str>>,
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Param {
        id: usize,
        value: f32,
    },
    Midi {
        status: u8,
        data1: u8,
        #[serde(default)]
        data2: u8,
    },
    Preset {
        index: usize,
    },
    Snapshot {
        values: Vec<f32>,
    },
    Learn {
        id: usize,
    },
    Panic,
    Trimpot {
        voice: usize,
        field: usize,
        value: f32,
    },
}
fn authorized(s: &AppState, h: &HeaderMap, q: &HashMap<String, String>) -> bool {
    s.token
        .as_ref()
        .map(|t| {
            h.get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.strip_prefix("Bearer ") == Some(t.as_ref()))
                .unwrap_or(false)
                || q.get("token").map(|v| v == t.as_ref()).unwrap_or(false)
        })
        .unwrap_or(true)
}
async fn command(
    State(s): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    Json(req): Json<Request>,
) -> Response {
    if !authorized(&s, &headers, &q) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let c = match req {
        Request::Param { id, value } if id < NUM_PARAMS && value.is_finite() => {
            Command::Param { id, value }
        }
        Request::Midi {
            status,
            data1,
            data2,
        } if (0x80..0xf0).contains(&status) && data1 < 128 && data2 < 128 => Command::Midi {
            status,
            data1,
            data2,
        },
        Request::Preset { index } if index < 10 => Command::Preset(index),
        Request::Snapshot { values }
            if values.len() == NUM_PARAMS && values.iter().all(|v| v.is_finite()) =>
        {
            Command::Snapshot(values.try_into().unwrap())
        }
        Request::Learn { id } if id < NUM_PARAMS => Command::Learn(id),
        Request::Panic => Command::Panic,
        Request::Trimpot {
            voice,
            field,
            value,
        } if voice < 12 && field < 4 && value.is_finite() => Command::Trimpot {
            voice,
            field,
            value,
        },
        _ => return (StatusCode::BAD_REQUEST, "invalid command").into_response(),
    };
    match s.tx.try_send(c) {
        Ok(()) => StatusCode::ACCEPTED.into_response(),
        Err(_) => (StatusCode::TOO_MANY_REQUESTS, "control queue full").into_response(),
    }
}
async fn state(
    State(s): State<AppState>,
    h: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    if !authorized(&s, &h, &q) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    (
        [("content-type", "application/json")],
        s.telemetry.borrow().clone(),
    )
        .into_response()
}
async fn ws(
    State(s): State<AppState>,
    h: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    upgrade: WebSocketUpgrade,
) -> Response {
    if !authorized(&s, &h, &q) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    upgrade.on_upgrade(move|mut socket|async move{
  let mut rx=s.telemetry.clone();
  loop {
   let message=rx.borrow_and_update().clone();
   if socket.send(Message::Text(message.into())).await.is_err(){break;}
   tokio::select!{
    _=rx.changed()=>{},
    incoming=socket.recv()=>match incoming {Some(Ok(Message::Close(_)))|None|Some(Err(_))=>break,_=>continue}
   }
  }
  // Releasing notes after disconnect prevents a remote keyboard from leaving stuck gates.
  let _=s.tx.send(Command::Panic).await;
 }).into_response()
}
async fn filter_response(
    State(s): State<AppState>,
    h: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    Json(values): Json<Vec<f32>>,
) -> Response {
    if !authorized(&s, &h, &q) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if values.len() != NUM_PARAMS || values.iter().any(|v| !v.is_finite()) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let result = tokio::task::spawn_blocking(move || {
        use prophet_params::ParamId::*;
        let mut f = prophet_dsp::vcf_moog::DualVcfStage::new(48000.0, 1.0);
        f.set_params(
            values[FilterCutoff as usize],
            values[FilterResonance as usize],
            values[FilterDrive as usize],
            values[MoogBassComp as usize],
            values[FilterMorph as usize],
        );
        let mut impulse = [0.0f32; 4096];
        for (i, y) in impulse.iter_mut().enumerate() {
            *y = f.process_sample(if i == 0 { 0.0001 } else { 0.0 }) / 0.0001;
        }
        let mut hz = Vec::new();
        let mut db = Vec::new();
        let mut phase = Vec::new();
        for k in 0..256 {
            let freq = 20.0f64 * 1000.0f64.powf(k as f64 / 255.0);
            let mut re = 0.0;
            let mut im = 0.0;
            for (i, &v) in impulse.iter().enumerate() {
                let a = -std::f64::consts::TAU * freq * i as f64 / 48000.0;
                re += v as f64 * a.cos();
                im += v as f64 * a.sin();
            }
            hz.push(freq);
            db.push(20.0 * re.hypot(im).max(1e-9).log10());
            phase.push(im.atan2(re));
        }
        serde_json::json!({"hz":hz,"db":db,"phase":phase})
    })
    .await;
    match result {
        Ok(v) => Json(v).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
fn wire(t: &Telemetry, underruns: u64) -> String {
    serde_json::json!({"v":1,"ready":true,"sampleRate":t.sample_rate,"activeVoiceMask":t.kernel.active_voice_mask,"voiceVcaLevels":t.kernel.voice_vca_levels,"voiceFiltEnvLevels":t.kernel.voice_filt_env_levels,"voiceMidiNotes":t.kernel.voice_midi_notes,"scope":t.scope.as_slice(),"dspLoadPercent":t.cpu,"peakDspLoadPercent":t.peak_cpu,"deadlineMisses":t.deadline_misses,"underruns":underruns,"frames":t.frames,"values":t.params.as_slice(),"trimpots":t.trimpots,"lateCommands":t.late_commands,"droppedTelemetry":t.dropped_telemetry}).to_string()
}
fn stream_for<T: cpal::SizedSample + cpal::FromSample<f32>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut p: AudioProcessor,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let channels = config.channels as usize;
    let errors = p.underruns.clone();
    device.build_output_stream(
        config,
        move |out: &mut [T], _| p.process(out, channels),
        move |_| {
            errors.fetch_add(1, Ordering::Relaxed);
        },
        None,
    )
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |k: &str| args.iter().any(|v| v == k);
    let val = |k: &str| args.windows(2).find(|v| v[0] == k).map(|v| v[1].clone());
    if has("--help") {
        println!("prophet-standalone [--bind 127.0.0.1:8787] [--ui ui/dist] [--device NAME] [--midi-port 0] [--duration SECONDS] [--demo] [--null]\nRemote bind requires PROPHET_CONTROL_TOKEN. --null is an explicit test sink, without audio device.");
        return Ok(());
    }
    let host = cpal::default_host();
    if has("--list-devices") {
        for d in host.output_devices()? {
            println!("{}", d.name()?)
        }
        return Ok(());
    }
    let bind = val("--bind").unwrap_or("127.0.0.1:8787".into());
    let addr: std::net::SocketAddr = bind.parse()?;
    let token = std::env::var("PROPHET_CONTROL_TOKEN")
        .ok()
        .filter(|t| t.len() >= 16)
        .map(Arc::<str>::from);
    if !addr.ip().is_loopback() && token.is_none() {
        return Err("remote control requires PROPHET_CONTROL_TOKEN (at least 16 characters)".into());
    }
    let (mut producer, consumer) = rtrb::RingBuffer::new(1024);
    let (telemetry_tx, mut telemetry_rx) = rtrb::RingBuffer::new(4);
    let underruns = Arc::new(AtomicU64::new(0));
    let device = if has("--null") {
        None
    } else if let Some(name) = val("--device") {
        Some(
            host.output_devices()?
                .find(|d| d.name().ok().as_ref() == Some(&name))
                .ok_or("audio device not found")?,
        )
    } else {
        Some(
            host.default_output_device()
                .ok_or("no audio output device; use --list-devices to diagnose")?,
        )
    };
    let mut config = device
        .as_ref()
        .map(|d| d.default_output_config())
        .transpose()?;
    if let Some(d) = device.as_ref() {
        if let Ok(mut ranges) = d.supported_output_configs() {
            if let Some(range) = ranges.find(|r| {
                r.channels() == 2
                    && r.min_sample_rate().0 <= 48000
                    && r.max_sample_rate().0 >= 48000
                    && r.sample_format() == cpal::SampleFormat::F32
            }) {
                config = Some(range.with_sample_rate(cpal::SampleRate(48000)));
            }
        }
    }
    let sr = config.as_ref().map(|c| c.sample_rate().0).unwrap_or(48000);
    let mut processor = AudioProcessor::new(sr as f32, consumer, telemetry_tx, underruns.clone());
    if has("--demo") {
        for n in [48, 55, 60, 64] {
            processor.command(Command::Midi {
                status: 0x90,
                data1: n,
                data2: 96,
            })
        }
    }
    let (tx, mut rx) = mpsc::channel(512);
    let (wire_tx, wire_rx) = watch::channel("{\"v\":1,\"ready\":false}".to_string());
    let error_count = underruns.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));
        loop {
            interval.tick().await;
            // A single producer owns the RT queue; HTTP and hardware MIDI merge before this boundary.
            for _ in 0..512 {
                match rx.try_recv() {
                    Ok(c) => {
                        if producer.push(c).is_err() {
                            // Release overflowed notes safely on the next available slot.
                            while producer.push(Command::Panic).is_err() {
                                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            while let Ok(t) = telemetry_rx.pop() {
                wire_tx.send_replace(wire(&t, error_count.load(Ordering::Relaxed)));
            }
        }
    });
    let _midi = if let Some(port) = val("--midi-port") {
        let mut input = midir::MidiInput::new("PROPHET-XII")?;
        input.ignore(midir::Ignore::None);
        let ports = input.ports();
        let selected = ports
            .get(port.parse::<usize>()?)
            .ok_or("MIDI port does not exist")?;
        let tx = tx.clone();
        Some(input.connect(
            selected,
            "PROPHET-XII input",
            move |_, msg, _| {
                if msg.len() >= 2 {
                    let _ = tx.try_send(Command::Midi {
                        status: msg[0],
                        data1: msg[1],
                        data2: *msg.get(2).unwrap_or(&0),
                    });
                }
            },
            (),
        )?)
    } else {
        None
    };
    let _stream;
    if let Some(device) = device {
        let supported = config.unwrap();
        let cfg = supported.config();
        eprintln!(
            "Audio device: {} / {} Hz / {} channels / {:?}",
            device.name()?,
            sr,
            cfg.channels,
            supported.sample_format()
        );
        _stream = Some(match supported.sample_format() {
            cpal::SampleFormat::F32 => stream_for::<f32>(&device, &cfg, processor)?,
            cpal::SampleFormat::I16 => stream_for::<i16>(&device, &cfg, processor)?,
            cpal::SampleFormat::U16 => stream_for::<u16>(&device, &cfg, processor)?,
            _ => return Err("unsupported device sample format".into()),
        });
        _stream.as_ref().unwrap().play()?;
    } else {
        _stream = None;
        eprintln!("NULL TEST SINK: no playback device");
        std::thread::spawn(move || {
            let mut buf = [0.0f32; 512];
            let mut deadline = std::time::Instant::now();
            loop {
                processor.process(&mut buf, 2);
                deadline += std::time::Duration::from_secs_f64(256.0 / 48000.0);
                std::thread::sleep(deadline.saturating_duration_since(std::time::Instant::now()));
            }
        });
    }
    let presets = prophet_params::presets::factory_presets();
    let metadata = serde_json::json!({"v":1,"parameters":PARAM_SPECS.iter().map(|s|serde_json::json!({"id":s.id as usize,"name":s.name,"min":s.min,"max":s.max})).collect::<Vec<_>>(),"presets":presets.iter().map(|p|serde_json::json!({"id":p.id,"name":p.name,"category":p.category.as_str()})).collect::<Vec<_>>()});
    let static_dir = PathBuf::from(val("--ui").unwrap_or("ui/dist".into()));
    let app = Router::new()
        .route("/api/v1/command", post(command))
        .route("/api/v1/state", get(state))
        .route("/api/v1/filter", post(filter_response))
        .route("/api/v1/ws", get(ws))
        .route(
            "/api/v1/meta",
            get(move || async move { Json(metadata.clone()) }),
        )
        .fallback_service(ServeDir::new(static_dir))
        .with_state(AppState {
            tx: tx.clone(),
            telemetry: wire_rx,
            token,
        });
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    println!("PROPHET-XII control: http://{bind}");
    let duration = val("--duration").map(|v| v.parse::<u64>()).transpose()?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            if let Some(seconds) = duration {
                tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
            } else {
                let _ = tokio::signal::ctrl_c().await;
            }
            let _ = tx.send(Command::Panic).await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        })
        .await?;
    Ok(())
}
