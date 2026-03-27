import os
import json
import asyncio
import logging
from logging.handlers import RotatingFileHandler
from typing import Optional, AsyncGenerator
from urllib.parse import urljoin

from fastapi import FastAPI, Request, Header, HTTPException, Query
from fastapi.responses import StreamingResponse, JSONResponse
from fastapi.middleware.cors import CORSMiddleware
import httpx
from dotenv import load_dotenv

# Load .env if present
load_dotenv()

# Configuration
KOBOLD_ROOT_URL = os.getenv("KOBOLD_BASE_URL", "http://localhost:5001").split("/v1")[0].rstrip("/")
DEFAULT_MODEL = os.getenv("DEFAULT_MODEL", "koboldcpp")
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO").upper()
CORS_ALLOW_ORIGINS = os.getenv("CORS_ALLOW_ORIGINS", "*").split(",")

# Logging setup
logger = logging.getLogger("cloudcode-kobold-proxy")
logger.setLevel(LOG_LEVEL)
formatter = logging.Formatter("%(asctime)s %(levelname)s %(message)s")
ch = logging.StreamHandler()
ch.setFormatter(formatter)
logger.addHandler(ch)
fh = RotatingFileHandler("proxy.log", maxBytes=5 * 1024 * 1024, backupCount=3, encoding="utf-8")
fh.setFormatter(formatter)
logger.addHandler(fh)

app = FastAPI(title="CloudCode ↔ KoboldCPP Native Proxy")

app.add_middleware(
    CORSMiddleware,
    allow_origins=CORS_ALLOW_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

def build_native_prompt(messages: list) -> str:
    prompt = ""
    for m in messages:
        role = m.get("role", "user")
        content = m.get("content", "")
        if isinstance(content, list):
            content = "\n".join([c.get("text", str(c)) if isinstance(c, dict) else str(c) for c in content])
        
        if role == "user":
            prompt += f"{{{{[INPUT]}}}}{content}"
        else:
            prompt += f"{{{{[OUTPUT]}}}}{content}"
    
    if not prompt.endswith("{{[OUTPUT]}}"):
        prompt += "{{[OUTPUT]}}"
    return prompt

def estimate_tokens(text: str) -> int:
    return max(1, len(text) // 4)

def format_sse(event: str, data: dict) -> bytes:
    """Formats an Anthropic-compliant SSE message."""
    return f"event: {event}\ndata: {json.dumps(data)}\n\n".encode("utf-8")

async def native_stream_generator(native_payload: dict, requested_model: str) -> AsyncGenerator[bytes, None]:
    """Generates Anthropic SSE events from Kobold Native stream."""
    prompt_tokens = estimate_tokens(native_payload["prompt"])
    
    # 1. message_start
    yield format_sse("message_start", {
        "type": "message_start",
        "message": {
            "id": "msg_proxy",
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": requested_model,
            "stop_reason": None,
            "stop_sequence": None,
            "usage": {"input_tokens": prompt_tokens, "output_tokens": 0}
        }
    })
    
    # 2. content_block_start
    yield format_sse("content_block_start", {
        "type": "content_block_start",
        "index": 0,
        "content_block": {"type": "text", "text": ""}
    })

    stream_url = f"{KOBOLD_ROOT_URL}/api/extra/generate/stream"
    total_output = ""

    async with httpx.AsyncClient(timeout=None) as client:
        try:
            async with client.stream("POST", stream_url, json=native_payload) as resp:
                async for line in resp.aiter_lines():
                    if not line: continue
                    if line.startswith("data: "):
                        data_str = line[6:].strip()
                        try:
                            chunk = json.loads(data_str)
                            token = chunk.get("token", "")
                            if token:
                                total_output += token
                                yield format_sse("content_block_delta", {
                                    "type": "content_block_delta",
                                    "index": 0,
                                    "delta": {"type": "text_delta", "text": token}
                                })
                        except:
                            continue
                
                output_tokens = estimate_tokens(total_output)
                # 3. content_block_stop
                yield format_sse("content_block_stop", {"type": "content_block_stop", "index": 0})
                # 4. message_delta
                yield format_sse("message_delta", {
                    "type": "message_delta",
                    "delta": {"stop_reason": "end_turn", "stop_sequence": None},
                    "usage": {"output_tokens": output_tokens}
                })
                # 5. message_stop
                yield format_sse("message_stop", {"type": "message_stop"})
        except Exception as e:
            logger.error("Stream failed: %s", e)
            yield format_sse("error", {"type": "error", "error": {"type": "api_error", "message": str(e)}})

@app.get("/health")
async def health():
    return {"status": "ok", "kobold_url": KOBOLD_ROOT_URL}

@app.get("/v1/models")
async def list_models():
    return {"data": [{"id": "claude-haiku-4-5-20251001", "object": "model"}, {"id": "claude-sonnet-4-6[1m]", "object": "model"}]}

@app.post("/v1/messages")
async def proxy_messages(request: Request):
    try:
        payload = await request.json()
    except:
        raise HTTPException(status_code=400, detail="Invalid JSON")

    messages = payload.get("messages", [])
    prompt = build_native_prompt(messages)
    prompt_tokens = estimate_tokens(prompt)
    requested_model = payload.get("model", DEFAULT_MODEL)
    is_streaming = bool(payload.get("stream", False))

    available_for_gen = 4096 - prompt_tokens - 100
    max_length = min(payload.get("max_tokens", 1024), max(64, available_for_gen))

    native_payload = {
        "n": 1,
        "max_context_length": 4096,
        "max_length": max_length,
        "rep_pen": 1.05,
        "temperature": payload.get("temperature", 0.75),
        "top_p": payload.get("top_p", 0.92),
        "sampler_order": [6, 0, 1, 3, 4, 2, 5],
        "stop_sequence": ["{{[INPUT]}}", "{{[OUTPUT]}}"],
        "prompt": prompt,
        "quiet": True
    }

    logger.info("Request: model=%s stream=%s", requested_model, is_streaming)

    if is_streaming:
        return StreamingResponse(
            native_stream_generator(native_payload, requested_model),
            media_type="text/event-stream",
            headers={
                "X-Accel-Buffering": "no", 
                "Cache-Control": "no-cache",
                "Connection": "keep-alive",
                "x-anthropic-version": "2023-06-01"
            }
        )

    # Non-streaming
    gen_url = f"{KOBOLD_ROOT_URL}/api/v1/generate"
    async with httpx.AsyncClient() as client:
        try:
            resp = await client.post(gen_url, json=native_payload, timeout=300.0)
            resp.raise_for_status()
            body = resp.json()
            output_text = body["results"][0]["text"]
            output_tokens = estimate_tokens(output_text)
            
            return JSONResponse(
                content={
                    "id": "msg_proxy",
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "text", "text": output_text}],
                    "model": requested_model,
                    "stop_reason": "end_turn",
                    "usage": {"input_tokens": prompt_tokens, "output_tokens": output_tokens}
                },
                headers={"x-anthropic-version": "2023-06-01"}
            )
        except Exception as e:
            logger.exception("Native request failed")
            raise HTTPException(status_code=502, detail=str(e))

@app.post("/v1/complete")
async def proxy_complete(request: Request):
    return await proxy_messages(request)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8080)
