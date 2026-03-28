# ALICE-TTS-SaaS

Text-to-Speech synthesis SaaS built on the ALICE-TTS engine. Delivers natural-sounding speech synthesis with SSML support, streaming output, and a rich voice library via REST API.

## Architecture

```
Client --> API Gateway (8110) --> Core Engine (8112)
```

- **API Gateway**: Authentication, rate limiting, request proxying
- **Core Engine**: TTS synthesis, voice management, SSML processing

## Features

- Neural TTS with natural prosody
- 100+ voices across 40+ languages
- SSML markup support (pitch, rate, emphasis, breaks)
- Streaming audio output
- Multiple output formats (PCM, WAV, MP3, OGG)
- Custom voice cloning endpoint
- Per-request voice style and speed control

## API Endpoints

### Core Engine (port 8112)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check with uptime and stats |
| POST | `/api/v1/tts/synthesize` | Synthesize text to audio |
| GET | `/api/v1/tts/voices` | List available voices |
| POST | `/api/v1/tts/ssml` | Synthesize SSML markup |
| POST | `/api/v1/tts/stream` | Streaming synthesis session |
| GET | `/api/v1/tts/stats` | Operational statistics |

### API Gateway (port 8110)

Proxies all `/api/v1/*` routes to the Core Engine with JWT/API-Key auth and token-bucket rate limiting.

## Quick Start

```bash
# Core Engine
cd services/core-engine
TTS_ADDR=0.0.0.0:8112 cargo run --release

# API Gateway
cd services/api-gateway
GATEWAY_ADDR=0.0.0.0:8110 CORE_ENGINE_URL=http://localhost:8112 cargo run --release
```

## Example Request

```bash
curl -X POST http://localhost:8112/api/v1/tts/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text":"Hello world","voice_id":"alice-ja-female-1","format":"wav","speed":1.0}'
```

## License

AGPL-3.0-or-later. SaaS operators must publish complete service source code under AGPL-3.0.
