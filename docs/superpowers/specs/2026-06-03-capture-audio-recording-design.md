# Design: Audio Recording Integration for Capture Extension

## Purpose
Add audio recording capability to the Capture extension, synchronized with the Audio extension's device selection, and ensure all settings persist across sessions.

## Architecture

### 1. Global Event Bus Changes (`ray-api`)
To allow the Audio extension to share its selected device with other extensions (like Capture), we will add a new variant to `AudioEvent`.

```rust
pub enum AudioEvent {
    Level(f32),
    Buffer(Vec<f32>),
    Spectrum(Vec<f32>),
    DeviceSelected(i32), // NEW: Broadcasts the currently selected device index
}
```

### 2. Audio Extension Updates (`ray-applet-audio`)
- **State**: Ensure `device_index` is persisted.
- **Initialization**: Load `device_index` from the database on startup.
- **UI**: When the `device_index` slider is moved, save the new value to the database and broadcast `AudioEvent::DeviceSelected(idx)` on the event bus.
- **Synchronization**: Broadcast the current `device_index` when the applet initializes so other applets can sync up.

### 3. Capture Extension Updates (`ray-applet-capture`)
- **State**:
    - `audio_enabled: bool` (default: false)
    - `audio_device_index: i32` (default: 1)
- **Initialization**: Load `audio_enabled` and `audio_device_index` from the database.
- **Event Handling**: Listen for `AudioEvent::DeviceSelected(idx)` and update `audio_device_index`. Save the updated index to the database.
- **FFmpeg Integration**:
    - Update `start_recording` to use the selected audio device if `audio_enabled` is true.
    - MacOS `avfoundation` input will change from `"1:none"` to `"1:{audio_device_index}"`.
    - Add `-c:a aac` for audio encoding.
- **Settings UI**:
    - Add a checkbox toggle for "Include Audio".
    - Save the toggle state to the database when changed.

## Persistence Strategy
Extensions will use `rusqlite` directly (as seen in `ray-core`) or via a shared mechanism if available. Looking at `ray-core`, the `framework_settings.db` is the standard location. Extensions should ideally manage their own table or use a common key-value store in the DB.

We will ensure `extension_settings` or a new `applet_configs` table is used to store:
- `audio_enabled` (for Capture)
- `device_index` (for Audio, mirrored in Capture)

## Success Criteria
1. Video recorded with the Capture extension includes audio from the device selected in the Audio extension.
2. The "Include Audio" setting persists across application restarts.
3. The selected audio device index persists across application restarts and is shared between extensions.
