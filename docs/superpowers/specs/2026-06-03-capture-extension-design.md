# Capture Extension Design (ShareX-like)

A cross-platform media capture extension for taking screenshots and recording video/audio using a "freeze-frame" selection UI.

## 1. Overview
The Capture Extension allows users to capture their screen via hotkeys. It uses a temporary full-screen overlay to allow precise region selection. Once a region is selected, the extension can either save a cropped screenshot or start a background video recording using `ffmpeg`.

## 2. Components

### 2.1 Core Primitive: Overlay Mode
The framework (`ray-core` and `ray-runner-macroquad`) will be updated to support a "Global Overlay" state.
- **Functionality:** Toggles the application between standard windowed mode and a borderless, always-on-top, full-desktop window.
- **Implementation:** Platform-specific windowing calls (e.g., `setLevel:NSFloatingWindowLevel` on macOS) to ensure the overlay stays above all other apps.

### 2.2 The Capture Applet
A tabless extension that manages the capture lifecycle.
- **State Machine:** `Idle` -> `Selecting` (Overlay Active) -> `Processing` (Screenshot) / `Recording` (Video).
- **Dependencies:**
    - `xcap`: For taking the initial full-screen "freeze frame".
    - `ffmpeg`: For background video/audio encoding.
    - `arboard`: For copying screenshots to the clipboard.

## 3. Data Flow & Logic

### 3.1 Region Selection (The "Freeze Frame")
1. **Trigger:** User presses `Capture Region` hotkey.
2. **Snapshot:** Applet calls `xcap` to get images of all monitors.
3. **Overlay Start:** Engine enters "Overlay Mode". The applet renders the captured images across the entire screen.
4. **Interaction:** User clicks and drags. The applet draws a selection rectangle and dims the unselected areas of the screen.
5. **Release:** Mouse release triggers the next phase and exits "Overlay Mode".

### 3.2 Action: Screenshot
1. Crop the original high-resolution snapshot to the selected coordinates.
2. Save to the configured "Screenshots" directory (auto-named with timestamp).
3. If "Copy to Clipboard" is enabled, push the image data to the OS clipboard.

### 3.3 Action: Video Recording
1. Calculate `ffmpeg` crop parameters based on selection.
2. Spawn a background `ffmpeg` process using platform-specific input drivers:
    - **macOS:** `avfoundation`
    - **Windows:** `gdigrab`
    - **Linux:** `x11grab`
3. **Recording Indicator:** Ray enters a "Mini Mode" window (e.g., 120x40) pinned to the corner, showing "● REC" and elapsed time.
4. **Stop:** User presses hotkey or clicks the stop button in the mini window.

## 4. Configuration & Settings

The extension will provide a settings UI in the Framework settings:
- **Paths:** Separate default directories for Screenshots, Videos, and Audio.
- **Screenshot Options:** Toggle "Copy to Clipboard", Toggle "Open folder after capture".
- **Video Options:** Quality slider (CRF), Framerate (30/60).
- **Hotkeys:**
    - Capture Region (Screenshot)
    - Capture Full Screen (Screenshot)
    - Record Region (Video)
    - Stop Recording

## 5. Implementation Roadmap
1. **Task 1:** Add `xcap` and update `RayEngine` for Overlay Mode.
2. **Task 2:** Implement selection UI logic in `CaptureApplet`.
3. **Task 3:** Implement screenshot cropping and saving/clipboard logic.
4. **Task 4:** Implement `ffmpeg` video capture logic and OS-specific command mapping.
5. **Task 5:** Implement the Recording Indicator mini-window.
