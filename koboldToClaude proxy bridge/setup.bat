@echo off
SETLOCAL
echo Checking for Python...
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Python is not installed or not in PATH. Please install Python 3.10+ and try again.
    pause
    exit /b 1
)

echo Creating Virtual Environment...
if not exist .venv (
    python -m venv .venv
    echo Virtual environment created.
) else (
    echo Virtual environment already exists.
)

echo Installing dependencies...
.venv\Scripts\python.exe -m pip install --upgrade pip
.venv\Scripts\python.exe -m pip install fastapi uvicorn httpx python-dotenv

if not exist .env (
    echo Creating default .env file...
    echo KOBOLD_BASE_URL=http://localhost:5001 > .env
    echo DEFAULT_MODEL=koboldcpp >> .env
    echo LOG_LEVEL=INFO >> .env
)

echo Setup complete!
pause
