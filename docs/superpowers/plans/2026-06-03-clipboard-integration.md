# Clipboard Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a unified, synchronous Clipboard API in the Ray framework using the `arboard` crate.

**Architecture:** `RayEngine` owns a single `arboard::Clipboard` instance on the main thread. `RayContext` carries a mutable reference to this instance during update/render loops, exposing simple `read` and `write` methods to extensions.

**Tech Stack:** Rust, `arboard`

---

### Task 1: Dependency Setup

**Files:**
- Modify: `ray-core/Cargo.toml`

- [ ] **Step 1: Add arboard dependency to ray-core**

```toml
[dependencies]
# ... existing
arboard = { workspace = true }
```

- [ ] **Step 2: Verify build**

Run: `cargo build -p ray-core`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add ray-core/Cargo.toml
git commit -m "chore: add arboard dependency to ray-core"
```

---

### Task 2: Update RayEngine Struct

**Files:**
- Modify: `ray-core/src/lib.rs`

- [ ] **Step 1: Update RayEngine struct to include clipboard**

```rust
pub struct RayEngine {
    extensions: Vec<ExtensionEntry>,
    bus: RayEventBus,
    pub active_extension_idx: usize,
    db_path: String,
    pub hotkey_registry: HotkeyRegistry,
    pub vsync_enabled: bool,
    clipboard: Option<arboard::Clipboard>, // Add this
}
```

- [ ] **Step 2: Initialize clipboard in RayEngine::new**

```rust
        let engine = Self {
            extensions: Vec::new(),
            bus,
            active_extension_idx: 0,
            db_path: db_path.to_string(),
            hotkey_registry: HotkeyRegistry::default(),
            vsync_enabled: true,
            clipboard: arboard::Clipboard::new().ok(), // Initialize here
        };
```

- [ ] **Step 3: Verify build**

Run: `cargo build -p ray-core`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add ray-core/src/lib.rs
git commit -m "feat: add clipboard owner to RayEngine"
```

---

### Task 3: Update RayContext API

**Files:**
- Modify: `ray-api/src/lib.rs`

- [ ] **Step 1: Add clipboard field to RayContext**

```rust
pub struct RayContext<'a> {
    pub delta_time: f32,
    pub screen_size: (f32, f32),
    pub bus: &'a RayEventBus,
    pub applet_name: String,
    pub is_active: bool,
    pub clipboard: Option<&'a mut arboard::Clipboard>, // Add this
}
```

- [ ] **Step 2: Add helper methods to RayContext**

```rust
impl<'a> RayContext<'a> {
    pub fn clipboard_read(&mut self) -> Option<String> {
        self.clipboard.as_mut().and_then(|cb| cb.get_text().ok())
    }

    pub fn clipboard_write(&mut self, text: String) {
        if let Some(cb) = self.clipboard.as_mut() {
            if let Err(e) = cb.set_text(text) {
                tracing::error!("Clipboard write error: {}", e);
            }
        }
    }
    // ...
}
```

- [ ] **Step 3: Verify build**

Run: `cargo build -p ray-api`
Expected: PASS (Note: ray-core will fail until next task)

- [ ] **Step 4: Commit**

```bash
git add ray-api/src/lib.rs
git commit -m "feat: add clipboard methods to RayContext"
```

---

### Task 4: Propagate Clipboard Reference

**Files:**
- Modify: `ray-core/src/lib.rs`

- [ ] **Step 1: Update RayEngine::update to pass clipboard**

```rust
    pub fn update(&mut self, dt: f32) -> Result<()> {
        // ... inside loop ...
        let clipboard = self.clipboard.as_mut();
        
        // Update on_event dispatch
        let mut ctx = RayContext {
            delta_time: dt,
            screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
            bus: &self.bus,
            applet_name: entry.instance.name().to_string(),
            is_active,
            clipboard: clipboard.as_deref_mut(), // Note: need appropriate lifetimes/borrowing
        };
        // ...
    }
```
*Correction: Since we use `clipboard` multiple times in the loops, we need to be careful with borrowing. We'll pass `self.clipboard.as_mut()` into the context constructors.*

- [ ] **Step 2: Update render and render_extension_settings to pass clipboard**

(Similar to Step 1, updating all `RayContext` instantiations)

- [ ] **Step 3: Verify build and fix borrow issues**

Run: `cargo build`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add ray-core/src/lib.rs
git commit -m "feat: propagate clipboard reference through RayContext"
```

---

### Task 5: Verification in Audio Applet (Example)

**Files:**
- Modify: `ray-applets/audio/src/lib.rs`

- [ ] **Step 1: Add a test button to Audio settings to write to clipboard**

```rust
    fn settings_ui(&mut self, ctx: &mut RayContext, ui: &mut macroquad::ui::Ui) -> anyhow::Result<()> {
        // ...
        if ui.button(None, "Copy Device List") {
            ctx.clipboard_write(self.device_list.clone());
        }
        // ...
    }
```

- [ ] **Step 2: Verify functionality**

1. Run the app.
2. Go to Audio -> Settings.
3. Click "Copy Device List".
4. Paste into another app to verify.

- [ ] **Step 3: Commit**

```bash
git add ray-applets/audio/src/lib.rs
git commit -m "test: verify clipboard write in audio applet"
```
