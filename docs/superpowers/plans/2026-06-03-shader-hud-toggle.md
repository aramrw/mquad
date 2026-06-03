# Shader IDE HUD Toggle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a local hotkey 'H' to toggle the visibility of the entire Shader IDE HUD (windows and controls).

**Architecture:** Add a `hud_visible` flag to the applet state. Register a `Local` hotkey in `init`. Toggle the flag on event and conditionally render UI elements based on it.

**Tech Stack:** Rust, Macroquad, Ray API.

---

### Task 1: State and Registration

**Files:**
- Modify: `ray-applets/shaders/src/lib.rs`

- [ ] **Step 1: Add `hud_visible` to `ShaderApplet`**
Add the field to the struct and initialize it to `true` in `new()`.

- [ ] **Step 2: Register the hotkey in `init`**
Call `ctx.register_hotkey` with `HotkeyDefinition` for 'H' (Local scope).

```rust
ctx.register_hotkey(HotkeyDefinition {
    id: "toggle_hud".to_string(),
    key: "H".to_string(),
    modifiers: HotkeyModifiers::NONE,
    scope: HotkeyScope::Local,
    description: "Toggle Shader IDE HUD".to_string(),
});
```

- [ ] **Step 3: Commit**
```bash
git add ray-applets/shaders/src/lib.rs
git commit -m "shader: add hud_visible state and register hotkey"
```

---

### Task 2: Event Handling

**Files:**
- Modify: `ray-applets/shaders/src/lib.rs`

- [ ] **Step 1: Update `on_event`**
Handle `RayEvent::HotkeyTriggered("toggle_hud")`.

```rust
if let RayEvent::HotkeyTriggered(id) = event {
    if id == "toggle_hud" {
        self.hud_visible = !self.hud_visible;
    }
}
```

- [ ] **Step 2: Commit**
```bash
git commit -m "shader: handle toggle_hud event"
```

---

### Task 3: Conditional Rendering

**Files:**
- Modify: `ray-applets/shaders/src/lib.rs`

- [ ] **Step 1: Wrap HUD logic in `render`**
Wrap the section starting from `// 2. Draw the HUDs` until the end of the `render` function in an `if self.hud_visible` block.

- [ ] **Step 2: Commit**
```bash
git commit -m "shader: implement conditional HUD rendering"
```

---

### Task 4: Verification

- [ ] **Step 1: Verify compilation**
Run `cargo check -p ray-applet-shaders`.

- [ ] **Step 2: Manual Verification (Instructions)**
1. Run Ray.
2. Select Shader IDE tab.
3. Press 'H'. All UI elements (windows, checkboxes) should disappear.
4. Press 'H' again. They should reappear in their previous states.
5. Go to Settings -> Hotkeys. Verify 'Toggle Shader IDE HUD' is listed under Shader IDE as a Local hotkey.
