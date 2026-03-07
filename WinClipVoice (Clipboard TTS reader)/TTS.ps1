# Load the necessary assemblies FIRST
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Speech

# Create the SpeechSynthesizer object initially
$speaker = New-Object System.Speech.Synthesis.SpeechSynthesizer

# --- DIAGNOSTIC CHECK & Setup ---
$voices = $speaker.GetInstalledVoices()
if ($voices.Count -eq 0) {
    [System.Windows.Forms.MessageBox]::Show("No speech voices found on this system. Cannot read aloud.", "Error", "OK", "Error")
    return
}
$script:lastClipboardText = ""
$script:isListenerActive = $true # Listener starts active by default

# --- Create the Form ---
$form = New-Object System.Windows.Forms.Form
$form.Text = "TTS Clipboard Reader"
$form.Size = New-Object System.Drawing.Size(440, 480)
$form.StartPosition = "CenterScreen"
$form.MinimumSize = New-Object System.Drawing.Size(440, 480)

# Function to toggle the listener state and update the button texts
function ToggleListener {
    if ($script:isListenerActive) {
        $timer.Stop()
        $script:isListenerActive = $false
        $buttonStartListener.Enabled = $true
        $buttonStopListener.Enabled = $false
        Write-Host "Clipboard Listener STOPPED" -ForegroundColor Red
    } else {
        $timer.Start()
        $script:isListenerActive = $true
        $buttonStartListener.Enabled = $false
        $buttonStopListener.Enabled = $true
        Write-Host "Clipboard Listener STARTED" -ForegroundColor Green
    }
}

# --- Event Handlers ---
$form.Add_Load({
    $buttonStartListener.Enabled = $false
    $buttonStopListener.Enabled = $true
})
$form.Add_FormClosing({
    $speaker.Dispose()
    $timer.Dispose()
})

# Helper functions (remain the same)
function GenerateSsml($text) {
    $rate = $numericRate.Value
    $volume = $trackBarVolume.Value
    $pitch = $comboBoxPitch.SelectedItem
    $interval = $numericInterval.Value
    $escapedText = $text -replace '&', '&amp;' -replace '<', '&lt;' -replace '>', '&gt;' -replace '"', '&quot;' -replace "'", '&apos;'
    $ssml = @"
<speak version="1.0" xml:lang="en-US">
  <prosody rate="$rate" volume="$volume" pitch="$pitch">
    $escapedText
  </prosody>
  <break time="${interval}ms" />
</speak>
"@
    return $ssml
}
function RestartReadingIfTextPresent {
    if ($textBox.Text.Trim() -ne "") {
        $speaker.SpeakAsyncCancelAll()
        $ssml = GenerateSsml $textBox.Text
        $speaker.SpeakSsmlAsync($ssml) | Out-Null
    }
}
function SafelyChangeVoice($voiceName) {
    $speaker.SpeakAsyncCancelAll()
    $speaker.Dispose()
    $script:speaker = New-Object System.Speech.Synthesis.SpeechSynthesizer
    $script:speaker.SelectVoice($voiceName)
    $script:speaker.Rate = $numericRate.Value
    $script:speaker.Volume = $trackBarVolume.Value
    RestartReadingIfTextPresent
}
function UpdateAndSpeak {
    if ($script:isListenerActive -and [System.Windows.Forms.Clipboard]::ContainsText()) {
        $currentText = [System.Windows.Forms.Clipboard]::GetText().Trim()
        $lastRecordedText = $script:lastClipboardText.Trim()
        if ($currentText -ne $lastRecordedText -and $currentText -ne "") {
            Write-Host "--- CLIPBOARD CHANGE DETECTED ---" -ForegroundColor Cyan
            $script:lastClipboardText = $currentText
            $textBox.Text = $currentText
            RestartReadingIfTextPresent 
        }
    }
}

# --- Add TableLayoutPanel to Form (UI Layout) ---
$tableLayoutPanel = New-Object System.Windows.Forms.TableLayoutPanel
$tableLayoutPanel.Dock = "Fill"
$tableLayoutPanel.Padding = New-Object System.Windows.Forms.Padding(5)
$tableLayoutPanel.RowCount = 6
$tableLayoutPanel.ColumnCount = 4
$form.Controls.Add($tableLayoutPanel)

# Configure Column Styles:
[void]$tableLayoutPanel.ColumnStyles.Add((New-Object System.Windows.Forms.ColumnStyle([System.Windows.Forms.SizeType]::Percent, 25)))
[void]$tableLayoutPanel.ColumnStyles.Add((New-Object System.Windows.Forms.ColumnStyle([System.Windows.Forms.SizeType]::Percent, 35)))
[void]$tableLayoutPanel.ColumnStyles.Add((New-Object System.Windows.Forms.ColumnStyle([System.Windows.Forms.SizeType]::Percent, 25)))
[void]$tableLayoutPanel.ColumnStyles.Add((New-Object System.Windows.Forms.ColumnStyle([System.Windows.Forms.SizeType]::Percent, 15)))

# Configure Row Styles:
[void]$tableLayoutPanel.RowStyles.Add((New-Object System.Windows.Forms.RowStyle([System.Windows.Forms.SizeType]::Percent, 50)))
[void]$tableLayoutPanel.RowStyles.Add((New-Object System.Windows.Forms.RowStyle([System.Windows.Forms.SizeType]::Absolute, 35)))
[void]$tableLayoutPanel.RowStyles.Add((New-Object System.Windows.Forms.RowStyle([System.Windows.Forms.SizeType]::Absolute, 40)))
[void]$tableLayoutPanel.RowStyles.Add((New-Object System.Windows.Forms.RowStyle([System.Windows.Forms.SizeType]::Absolute, 40)))
[void]$tableLayoutPanel.RowStyles.Add((New-Object System.Windows.Forms.RowStyle([System.Windows.Forms.SizeType]::Absolute, 40)))
[void]$tableLayoutPanel.RowStyles.Add((New-Object System.Windows.Forms.RowStyle([System.Windows.Forms.SizeType]::Absolute, 40)))

# --- Create UI Elements and add to Layout ---
$textBox = New-Object System.Windows.Forms.TextBox
$textBox.Multiline = $true; $textBox.ScrollBars = "Vertical"; $textBox.Dock = "Fill"
$tableLayoutPanel.SetColumnSpan($textBox, 4)
$tableLayoutPanel.Controls.Add($textBox, 0, 0)
$labelVoice = New-Object System.Windows.Forms.Label
$labelVoice.Text = "Voice:"; $labelVoice.Dock = "Fill"; $labelVoice.TextAlign = [System.Drawing.ContentAlignment]::MiddleLeft
$tableLayoutPanel.Controls.Add($labelVoice, 0, 1)
$comboBoxVoice = New-Object System.Windows.Forms.ComboBox
$comboBoxVoice.Dock = "Fill"; $comboBoxVoice.DropDownStyle = "DropDownList"
foreach ($voice in $voices) { [void]$comboBoxVoice.Items.Add($voice.VoiceInfo.Name) }
$comboBoxVoice.SelectedItem = $speaker.Voice.Name
$comboBoxVoice.Add_SelectedIndexChanged({ SafelyChangeVoice($comboBoxVoice.SelectedItem) })
$tableLayoutPanel.SetColumnSpan($comboBoxVoice, 2)
$tableLayoutPanel.Controls.Add($comboBoxVoice, 1, 1)
$buttonSwitchVoice = New-Object System.Windows.Forms.Button
$buttonSwitchVoice.Dock = "Fill"; $buttonSwitchVoice.Text = "Switch"
$buttonSwitchVoice.Add_Click({
    $currentName = $comboBoxVoice.SelectedItem
    for ($i = 0; $i -lt $comboBoxVoice.Items.Count; $i++) {
        if ($comboBoxVoice.Items[$i] -eq $currentName) {
            $nextIndex = ($i + 1) % $comboBoxVoice.Items.Count
            $comboBoxVoice.SelectedItem = $comboBoxVoice.Items[$nextIndex]
            break
        }
    }
})
$tableLayoutPanel.Controls.Add($buttonSwitchVoice, 3, 1)
$labelRate = New-Object System.Windows.Forms.Label
$labelRate.Text = "Speed:"; $labelRate.Dock = "Fill"; $labelRate.TextAlign = [System.Drawing.ContentAlignment]::MiddleLeft
$tableLayoutPanel.Controls.Add($labelRate, 0, 2)
$numericRate = New-Object System.Windows.Forms.NumericUpDown
$numericRate.Dock = "Fill"; $numericRate.Minimum = -5; $numericRate.Maximum = 5
$numericRate.Increment = 0.1; $numericRate.DecimalPlaces = 1; $numericRate.Value = 1.0
$numericRate.Add_ValueChanged({ RestartReadingIfTextPresent })
$tableLayoutPanel.Controls.Add($numericRate, 1, 2)
$labelVolume = New-Object System.Windows.Forms.Label
$labelVolume.Text = "Vol:"; $labelVolume.Dock = "Fill"; $labelVolume.TextAlign = [System.Drawing.ContentAlignment]::MiddleRight
$tableLayoutPanel.Controls.Add($labelVolume, 2, 2)
$trackBarVolume = New-Object System.Windows.Forms.TrackBar
$trackBarVolume.Dock = "Fill"; $trackBarVolume.Minimum = 0; $trackBarVolume.Maximum = 100; $trackBarVolume.Value = 100; $trackBarVolume.TickFrequency = 20
$trackBarVolume.Add_Scroll({ $speaker.Volume = $trackBarVolume.Value; RestartReadingIfTextPresent })
$tableLayoutPanel.Controls.Add($trackBarVolume, 3, 2)
$labelPitch = New-Object System.Windows.Forms.Label
$labelPitch.Text = "Pitch:"; $labelPitch.Dock = "Fill"; $labelPitch.TextAlign = [System.Drawing.ContentAlignment]::MiddleLeft
$tableLayoutPanel.Controls.Add($labelPitch, 0, 3)
$comboBoxPitch = New-Object System.Windows.Forms.ComboBox
$comboBoxPitch.Dock = "Fill"; $comboBoxPitch.DropDownStyle = "DropDownList"
$pitchOptions = @("default", "x-low", "low", "medium", "high", "x-high")
foreach ($pitch in $pitchOptions) { [void]$comboBoxPitch.Items.Add($pitch) }
$comboBoxPitch.SelectedItem = "medium"
$comboBoxPitch.Add_SelectedIndexChanged({ RestartReadingIfTextPresent })
$tableLayoutPanel.Controls.Add($comboBoxPitch, 1, 3)
$labelInterval = New-Object System.Windows.Forms.Label
$labelInterval.Text = "Interval (ms):"
$labelInterval.Dock = "Fill"; $labelInterval.TextAlign = [System.Drawing.ContentAlignment]::MiddleRight
$tableLayoutPanel.Controls.Add($labelInterval, 2, 3)
$numericInterval = New-Object System.Windows.Forms.NumericUpDown
$numericInterval.Dock = "Fill"
$numericInterval.Minimum = 0; $numericInterval.Maximum = 5000; $numericInterval.Value = 1000; $numericInterval.Increment = 100
$numericInterval.Add_ValueChanged({ $timer.Interval = $numericInterval.Value; RestartReadingIfTextPresent })
$tableLayoutPanel.Controls.Add($numericInterval, 3, 3)


# --- Create Buttons (Row 4: Play/Paste/Stop Speaker) ---
$buttonPaste = New-Object System.Windows.Forms.Button
$buttonPaste.Dock = "Fill"; $buttonPaste.Text = "Paste Clipboard"
# FIX: Use a robust method to get clipboard text in the UI thread
$buttonPaste.Add_Click({
    if ([System.Windows.Forms.Clipboard]::ContainsText()) {
        $text = [System.Windows.Forms.Clipboard]::GetText()
        $script:lastClipboardText = $text.Trim()
        $textBox.Text = $text
        RestartReadingIfTextPresent
    }
})
$tableLayoutPanel.Controls.Add($buttonPaste, 0, 4)

$buttonRead = New-Object System.Windows.Forms.Button
$buttonRead.Dock = "Fill"; $buttonRead.Text = "Read Aloud"
$buttonRead.Add_Click({ RestartReadingIfTextPresent })
$tableLayoutPanel.Controls.Add($buttonRead, 1, 4)

$buttonStopSpeaker = New-Object System.Windows.Forms.Button
$buttonStopSpeaker.Dock = "Fill"; $buttonStopSpeaker.Text = "Stop Speaker"
$buttonStopSpeaker.Add_Click({ $speaker.SpeakAsyncCancelAll() })
$tableLayoutPanel.SetColumnSpan($buttonStopSpeaker, 2)
$tableLayoutPanel.Controls.Add($buttonStopSpeaker, 2, 4)


$labelListenerStatus = New-Object System.Windows.Forms.Label
$labelListenerStatus.Text = "Listener Status:"
$labelListenerStatus.Dock = "Fill"; $labelListenerStatus.TextAlign = [System.Drawing.ContentAlignment]::MiddleLeft
$tableLayoutPanel.Controls.Add($labelListenerStatus, 0, 5)
$buttonStartListener = New-Object System.Windows.Forms.Button
$buttonStartListener.Dock = "Fill"; $buttonStartListener.Text = "Start Listener"
$buttonStartListener.Add_Click({ ToggleListener })
$tableLayoutPanel.Controls.Add($buttonStartListener, 1, 5)
$buttonStopListener = New-Object System.Windows.Forms.Button
$buttonStopListener.Dock = "Fill"; $buttonStopListener.Text = "Stop Listener"
$buttonStopListener.Add_Click({ ToggleListener })
$tableLayoutPanel.SetColumnSpan($buttonStopListener, 2)
$tableLayoutPanel.Controls.Add($buttonStopListener, 2, 5)

# --- Add a Timer for Polling the Clipboard ---
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = $numericInterval.Value
$timer.Add_Tick({ UpdateAndSpeak })
$timer.Start() | Out-Null

# --- Display the Form ---
# Must run PowerShell with the -STA switch.
[void]$form.ShowDialog()
