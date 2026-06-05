# Design: Buffer-Muxed Audio Capture for Capture Extension

## Purpose
Enable the Capture extension to record video with audio by consuming the live audio buffers produced by the Audio extension. This ensures perfect synchronization with system-wide audio (used by shaders, etc.) while keeping the Audio extension as a decoupled, standalone source.

## Architecture

### 1. Integration with Audio Extension
- **Activation**: When a video recording starts in the Capture extension (and "Include Audio" is toggled on), the extension will send a `RayCommand::Audio(AudioCommand::StartRecording)` to ensure the system-wide audio feed is active.
- **Consumption**: The Capture extension will listen for `AudioEvent::Buffer(Vec<f32>)` on the `RayEventBus`.

### 2. Capture Extension Updates (`ray-applet-capture`)
- **State**:
    - `audio_enabled: bool` (persisted)
    - `audio_pipe: Option<std::process::ChildStdin>`: Handle to the `ffmpeg` stdin for audio samples.
- **Recording Logic**:
    - **Start**:
        - Construct an `ffmpeg` command that accepts a video input (hardware grab) and an audio input via stdin (`-f f32le -ar 44100 -ac 1 -i -`).
        - Spawn the process with `stdin(Stdio::piped())`.
        - If `audio_enabled`, send `RayCommand::Audio(AudioCommand::StartRecording)`.
    - **Loop**:
        - On `RayEvent::Audio(AudioEvent::Buffer(samples))`, if a recording is active and `audio_enabled`, convert the `f32` samples to `le_bytes` and write them to the `ffmpeg` stdin.
    - **Stop**:
        - Close the stdin pipe (this signals EOF to ffmpeg for the audio stream).
        - Kill the `ffmpeg` process (or wait for it to finish encoding).
- **Settings UI**:
    - Persisted checkbox for "Include Audio".

### 3. Synchronization & Persistence
- **Device Index**: The Capture extension does *not* need to care about the device index anymore, as the Audio extension handles the hardware interface and produces the buffers.
- **Persistence**: The `audio_enabled` toggle will be saved to `applet_configs` in `framework_settings.db`.

## FFmpeg Command Detail
```bash
ffmpeg -f avfoundation -framerate 30 -i "1:none" \
       -f f32le -ar 44100 -ac 1 -i - \
       -map 0:v -map 1:a \
       -c:v libx264 -crf 23 -pix_fmt yuv420p \
       -c:a aac -shortest \
       output.mp4
```

## Success Criteria
1. Video recorded by the Capture extension contains audio synced with the live feed.
2. The Audio extension is automatically activated when recording starts.
3. No modifications are made to the internal logic of the Audio extension.
