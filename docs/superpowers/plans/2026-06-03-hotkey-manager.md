# Global Hotkey Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a centralized hotkey management system supporting Framework-Global, OS-Global, and Applet-Local scopes with persistence and conflict detection.

**Architecture:** A registry in `ray-core` stores definitions, while the `ray-runner-macroquad` performs per-frame polling of both Macroquad and OS-level keyboard events. Events are dispatched back to applets via the `RayEventBus`.

**Tech Stack:** Rust, Macroquad (Local/Global), global-hotkey (OS-Global), SQLite (Persistence).

---

### Task 1: API Definitions (`ray-api`)

**Files:**
- Modify: `ray-api/src/lib.rs`

- [ ] **Step 1: Define Hotkey types**
Add `HotkeyScope`, `HotkeyModifiers`, and `HotkeyDefinition` to `lib.rs`.

```rust
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum HotkeyScope {
    Global,
    OS,
    Local,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    pub struct HotkeyModifiers: u32 {
        const NONE = 0;
        const SHIFT = 1 << 0;
        const CTRL = 1 << 1;
        const ALT = 1 << 2;
        const LOGO = 1 << 3;
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HotkeyDefinition {
    pub id: String,
    pub key: String, // Use string representation for cross-crate compatibility (e.g. "Space")
    pub modifiers: HotkeyModifiers,
    pub scope: HotkeyScope,
    pub description: String,
}
```

- [ ] **Step 2: Update `RayEvent` and `RayContext`**
Add `HotkeyTriggered(String)` to `RayEvent`.

- [ ] **Step 3: Commit**
```bash
git add ray-api/src/lib.rs
git commit -m "api: add hotkey definitions and events"
```

---

### Task 2: Core Registry & Persistence (`ray-core`)

**Files:**
- Modify: `ray-core/src/lib.rs`

- [ ] **Step 1: Create `HotkeyRegistry` struct**
Implement a registry that stores `HotkeyDefinition`s and tracks conflicts.

- [ ] **Step 2: Add SQLite Migration**
Update `ensure_db_schema` to create `hotkey_overrides` table.
```sql
CREATE TABLE IF NOT EXISTS hotkey_overrides (
    applet_name TEXT,
    hotkey_id TEXT,
    key_code TEXT,
    modifiers INTEGER,
    PRIMARY KEY (applet_name, hotkey_id)
)
```

- [ ] **Step 3: Implement Registration Logic**
Add `register_hotkey` to `RayEngine`. This should load overrides from the DB if they exist.

- [ ] **Step 4: Commit**
```bash
git add ray-core/src/lib.rs
git commit -m "core: implement hotkey registry and persistence"
```

---

### Task 3: OS-Global Integration (`ray-runner-macroquad`)

**Files:**
- Modify: `ray-runner-macroquad/Cargo.toml`
- Modify: `ray-runner-macroquad/src/main.rs`

- [ ] **Step 1: Add dependencies**
Add `global-hotkey` and platform-specific loop helpers.

- [ ] **Step 2: Initialize Manager**
In `main.rs`, initialize `GlobalHotKeyManager`. Note: On macOS, this must be main-thread.

- [ ] **Step 3: Poll OS Events**
In the main loop, poll the `GlobalHotKeyEvent::receiver()` and map OS IDs back to Framework Hotkey IDs.

- [ ] **Step 4: Commit**
```bash
git add ray-runner-macroquad/Cargo.toml ray-runner-macroquad/src/main.rs
git commit -m "runner: integrate global-hotkey for OS-scope"
```

---

### Task 4: Framework & Local Polling Logic

**Files:**
- Modify: `ray-core/src/lib.rs`
- Modify: `ray-runner-macroquad/src/main.rs`

- [ ] **Step 1: Implement Macroquad Polling**
In `engine.update()`, check all registered `Global` and `Local` hotkeys using `macroquad::input::is_key_pressed`.

- [ ] **Step 2: Scope Validation**
Ensure `Local` hotkeys only trigger if `active_extension_idx` matches the applet that registered it.

- [ ] **Step 3: Dispatch Events**
Send `RayEvent::HotkeyTriggered(id)` through the bus.

- [ ] **Step 4: Commit**
```bash
git commit -m "feat: implement hotkey polling and scope validation"
```

---

### Task 5: Settings UI

**Files:**
- Modify: `ray-runner-macroquad/src/main.rs`

- [ ] **Step 1: Add Hotkeys Tab**
Create a new section in the settings window to list all hotkeys.

- [ ] **Step 2: Display Conflicts**
Highlight hotkeys that share the same key combination in red.

- [ ] **Step 3: Commit**
```bash
git commit -m "ui: add hotkeys management to settings"
```

---

### Task 6: Example Integration (Audio Applet)

**Files:**
- Modify: `ray-applets/audio/src/lib.rs`

- [ ] **Step 1: Register "Toggle Recording" Hotkey**
In `AudioApplet::init`, register a `Global` hotkey (e.g., `Ctrl+R`).

- [ ] **Step 2: Handle Event**
In `on_event`, listen for `HotkeyTriggered("toggle_recording")` and execute the command.

- [ ] **Step 3: Commit**
```bash
git commit -m "applet: use hotkey system in audio applet"
```
