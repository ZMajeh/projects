@echo off
echo Starting Claude Code ↔ KoboldCPP Proxy...
if not exist .venv (
    echo Error: Virtual environment not found. Run setup.bat first.
    pause
    exit /b 1
)

.venv\Scripts\python.exe -m uvicorn proxy_server:app --host 0.0.0.0 --port 8080 --log-level info
pause
