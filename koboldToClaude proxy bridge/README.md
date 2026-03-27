# Claude Code ↔ KoboldCPP Proxy

A high-performance FastAPI proxy that allows **Claude Code** (and other Anthropic-compatible clients) to communicate with a local **KoboldCPP** instance using its Native API.

## Features

- **Anthropic ↔ Kobold Native Translation**: Automatically converts Anthropic Messages format to Kobold Native prompt format (`{{[INPUT]}}` / `{{[OUTPUT]}}`).
- **Real-Time Streaming**: Implements strict Anthropic SSE (Server-Sent Events) formatting with mandatory `event:` headers for smooth, token-by-token streaming in Claude Code.
- **Optimized Samplers**: Forwards advanced samplers like `rep_pen`, `tfs`, and `sampler_order` specifically tuned for local LLMs.
- **Context Management**: Automatically caps `max_length` to ensure prompt + response fits within your local model's context (e.g., 4096 tokens), preventing incoherent outputs.
- **Token Estimation**: Provides realistic `input_tokens` and `output_tokens` usage data to satisfy client SDK requirements.
- **Claude Code Compatibility**: Includes necessary headers (`x-anthropic-version`) and a `/v1/models` endpoint to pass Claude Code's pre-flight checks.

## Prerequisites

- Python 3.10+
- A running [KoboldCPP](https://github.com/LostRuins/koboldcpp) instance.

## Installation

1. Clone this repository or copy the `proxy_server.py` file.
2. Create and activate a virtual environment:
   ```powershell
   python -m venv .venv
   .\.venv\Scripts\activate
   ```
3. Install dependencies:
   ```powershell
   pip install fastapi uvicorn httpx python-dotenv
   ```

## Configuration

Create a `.env` file in the project root:

```env
KOBOLD_BASE_URL=http://localhost:5001
DEFAULT_MODEL=koboldcpp
LOG_LEVEL=INFO
CORS_ALLOW_ORIGINS=*
```

- **KOBOLD_BASE_URL**: The URL where your KoboldCPP is running (usually port 5001).
- **DEFAULT_MODEL**: The model name reported to the client.

## Running the Proxy

Start the server using Uvicorn:

```powershell
python -m uvicorn proxy_server:app --host 0.0.0.0 --port 8080 --log-level info
```

The proxy will be available at `http://localhost:8080`.

## Connecting Claude Code

To use this proxy with Claude Code, you need to override the Anthropic base URL. In your terminal where you run `claude`, set the following environment variable:

**Windows (PowerShell):**
```powershell
$env:ANTHROPIC_BASE_URL="http://localhost:8080/v1"
claude
```

**Linux / macOS:**
```bash
export ANTHROPIC_BASE_URL="http://localhost:8080/v1"
claude
```

## How It Works

1. **Prompt Construction**: Every message from the user is prefixed with `{{[INPUT]}}` and every assistant response with `{{[OUTPUT]}}`. This is the standard format for many local models (Gemma, Llama, etc.) running on Kobold.
2. **Streaming Protocol**: Kobold sends `data: {token: "..."}`. The proxy translates this into `event: content_block_delta` + `data: {"type": "content_block_delta", ...}`.
3. **Safety Capping**: The proxy calculates the size of your input and instructs Kobold to only generate up to the remaining space in your 4096-token context, avoiding "context stripping" errors.

## Troubleshooting

- **"Stream completed without message_start"**: This is usually caused by buffering. The proxy includes `X-Accel-Buffering: no` headers to prevent this.
- **Coherent Warning**: If you see "outputs will not be very coherent" in your Kobold logs, it means the prompt is too long for the allocated context. The proxy now tries to mitigate this by capping `max_length`.
