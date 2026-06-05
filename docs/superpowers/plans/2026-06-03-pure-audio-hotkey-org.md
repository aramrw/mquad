# Pure Audio Capture & Hotkey Reorganization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add standalone audio recording (MP3/OGG/WAV) to the Capture extension and reorganize the hotkey settings UI by extension and scope.

**Architecture:** 
- **Audio Capture**: Capture extension handles a second FFmpeg process for audio-only recording, piping buffers from the event bus.
- **Hotkey UI**: `RayEngine` hotkey registry will be used to group hotkeys by their associated applet name and sort them by `HotkeyScope`.

**Tech Stack:** Rust, FFmpeg, Macroquad UI.

---

### Task 1: Add Standalone Audio State and Hotkey to Capture

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Define `AudioFormat` enum and update `CaptureApplet` struct**
Add `standalone_audio_format` and `audio_only_process`.

```rust
#[derive(PartialEq, Clone, Copy, Debug)]
enum AudioFormat {
    Mp3,
    Ogg,
    Wav,
}

impl AudioFormat {
    fn ext(&self) -> &str {
        match self {
            Self::Mp3 => "mp3",
            Self::Ogg => "ogg",
            Self::Wav => "wav",
        }
    }
}

pub struct CaptureApplet {
    // ...
    standalone_audio_format: AudioFormat,
    audio_only_process: Option<std::process::Child>,
    // ...
}
```

- [ ] **Step 2: Update `new()` and register `capture_pure_audio` hotkey in `init()`**
Hotkey: `Ctrl+Shift+A`.

- [ ] **Step 3: Commit changes**
```bash
git add ray-applets/capture/src/lib.rs
git commit -m "feat: add standalone audio state and hotkey registration"
```

### Task 2: Implement Standalone Audio Recording Logic

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Implement `start_audio_recording`**
Spawn FFmpeg with the selected format's encoder.

```rust
fn start_audio_recording(&mut self, ctx: &mut RayContext) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("audio_{}.{}", timestamp, self.standalone_audio_format.ext());
    let path = std::path::Path::new(&self.save_dir).join(filename);

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args(&["-f", "f32le", "-ar", "44100", "-ac", "1", "-i", "-"]);
    
    match self.standalone_audio_format {
        AudioFormat::Mp3 => { cmd.args(&["-c:a", "libmp3lame"]); }
        AudioFormat::Ogg => { cmd.args(&["-c:a", "libvorbis"]); }
        AudioFormat::Wav => { /* pcm by default */ }
    }
    
    cmd.arg(path.to_str().unwrap());
    cmd.stdin(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()?;
    if let Some(stdin) = child.stdin.take() {
        self.audio_stdin = Some(std::io::BufWriter::new(stdin));
    }
    self.audio_only_process = Some(child);
    
    // Ensure audio feed is on
    ctx.bus.send(RayEvent::Command(RayCommand::Audio(ray_api::AudioCommand::StartRecording)));
    Ok(())
}
```

- [ ] **Step 2: Update `on_event` to trigger audio recording and `stop_recording` to handle both**

- [ ] **Step 3: Commit changes**
```bash
git add ray-applets/capture/src/lib.rs
git commit -m "feat: implement pure audio recording logic"
```

### Task 3: Add Format Selector to Capture UI and Persistence

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Update `save_settings` and `load_settings`**
Persist `standalone_audio_format` as a string.

- [ ] **Step 2: Update `settings_ui` with a format selector**

- [ ] **Step 3: Commit changes**
```bash
git add ray-applets/capture/src/lib.rs
git commit -m "feat: add audio format selector and persistence"
```

### Task 4: Reorganize Hotkey Settings UI

**Files:**
- Modify: `ray-runner-macroquad/src/main.rs` (specifically the settings rendering logic)

- [ ] **Step 1: Refactor hotkey listing logic**
Group hotkeys by applet name and sort by scope within each group.

```rust
// In main.rs where hotkeys are rendered
let mut groups: std::collections::HashMap<String, Vec<&HotkeyDefinition>> = std::collections::HashMap::new();
for ((applet, _), def) in &engine.hotkey_registry.registered {
    groups.entry(applet.clone()).or_default().push(def);
}

// Render groups
for (applet, mut defs) in groups {
    ui.label(None, &format!("--- {} ---", applet));
    defs.sort_by_key(|d| match d.scope {
        HotkeyScope::Global => 0,
        HotkeyScope::OS => 1,
        HotkeyScope::Local => 2,
    });
    // ... render each def ...
}
```

- [ ] **Step 2: Commit framework changes**
```bash
git add ray-runner-macroquad/src/main.rs
git commit -m "feat: reorganize hotkey settings UI by applet and scope"
```
