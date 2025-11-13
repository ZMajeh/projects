# <h1>WinClipVoice: Windows Clipboard TTS Reader</h1>

<p>WinClipVoice is a simple, lightweight utility written entirely in <strong>PowerShell</strong> that monitors your Windows clipboard for new text content and automatically reads it aloud using the built-in Windows Text-to-Speech (TTS) engine.</p>

<p>It features a graphical user interface (GUI) with buttons to manually paste, read the current text, or stop the speech. A background timer constantly checks the clipboard for changes.</p>

<h2>Features</h2>

<ul>
    <li><b>GUI Interface:</b> A clean Windows Forms window.</li>
    <li><b>Automatic Reading:</b> Listens for new text copied to the clipboard and speaks it instantly.</li>
    <li><b>Manual Controls:</b> Buttons for "Paste Clipboard", "Read Aloud", and "Stop Reading".</li>
    <li><b>Pure PowerShell:</b> No external dependencies beyond standard Windows components (<code>System.Speech</code>, <code>System.Windows.Forms</code>).</li>
    <li><b>Voice Selection:</b> Uses the default Windows system voice (configurable within the script).</li>
</ul>

<h2>Requirements</h2>

<ul>
    <li><strong>Operating System:</strong> Windows 7 or newer.</li>
    <li><strong>PowerShell Version:</strong> PowerShell 3.0 or higher.</li>
    <li><strong>.NET Framework:</strong> Required for the Windows Forms and Speech assemblies (standard on modern Windows systems).</li>
    <li><strong>System Voices:</strong> Your system must have at least one installed and enabled speech synthesis voice.</li>
</ul>

<h2>How to Run the Script</h2>

<p>The script must be executed in a <strong>Single-Threaded Apartment (STA) mode</strong> so that UI elements (the form, the buttons) and clipboard operations can interact correctly.</p>

<h3>1. Save the Code</h3>

<p>Save the entire script provided previously into a file with a <code>.ps1</code> extension. For this example, let's assume you save it as <code>WinClipVoice.ps1</code>.</p>

<h3>2. Execute via Command Prompt or Run</h3>

<p>Open a standard Command Prompt (<code>cmd</code>) or the Run dialog (<kbd>Win</kbd> + <kbd>R</kbd>), navigate to the script's directory, and use the following command:</p>

```bash
powershell -sta -File "C:\Path\To\WinClipVoice.ps1"
