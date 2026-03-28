use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

struct AppState { start_time: Instant, stats: Mutex<Stats> }
struct Stats { total_syntheses: u64, total_ssml_requests: u64, total_stream_sessions: u64, total_chars_synthesized: u64 }

#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_ops: u64 }

#[derive(Deserialize)]
struct SynthesizeRequest { text: String, voice_id: Option<String>, format: Option<String>, speed: Option<f32>, pitch: Option<f32>, volume: Option<f32> }
#[derive(Serialize)]
struct SynthesizeResponse { job_id: String, voice_id: String, format: String, audio_b64: String, duration_ms: u64, sample_rate: u32, char_count: u32, processing_ms: u128 }

#[derive(Deserialize)]
struct SsmlRequest { ssml: String, voice_id: Option<String>, format: Option<String> }
#[derive(Serialize)]
struct SsmlResponse { job_id: String, voice_id: String, format: String, audio_b64: String, duration_ms: u64, processing_ms: u128 }

#[derive(Deserialize)]
struct StreamRequest { session_id: Option<String>, text_chunk: String, voice_id: Option<String>, flush: Option<bool> }
#[derive(Serialize)]
struct StreamResponse { session_id: String, audio_chunk_b64: String, is_final: bool, chunk_duration_ms: u64 }

#[derive(Serialize)]
struct VoiceInfo { id: String, name: String, language: String, gender: String, style: Vec<String>, sample_rate: u32 }

#[derive(Serialize)]
struct StatsResponse { total_syntheses: u64, total_ssml_requests: u64, total_stream_sessions: u64, total_chars_synthesized: u64 }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "tts_engine=info".into())).init();
    let state = Arc::new(AppState { start_time: Instant::now(), stats: Mutex::new(Stats { total_syntheses: 0, total_ssml_requests: 0, total_stream_sessions: 0, total_chars_synthesized: 0 }) });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/tts/synthesize", post(synthesize))
        .route("/api/v1/tts/voices", get(voices))
        .route("/api/v1/tts/ssml", post(ssml))
        .route("/api/v1/tts/stream", post(stream))
        .route("/api/v1/tts/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("TTS_ADDR").unwrap_or_else(|_| "0.0.0.0:8112".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("TTS Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health { status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(), uptime_secs: s.start_time.elapsed().as_secs(), total_ops: st.total_syntheses + st.total_ssml_requests })
}

async fn synthesize(State(s): State<Arc<AppState>>, Json(req): Json<SynthesizeRequest>) -> Json<SynthesizeResponse> {
    let t = Instant::now();
    let voice_id = req.voice_id.unwrap_or_else(|| "alice-ja-female-1".into());
    let format = req.format.unwrap_or_else(|| "wav".into());
    let char_count = req.text.chars().count() as u32;
    let duration_ms = (char_count as u64) * 80;
    { let mut st = s.stats.lock().unwrap(); st.total_syntheses += 1; st.total_chars_synthesized += char_count as u64; }
    Json(SynthesizeResponse { job_id: uuid::Uuid::new_v4().to_string(), voice_id, format, audio_b64: "UklGRiQAAABXQVZF".into(), duration_ms, sample_rate: 24000, char_count, processing_ms: t.elapsed().as_millis() })
}

async fn voices() -> Json<Vec<VoiceInfo>> {
    Json(vec![
        VoiceInfo { id: "alice-ja-female-1".into(), name: "Akari".into(), language: "ja".into(), gender: "female".into(), style: vec!["neutral".into(), "happy".into(), "sad".into()], sample_rate: 24000 },
        VoiceInfo { id: "alice-ja-male-1".into(), name: "Haruto".into(), language: "ja".into(), gender: "male".into(), style: vec!["neutral".into(), "professional".into()], sample_rate: 24000 },
        VoiceInfo { id: "alice-en-female-1".into(), name: "Emma".into(), language: "en".into(), gender: "female".into(), style: vec!["neutral".into(), "cheerful".into(), "calm".into()], sample_rate: 24000 },
        VoiceInfo { id: "alice-en-male-1".into(), name: "James".into(), language: "en".into(), gender: "male".into(), style: vec!["neutral".into(), "newscast".into()], sample_rate: 24000 },
    ])
}

async fn ssml(State(s): State<Arc<AppState>>, Json(req): Json<SsmlRequest>) -> Json<SsmlResponse> {
    let t = Instant::now();
    let voice_id = req.voice_id.unwrap_or_else(|| "alice-ja-female-1".into());
    let format = req.format.unwrap_or_else(|| "wav".into());
    { let mut st = s.stats.lock().unwrap(); st.total_ssml_requests += 1; st.total_chars_synthesized += req.ssml.len() as u64; }
    Json(SsmlResponse { job_id: uuid::Uuid::new_v4().to_string(), voice_id, format, audio_b64: "UklGRiQAAABXQVZF".into(), duration_ms: 1500, processing_ms: t.elapsed().as_millis() })
}

async fn stream(State(s): State<Arc<AppState>>, Json(req): Json<StreamRequest>) -> Json<StreamResponse> {
    let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let is_final = req.flush.unwrap_or(false);
    { let mut st = s.stats.lock().unwrap(); if is_final { st.total_stream_sessions += 1; } st.total_chars_synthesized += req.text_chunk.len() as u64; }
    Json(StreamResponse { session_id, audio_chunk_b64: "UklGRiQAAABXQVZF".into(), is_final, chunk_duration_ms: (req.text_chunk.len() as u64) * 60 })
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse { total_syntheses: st.total_syntheses, total_ssml_requests: st.total_ssml_requests, total_stream_sessions: st.total_stream_sessions, total_chars_synthesized: st.total_chars_synthesized })
}
