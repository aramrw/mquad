# Shader IDE HUD Toggle Design

**Date:** 2026-06-03
**Status:** Draft
**Topic:** Implementing a local hotkey to toggle all Shader IDE UI elements.

## 1. Purpose
Allow the user to quickly hide/show the entire Shader IDE interface (Controls window, Library window, and Top-level checkboxes) to focus on the shader output.

## 2. Architecture

### 2.1 State Management
- Add `hud_visible: bool` to `ShaderApplet`.
- Default value: `true`.

### 2.2 Hotkey Registration
- **Key**: `H`
- **Modifiers**: `NONE`
- **Scope**: `Local`
- **ID**: `toggle_hud`
- **Description**: `Toggle Shader IDE HUD`

### 2.3 Event Handling
When a `HotkeyTriggered("toggle_hud")` event is received:
- `self.hud_visible = !self.hud_visible`

### 2.4 Rendering Logic
Wrap the entire UI section in `ShaderApplet::render` (checkboxes and windows) with:
```rust
if self.hud_visible {
    // Checkboxes
    // Controls Window
    // Library Window
}
```

## 3. Benefits
- **Non-Destructive**: Hiding the HUD does not change the user's preferred layout (`show_controls` and `show_library` flags remain untouched).
- **Clean Output**: Toggling off provides a completely unobstructed view of the shader.
- **Context Awareness**: Being a `Local` hotkey, it won't interfere with other applets (e.g., if you are in the Audio tab, 'H' won't trigger the shader HUD toggle).

## 4. Implementation Steps
1. Update `ShaderApplet` struct.
2. Register hotkey in `init`.
3. Handle event in `on_event`.
4. Update `render` with conditional check.
