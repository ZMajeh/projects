@echo off
echo Stopping the proxy server on port 8080...

:: Find the PID of the process listening on port 8080
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :8080 ^| findstr LISTENING') do (
    echo Found server with PID %%a. Terminating...
    taskkill /F /PID %%a
)

echo Server stopped (if it was running).
pause
