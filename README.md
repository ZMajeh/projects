# Projects Overview

A collection of various software projects ranging from low-level system utilities to web applications and AI tools.

## Summary Table

| Project Name | Primary Language | Purpose |
| :--- | :--- | :--- |
| **KeyLogger(QT)** | C++/Qt | Windows keylogger using low-level hooks to record keystrokes. |
| **KoboldConExt** | TypeScript | VS Code extension for AI code completion via KoboldCPP. |
| **koboldToClaude** | Python | Proxy bridge for using Claude Code with local KoboldCPP instances. |
| **lodgeManagement** | Rust (Wasm) | Web-based lodge management system with Firebase and OCR. |
| **Majeh's PDF Viewer** | C/C++ | Lightweight document viewer with native touch and pinch-to-zoom. |
| **MyBook (QT)** | C++/Qt | Graphical book viewer and library management application. |
| **myQNotepad** | C++/Qt | Simple notepad application with standard editing features. |
| **NetCat(QT)** | C++/Qt | GUI tool for sending UDP datagrams and network testing. |
| **Tripple_X(CPP)** | C++ | A simple terminal-based number guessing game. |
| **WinClipVoice** | PowerShell | Clipboard monitor that reads text aloud using Windows TTS. |
| **WTLO** | HTML/JS | Grid-based canvas visualization and coordinate mapping tool. |

---

## Detailed Project Info

### KeyLogger(QT)
*   **Technologies:** C++, Qt Framework, Win32 API.
*   **Description:** A Windows utility that hooks into the keyboard stream to capture user input. It records both the literal character and the key name, saving them with timestamps into daily log files.
*   **Key Features:** Low-level keyboard hooks, real-time logging, Qt-based event loop.

### KoboldConExt
*   **Technologies:** TypeScript, VS Code API, Node.js, Webpack.
*   **Description:** An extension for Visual Studio Code that integrates with KoboldCPP. It provides AI-powered code suggestions, context-aware editing, and a built-in sidebar for the KoboldCPP interface.
*   **Key Features:** `KoboldEdit` for context generation, `KoboldAsk` for custom prompts, and streaming responses.

### koboldToClaude proxy bridge
*   **Technologies:** Python, FastAPI, Uvicorn, SSE.
*   **Description:** A high-performance proxy that translates Anthropic's message format into KoboldCPP's native format. This enables users to run "Claude Code" against their local LLMs.
*   **Key Features:** Real-time streaming (SSE), context management, and token estimation.

### lodgeManagement
*   **Technologies:** Rust, Leptos (Wasm), Firebase (Auth/Firestore), JavaScript, Tesseract.js.
*   **Description:** A full-featured web application for managing lodge operations. It handles room availability, guest check-ins, Aadhaar-based OCR for guest identification, and bill generation.
*   **Key Features:** Wasm-based frontend, Firebase integration, client-side OCR, and camera integration.

### Majeh's PDF Viewer (Touch Support)
*   **Technologies:** C, MuPDF Engine, Win32.
*   **Description:** A performance-focused document viewer optimized for speed and low memory usage. This specific edition adds native touch support for smooth pinch-to-zoom and scrolling on touch-enabled devices.
*   **Key Features:** Instant launch, supports PDF/EPUB/CBZ, native touch gestures, and a suite of command-line PDF tools (`mutool`, `mudraw`).

### MyBook (QT)
*   **Technologies:** C++, Qt Framework.
*   **Description:** A desktop application designed for viewing and managing a collection of digital books. It provides a visual interface for browsing library contents with metadata such as file size and last modified date.
*   **Key Features:** Visual library management, book previewing, and metadata tracking.

### myQNotepad
*   **Technologies:** C++, Qt Framework.
*   **Description:** A classic notepad clone built with the Qt framework. It provides a clean interface for basic text editing tasks.
*   **Key Features:** Standard file operations (New, Open, Save), Edit operations (Undo, Redo, Cut, Copy, Paste).

### NetCat(QT)
*   **Technologies:** C++, Qt Network Module.
*   **Description:** A graphical implementation of a network utility similar to Netcat, specifically focused on UDP communication. It allows users to send multiple datagrams to a target IP and port.
*   **Key Features:** UDP packet generation, configurable repetition, and user-friendly GUI.

### Tripple_X(CPP)
*   **Technologies:** C++.
*   **Description:** A simple yet engaging terminal-based puzzle game. Players must solve mathematical puzzles (finding numbers that add up and multiply to specific values) to "hack" through levels.
*   **Key Features:** Level-based progression, simple terminal I/O, procedural logic.

### WinClipVoice
*   **Technologies:** PowerShell, Windows Forms, System.Speech.
*   **Description:** A productivity tool that monitors the Windows clipboard. Whenever text is copied, the application automatically reads it out loud using the system's default text-to-speech engine.
*   **Key Features:** Background clipboard monitoring, "Read Aloud" controls, no external dependencies.

### WTLO (Way To Lead Others)
*   **Technologies:** HTML5 Canvas, JavaScript.
*   **Description:** A web-based grid visualization tool that renders a massive coordinate system on an HTML5 canvas. It maps out alphanumeric grids, likely for strategy mapping or coordinate-based planning.
*   **Key Features:** Large-scale canvas rendering, automated grid labeling, and coordinate mapping logic.
