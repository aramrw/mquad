# Node-Based Extension Chaining & Payload System

## Overview
Transform `ray` from a static dashboard of independent extensions into a visual, node-based pipeline engine. Extensions will act as "Nodes" that can consume, mutate, and broadcast generic payloads (like video files, audio buffers, or text) to other extensions. This will allow for complex automated workflows (e.g., Image Cropper -> Image Trimmer -> Uploader).

## 1. The Payload Event Architecture
To enable communication without tightly coupling extensions, a new `RayEvent` variant will be introduced to the `ray-api`.

```rust
pub enum RayEvent {
    // ... existing events
    Payload(PayloadMessage),
}

pub struct PayloadMessage {
    pub source: String,       // The ID of the extension that emitted this
    pub target: String,       // The ID of the extension meant to receive it
    pub data: PayloadData,
}

pub enum PayloadData {
    File(std::path::PathBuf), // For large media (videos, images, audio) handled by ffmpeg
    Buffer(Vec<u8>),          // For in-memory binary data
    Text(String),             // For clipboard/dictionary integrations
}
```

Extensions will inspect `RayEvent::Payload`. If the `target` matches their ID, they will process the `data`, perform their task, and emit a new `Payload` targeting the next node in the chain.

## 2. Multi-Modal Visual Node UI (Macroquad)
Instead of forcing a single interaction paradigm, the UI will leverage Macroquad's immediate-mode rendering and GPU power to offer a flexible, fun, and visually rich "Node View".

The user can choose their preferred wiring interaction mode (which can be toggled via a UI button or hotkey):

* **Mode 1: Snap-Together Blocks (LEGO):** Dragging a node close to another node's input port will physically snap them together.
* **Mode 2: Auto-Wiring (Magnets):** Moving nodes near each other automatically draws a glowing, shader-based link between them.
* **Mode 3: Click-to-Link (Quick-Draw):** Clicking an output port, then an input port, instantly draws the connection.

Visuals will heavily utilize shaders (like the audio shaders) to show data flowing through the pipes when a payload is active.

## 3. Initial Implementation Targets
To prove the architecture, two new generic node extensions will be built:

1. **Cropper Node:** 
   - Takes a `PayloadData::File`.
   - UI provides Width/Height inputs.
   - Runs `ffmpeg` in the background to crop the video/image.
   - Emits a new `PayloadData::File` pointing to the cropped temp file.
2. **Trimmer Node:**
   - Takes a `PayloadData::File`.
   - UI provides Start Time / Duration.
   - Runs `ffmpeg` to trim the video/image.
   - Emits the final `PayloadData::File`.

These will demonstrate that complex video editing workflows can be built dynamically just by chaining independent extensions together in the Node UI.