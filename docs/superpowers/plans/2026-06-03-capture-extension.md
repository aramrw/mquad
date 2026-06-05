# Capture Extension Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a cross-platform ShareX-like capture extension with region selection, screenshots, and video recording.

**Architecture:** 
1.  **Overlay Mode:** Framework-level support for borderless, always-on-top, full-screen windows.
2.  **Freeze-Frame Selection:** Using `xcap` for instant snapshots displayed in Overlay Mode for region selection.
3.  **Handoff:** Crop snapshots for screenshots; spawn background `ffmpeg` processes for video/audio.
4.  **Indicator:** A mini-window mode for showing recording status.

**Tech Stack:** Rust, `macroquad`, `xcap`, `ffmpeg` (external), `arboard` (existing).

---

### Task 1: Framework Overlay Mode & Dependencies

**Files:**
- Modify: `Cargo.toml` (root)
- Modify: `ray-core/Cargo.toml`
- Modify: `ray-core/src/lib.rs`
- Modify: `ray-runner-macroquad/src/main.rs`

- [ ] **Step 1: Add xcap dependency**
Add `xcap = "0.9"` to root `Cargo.toml` workspace dependencies and `ray-core/Cargo.toml`.

- [ ] **Step 2: Add Overlay state to RayEngine**
Update `RayEngine` to track `overlay_active: bool` and provide a method `toggle_overlay(bool)`.

- [ ] **Step 3: Implement Platform-Specific Overlay (Runner)**
In `ray-runner-macroquad/src/main.rs`, implement the window state change using platform APIs (e.g., `winapi` for Windows, `core-foundation`/`objc` for macOS) to set the window to borderless, full-screen, and always-on-top when `overlay_active` is true.

- [ ] **Step 4: Commit**
`git commit -m "feat: add framework support for Overlay Mode"`

---

### Task 2: Capture Applet & Selection UI

**Files:**
- Create: `ray-applets/capture/Cargo.toml`
- Create: `ray-applets/capture/src/lib.rs`
- Modify: `ray-runner-macroquad/src/main.rs` (register applet)

- [ ] **Step 1: Scaffold CaptureApplet**
Implement `RayExtension` trait. Store `snapshot: Option<Texture2D>` and selection coordinates.

- [ ] **Step 2: Implement Freeze-Frame Snapshot**
In `on_event` or a specific method, use `xcap` to capture all monitors, convert to an image, and load as a `macroquad::Texture2D`.

- [ ] **Step 3: Implement Selection Logic (Update/Render)**
When in selection mode:
- Draw the snapshot texture.
- Handle mouse drag to update selection rect.
- Draw dimmed overlay around the selection.

- [ ] **Step 4: Commit**
`git commit -m "feat: implement freeze-frame region selection UI"`

---

### Task 3: Screenshot Action (Cropping & Saving)

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Implement Screenshot Cropping**
When selection is released, crop the high-res snapshot using selection coordinates.

- [ ] **Step 2: Save and Clipboard Integration**
Save the cropped image to a timestamped file. Use `ctx.clipboard_write` (using existing API) to copy the image to the clipboard (if supported by arboard/our API).

- [ ] **Step 3: Commit**
`git commit -m "feat: implement screenshot cropping and saving"`

---

### Task 4: Video Recording (FFMPEG Integration)

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Implement OS-specific FFMPEG Command Mapping**
Map selection rect `(x, y, w, h)` to `ffmpeg` crop parameters for `avfoundation` (macOS), `gdigrab` (Windows), and `x11grab` (Linux).

- [ ] **Step 2: Spawn Recording Process**
Use `std::process::Command` to run `ffmpeg` in the background.

- [ ] **Step 3: Commit**
`git commit -m "feat: implement ffmpeg-based video recording"`

---

### Task 5: Recording Indicator & Settings

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`
- Modify: `ray-core/src/lib.rs` (Mini Mode support)

- [ ] **Step 1: Implement "Mini Mode" in Framework**
Add support for shrinking the Ray window to a small corner tab during recording.

- [ ] **Step 2: Capture Applet Settings UI**
Add sliders for video quality (CRF), framerate, and directory path selectors using `rfd`.

- [ ] **Step 3: Final Verification**
Test all hotkey combinations (Region Screenshot, Full Screenshot, Region Video).

- [ ] **Step 4: Commit**
`git commit -m "feat: add recording indicator and capture settings"`
