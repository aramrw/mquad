# Buffer-Muxed Audio Capture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement video recording with audio by piping live buffers from the Audio extension into the Capture extension's FFmpeg process.

**Architecture:** The Capture extension will open an FFmpeg process with an audio stdin pipe. It will listen for `AudioEvent::Buffer` events and write the samples to that pipe. It will also trigger the Audio extension to start if needed.

**Tech Stack:** Rust, FFmpeg, Stdio piping.

---

### Task 1: Update CaptureApplet State for Audio Piping

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Update `CaptureApplet` struct**
Add `audio_stdin: Option<std::io::BufWriter<std::process::ChildStdin>>` and remove `audio_device_index`.

```rust
pub struct CaptureApplet {
    // ... existing fields ...
    audio_enabled: bool,
    // Removed: audio_device_index
    audio_stdin: Option<std::io::BufWriter<std::process::ChildStdin>>,
}
```

- [ ] **Step 2: Update `CaptureApplet::new`**
Initialize `audio_stdin` as `None`.

```rust
impl CaptureApplet {
    pub fn new() -> Self {
        Self {
            // ...
            audio_enabled: false,
            audio_stdin: None,
        }
    }
}
```

- [ ] **Step 3: Commit changes**
```bash
git add ray-applets/capture/src/lib.rs
git commit -m "feat: add audio_stdin to CaptureApplet state"
```

### Task 2: Implement Multi-Input FFmpeg Command

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Update `start_recording` to use stdin for audio**

```rust
fn start_recording(&mut self, ctx: &mut RayContext) -> Result<()> {
    // ... (selection logic) ...
    if w > 10 && h > 10 {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let path = std::path::Path::new(&self.save_dir).join(format!("recording_{}.mp4", timestamp));
        let filename = path.to_str().unwrap().to_string();
        
        #[cfg(target_os = "macos")]
        let (v_format, v_device) = ("avfoundation", "1:none");
        // ... other platforms ...

        let mut cmd = std::process::Command::new("ffmpeg");
        
        // Input 0: Video
        cmd.args(&["-f", v_format, "-framerate", &self.fps.to_string(), "-i", v_device]);
        
        if self.audio_enabled {
            // Input 1: Audio from Stdin
            cmd.args(&["-f", "f32le", "-ar", "44100", "-ac", "1", "-i", "-"]);
            cmd.stdin(std::process::Stdio::piped());
            
            // Trigger Audio extension
            ctx.bus.send(RayEvent::Command(RayCommand::Audio(ray_api::AudioCommand::StartRecording)));
        }

        let crop_filter = format!("crop={}:{}:{}:{}", w, h, x, y);
        cmd.args(&["-vf", &crop_filter, "-c:v", "libx264", "-crf", &self.crf.to_string(), "-pix_fmt", "yuv420p"]);
        
        if self.audio_enabled {
            cmd.args(&["-c:a", "aac", "-shortest"]);
            // Map inputs: 0:v for video, 1:a for audio
            cmd.args(&["-map", "0:v", "-map", "1:a"]);
        }

        cmd.arg(&filename);

        let mut child = cmd.spawn()?;
        if self.audio_enabled {
            if let Some(stdin) = child.stdin.take() {
                self.audio_stdin = Some(std::io::BufWriter::new(stdin));
            }
        }
        self.recording_process = Some(child);
        // ...
    }
    Ok(())
}
```

- [ ] **Step 2: Update `stop_recording` to close stdin**

```rust
fn stop_recording(&mut self, ctx: &mut RayContext) -> Result<()> {
    self.audio_stdin = None; // Dropping BufWriter closes the pipe
    if let Some(mut child) = self.recording_process.take() {
        let _ = child.kill();
        // ...
    }
    Ok(())
}
```

- [ ] **Step 3: Commit changes**
```bash
git add ray-applets/capture/src/lib.rs
git commit -m "feat: implement dual-input ffmpeg command in capture"
```

### Task 3: Implement Audio Buffer Muxing

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Update `on_event` to pipe buffers**

```rust
fn on_event(&mut self, ctx: &mut RayContext, event: &RayEvent) -> Result<()> {
    match event {
        // ... existing hotkey cases ...
        RayEvent::Audio(AudioEvent::Buffer(samples)) => {
            if let Some(writer) = &mut self.audio_stdin {
                use std::io::Write;
                for &sample in samples {
                    let bytes = sample.to_le_bytes();
                    if writer.write_all(&bytes).is_err() {
                        // Pipe might have closed if ffmpeg crashed
                        break;
                    }
                }
                let _ = writer.flush();
            }
        }
        // Removed: AudioEvent::DeviceSelected
        _ => {}
    }
    Ok(())
}
```

- [ ] **Step 2: Commit changes**
```bash
git add ray-applets/capture/src/lib.rs
git commit -m "feat: pipe audio buffers to ffmpeg stdin"
```

### Task 4: Cleanup Persistence and UI

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Refactor `save_settings` and `load_settings`**
Remove references to `audio_device_index`.

- [ ] **Step 2: Update `settings_ui`**
Remove the device index label.

- [ ] **Step 3: Commit changes**
```bash
git add ray-applets/capture/src/lib.rs
git commit -m "chore: clean up capture persistence and UI"
```
