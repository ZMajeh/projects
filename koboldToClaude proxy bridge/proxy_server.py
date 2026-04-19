import os
import json
import asyncio
import logging
from logging.handlers import RotatingFileHandler
from typing import Optional, AsyncGenerator
import re
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

def fix_truncated_json(s: str) -> str:
    """Aggressively repairs truncated or malformed JSON from LLMs."""
    # 1. Collapse multiple backslashes (common in Windows paths from LLMs)
    s = re.sub(r'\\{2,}', r'\\\\', s)
    
    # 2. Escape literal newlines
    s = s.replace('\n', '\\n')
    
    # 3. Close dangling quotes
    unescaped_quotes = len(re.findall(r'(?<!\\)"', s))
    if unescaped_quotes % 2 != 0:
        s += '"'
    
    # 4. Balance braces
    open_braces = s.count('{')
    close_braces = s.count('}')
    if open_braces > close_braces:
        s += '}' * (open_braces - close_braces)
    
    return s

def parse_tool_code(text: str) -> Optional[dict]:
    """Robustly parses tool calls, handling truncation and malformed JSON."""
    tag = "<tool_code>"
    if tag not in text:
        return None
        
    try:
        start_pos = text.find(tag) + len(tag)
        end_pos = text.find("</tool_code>", start_pos)
        raw_content = text[start_pos:end_pos].strip() if end_pos != -1 else text[start_pos:].strip()
        
        if not raw_content.startswith("{"):
            first_brace = raw_content.find("{")
            if first_brace != -1:
                raw_content = raw_content[first_brace:]
            else:
                return None

        repaired_json = fix_truncated_json(raw_content)
        data = json.loads(repaired_json, strict=False)
        
        if data and data.get("name"):
            tool_name = data["name"]
            tool_id = str(data.get("id", f"tool_{os.urandom(8).hex()}"))
            if not tool_id.startswith("toolu_"):
                tool_id = f"toolu_{tool_id}"

            tool_input = data.get("input", {})
            if not isinstance(tool_input, dict):
                tool_input = {}

            if "file_path" in tool_input and isinstance(tool_input["file_path"], str):
                tool_input["file_path"] = re.sub(r'\\{2,}', r'\\', tool_input["file_path"])

            logger.info("Tool call parsed successfully: %s (%s)", tool_name, tool_id)
            return {
                "type": "tool_use",
                "id": tool_id,
                "name": tool_name,
                "input": tool_input
            }
    except Exception as e:
        logger.error("Failed to parse/repair tool code: %s", e)
        try:
            name_match = re.search(r'"name":\s*"Write"', raw_content)
            path_match = re.search(r'"file_path":\s*"([^"]+)"', raw_content)
            if name_match and path_match:
                content_match = re.search(r'"content":\s*"(.+)"', raw_content, re.DOTALL)
                content = content_match.group(1) if content_match else ""
                return {
                    "type": "tool_use",
                    "id": f"toolu_{os.urandom(8).hex()}",
                    "name": "Write",
                    "input": {
                        "file_path": re.sub(r'\\{2,}', r'\\', path_match.group(1)),
                        "content": content.replace('\\n', '\n').replace('\\"', '"')
                    }
                }
        except: pass
        
    return None

def build_native_prompt(messages: list, tools: Optional[list] = None) -> str:
    """Builds a Kobold-native format prompt from Claude-style messages, including tool definitions."""
    prompt = ""
    system_content = ""
    if tools:
        system_content = "This is a conversation with a function calling AI assistant.\n"
        system_content += "Here are the available functions:\n"
        system_content += "<tools>\n"
        for tool in tools:
            simple_tool = {
                "name": tool.get("name"),
                "description": tool.get("description", "").split("\n\n")[0],
                "input_schema": tool.get("input_schema")
            }
            system_content += json.dumps(simple_tool, indent=2) + "\n"
        system_content += "</tools>\n\n"
    
    for m in messages:
        if m.get("role") == "system":
            system_content += m.get("content", "") + "\n"
            break

    if system_content:
        prompt += f"{{{{[SYSTEM]}}}}{system_content.strip()}"

    for m in messages:
        role = m.get("role", "user")
        if role == "system": continue
        content = m.get("content", "")

        if role == "user":
            if isinstance(content, list):
                text_parts = []
                for item in content:
                    if item.get("type") == "tool_result" and "content" in item:
                        result_content = item['content']
                        if isinstance(result_content, list) and len(result_content) > 0:
                             result_content = result_content[0].get('text', '')
                        tool_use_id = item.get("tool_use_id", "")
                        text_parts.append(f"<tool_result>\n{{\"tool_use_id\": \"{tool_use_id}\", \"content\": \"{result_content}\"}}\n</tool_result>")
                    elif item.get("type") == "text":
                        text_parts.append(item.get("text", ""))
                prompt += f"{{{{[INPUT]}}}}{' '.join(text_parts)}"
            else:
                prompt += f"{{{{[INPUT]}}}}{content}"
        elif role == "assistant":
            full_content = ""
            if isinstance(content, list):
                for part in content:
                    if part.get("type") == "text":
                        full_content += part.get("text", "")
                    elif part.get("type") == "tool_use":
                        tool_name = part.get("name")
                        tool_input = part.get("input")
                        tool_use_id = part.get("id")
                        full_content += f"<tool_code>\n{{\"name\": \"{tool_name}\", \"id\": \"{tool_use_id}\", \"input\": {json.dumps(tool_input)}}}\n</tool_code>"
            else:
                full_content = content
            prompt += f"{{{{[OUTPUT]}}}}{full_content}"
        else:
            prompt += f"{{{{[OUTPUT]}}}}{content}"
            
    if tools:
        instructions = "\n### TOOL CALLING RULES ###\n"
        instructions += "To use a tool, you MUST output a <tool_code> block containing a JSON object.\n"
        instructions += "The JSON object MUST have 'name', 'id', and 'input' keys.\n"
        instructions += "DO NOT use markdown code blocks like ```tool or ```python for tool calls.\n"
        instructions += "Example:\n<tool_code>\n{\"name\": \"Grep\", \"id\": \"123\", \"input\": {\"pattern\": \"foo\"}}\n</tool_code>\n"
        prompt += instructions

    if not prompt.endswith("{{[OUTPUT]}}"):
        prompt += "{{[OUTPUT]}}"
    return prompt

def estimate_tokens(text: str) -> int:
    return max(1, len(text) // 4)

def format_sse(event: str, data: dict) -> bytes:
    """Formats an Anthropic-compliant SSE message."""
    return f"event: {event}\ndata: {json.dumps(data)}\n\n".encode("utf-8")

async def native_stream_generator(native_payload: dict, requested_model: str) -> AsyncGenerator[bytes, None]:
    """Generates Anthropic SSE events from Kobold Native stream, handling text and tool use."""
    prompt_tokens = estimate_tokens(native_payload.get("prompt", ""))
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
    
    stream_url = f"{KOBOLD_ROOT_URL}/api/extra/generate/stream"
    buffer = ""
    text_generated = False
    text_content_index = 0

    async with httpx.AsyncClient(timeout=None) as client:
        try:
            async with client.stream("POST", stream_url, json=native_payload) as resp:
                async for line in resp.aiter_lines():
                    if not line or not line.startswith("data: "):
                        continue
                    data_str = line[len("data: "):].strip()
                    try:
                        chunk = json.loads(data_str)
                        token = chunk.get("token", "")
                        if not token: continue
                        buffer += token
                        if not text_generated and "<tool_code>" not in buffer:
                            yield format_sse("content_block_start", {
                                "type": "content_block_start",
                                "index": text_content_index,
                                "content_block": {"type": "text", "text": ""}
                            })
                            text_generated = True
                        if text_generated and "<tool_code>" not in buffer:
                             yield format_sse("content_block_delta", {
                                "type": "content_block_delta",
                                "index": text_content_index,
                                "delta": {"type": "text_delta", "text": token}
                            })
                    except json.JSONDecodeError: continue

            logger.info("Raw model output (streamed):\n%s", buffer)
            stop_reason = "end_turn"
            if text_generated:
                yield format_sse("content_block_stop", {"type": "content_block_stop", "index": text_content_index})

            tool_use_content = parse_tool_code(buffer)
            final_output_text = buffer
            if tool_use_content:
                stop_reason = "tool_use"
                text_before_tool = buffer.split("<tool_code>")[0].strip()
                if not text_generated and text_before_tool:
                     yield format_sse("content_block_start", {"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": text_before_tool}})
                     yield format_sse("content_block_stop", {"type": "content_block_stop", "index": 0})
                tool_index = 1 if text_generated or (not text_generated and text_before_tool) else 0
                yield format_sse("content_block_start", {"type": "content_block_start", "index": tool_index, "content_block": tool_use_content})
                yield format_sse("content_block_stop", {"type": "content_block_stop", "index": tool_index})
                final_output_text = text_before_tool 

            output_tokens = estimate_tokens(final_output_text)
            yield format_sse("message_delta", {
                "type": "message_delta",
                "delta": {"stop_reason": stop_reason, "stop_sequence": None},
                "usage": {"output_tokens": output_tokens}
            })
            yield format_sse("message_stop", {"type": "message_stop"})
        except Exception as e:
            logger.error("Stream failed: %s", e)
            yield format_sse("error", {"type": "error", "error": {"type": "api_error", "message": str(e)}})

@app.get("/")
@app.head("/")
async def root():
    return {"status": "ok", "proxy": "cloudcode-kobold-proxy"}

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
    tools = payload.get("tools")
    prompt = build_native_prompt(messages, tools)
    prompt_tokens = estimate_tokens(prompt)
    requested_model = payload.get("model", DEFAULT_MODEL)
    is_streaming = bool(payload.get("stream", False))

    available_for_gen = 99096 - prompt_tokens - 100
    max_length = min(payload.get("max_tokens", 4096), max(64, available_for_gen))

    native_payload = {
        "n": 1,
        "max_context_length": 99096,
        "max_length": max_length,
        "rep_pen": 1.05,
        "temperature": payload.get("temperature", 0.75),
        "top_p": payload.get("top_p", 0.92),
        "sampler_order": [6, 0, 1, 3, 4, 2, 5],
        "stop_sequence": ["{{[INPUT]}}", "{{[OUTPUT]}}", "</tool_code>"],
        "prompt": prompt,
        "quiet": True
    }

    logger.info("Request: model=%s stream=%s", requested_model, is_streaming)
    if tools:
        logger.info("Tools provided: %s", json.dumps(tools, indent=2))
    logger.debug("Full Prompt:\n%s", prompt)

    if is_streaming:
        streaming_payload = native_payload.copy()
        return StreamingResponse(
            native_stream_generator(streaming_payload, requested_model),
            media_type="text/event-stream",
            headers={
                "X-Accel-Buffering": "no", 
                "Cache-Control": "no-cache",
                "Connection": "keep-alive",
                "x-anthropic-version": "2023-06-01"
            }
        )

    gen_url = f"{KOBOLD_ROOT_URL}/api/v1/generate"
    async with httpx.AsyncClient() as client:
        try:
            resp = await client.post(gen_url, json=native_payload, timeout=300.0)
            resp.raise_for_status()
            body = resp.json()
            output_text = body["results"][0]["text"]
            logger.info("Raw model output (non-streamed):\n%s", output_text)
            output_tokens = estimate_tokens(output_text)
            tool_use_content = parse_tool_code(output_text)
            final_content = []
            stop_reason = "end_turn"
            if tool_use_content:
                final_content.append(tool_use_content)
                text_before_tool = output_text.split("<tool_code>")[0].strip()
                if text_before_tool:
                    final_content.insert(0, {"type": "text", "text": text_before_tool})
                stop_reason = "tool_use"
            else:
                final_content.append({"type": "text", "text": output_text})

            return JSONResponse(
                content={
                    "id": "msg_proxy",
                    "type": "message",
                    "role": "assistant",
                    "content": final_content,
                    "model": requested_model,
                    "stop_reason": stop_reason,
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
