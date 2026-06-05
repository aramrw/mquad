# Design: Pure Audio Capture & Hotkey Reorganization

## Purpose
Expand the Capture extension to support standalone audio recording (MP3, OGG, WAV) and reorganize the framework's hotkey management UI for better readability.

## Architecture

### 1. Pure Audio Capture (`ray-applet-capture`)
- **State**:
    - `audio_format: AudioFormat` (Enum: MP3, OGG, WAV).
    - `is_audio_only_recording: bool`.
- **Hotkey**: Register a new `capture_pure_audio` hotkey.
- **Recording Logic**:
    - **Activation**: When triggered, send `AudioCommand::StartRecording` to the Audio extension.
    - **FFmpeg**: Spawn an audio-only process:
      `ffmpeg -f f32le -ar 44100 -ac 1 -i - -c:a [encoder] output.[ext]`
      - MP3: `-c:a libmp3lame`
      - OGG: `-c:a libvorbis`
      - WAV: No encoder needed (pcm_s16le or similar).
- **Settings UI**:
    - Add a "Standalone Audio Format" selector.

### 2. Hotkey UI Reorganization (`ray-runner-macroquad`)
- **Grouping**: Refactor the `Settings` rendering loop to iterate through extensions and collect their registered hotkeys.
- **Sorting**: For each extension's group, sort the hotkeys so that `HotkeyScope::Global` comes first, followed by `HotkeyScope::OS` and `HotkeyScope::Local`.
- **Visuals**: Add separator labels or headers for each extension in the hotkey list.

## Persistence
- `audio_format` will be persisted in `applet_configs`.

## Success Criteria
1. User can trigger pure audio recording via hotkey.
2. User can select MP3, OGG, or WAV in the settings.
3. The hotkey settings UI is organized by extension and sorted by scope.
