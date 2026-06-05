# Audio Recording Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add audio recording to the Capture extension, synced with Audio extension settings and persisted in the database.

**Architecture:** Add `AudioEvent::DeviceSelected` to broadcast selection changes. Extensions will persist settings to `framework_settings.db` and react to broadcasts to stay in sync.

**Tech Stack:** Rust, FFmpeg, SQLite (rusqlite)

---

### Task 1: Update Ray API for Device Selection Event

**Files:**
- Modify: `ray-api/src/lib.rs`

- [ ] **Step 1: Add `DeviceSelected` variant to `AudioEvent`**

```rust
pub enum AudioEvent {
    /// Current peak audio level (0.0 to 1.0).
    Level(f32),
    /// Raw PCM audio buffer.
    Buffer(Vec<f32>),
    /// Frequency spectrum data.
    Spectrum(Vec<f32>),
    /// NEW: Broadcasts the currently selected device index.
    DeviceSelected(i32),
}
```

- [ ] **Step 2: Commit API changes**

```bash
git add ray-api/src/lib.rs
git commit -m "api: add AudioEvent::DeviceSelected for synchronization"
```

### Task 2: Persistence and Broadcasting in Audio Extension

**Files:**
- Modify: `ray-applets/audio/src/lib.rs`

- [ ] **Step 1: Implement persistence helpers for AudioApplet**

```rust
// Add to AudioApplet impl
fn save_device_index(&self, idx: i32) -> anyhow::Result<()> {
    let conn = rusqlite::Connection::open("framework_settings.db")?;
    conn.execute(
        "INSERT OR REPLACE INTO applet_configs (applet, key, value) VALUES (?1, ?2, ?3)",
        rusqlite::params!["audio", "device_index", idx.to_string()],
    )?;
    Ok(())
}

fn load_device_index() -> i32 {
    if let Ok(conn) = rusqlite::Connection::open("framework_settings.db") {
        let mut stmt = conn.prepare("SELECT value FROM applet_configs WHERE applet = ?1 AND key = ?2").ok().unwrap();
        if let Ok(val) = stmt.query_row(rusqlite::params!["audio", "device_index"], |row| row.get::<_, String>(0)) {
            return val.parse().unwrap_or(1);
        }
    }
    1
}
```

- [ ] **Step 2: Update `AudioApplet::new` and `init` to load and broadcast device index**

- [ ] **Step 3: Update `settings_ui` to save and broadcast on change**

- [ ] **Step 4: Commit Audio changes**

```bash
git add ray-applets/audio/src/lib.rs
git commit -m "feat: persist and broadcast device index in audio applet"
```

### Task 3: Sync and Audio Recording in Capture Extension

**Files:**
- Modify: `ray-applets/capture/src/lib.rs`

- [ ] **Step 1: Add persistence helpers to CaptureApplet**

```rust
// Add to CaptureApplet struct
audio_enabled: bool,
audio_device_index: i32,

// Add to CaptureApplet impl
fn save_settings(&self) -> anyhow::Result<()> {
    let conn = rusqlite::Connection::open("framework_settings.db")?;
    conn.execute(
        "INSERT OR REPLACE INTO applet_configs (applet, key, value) VALUES (?1, ?2, ?3)",
        rusqlite::params!["capture", "audio_enabled", self.audio_enabled.to_string()],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO applet_configs (applet, key, value) VALUES (?1, ?2, ?3)",
        rusqlite::params!["capture", "audio_device_index", self.audio_device_index.to_string()],
    )?;
    Ok(())
}
```

- [ ] **Step 2: Update `on_event` to handle `AudioEvent::DeviceSelected`**

- [ ] **Step 3: Update `start_recording` to include audio flags in FFmpeg**

```rust
// macOS example
if self.audio_enabled {
    let audio_input = format!("1:{}", self.audio_device_index);
    cmd.args(&["-f", "avfoundation", "-i", &audio_input, "-c:a", "aac"]);
} else {
    cmd.args(&["-f", "avfoundation", "-i", "1:none"]);
}
```

- [ ] **Step 4: Add toggle to `settings_ui`**

- [ ] **Step 5: Commit Capture changes**

```bash
git add ray-applets/capture/src/lib.rs
git commit -m "feat: implement synced audio recording in capture applet"
```

### Task 4: Framework DB Schema Update

**Files:**
- Modify: `ray-core/src/lib.rs`

- [ ] **Step 1: Add `applet_configs` table to `ensure_db_schema`**

```rust
conn.execute(
    "CREATE TABLE IF NOT EXISTS applet_configs (
        applet TEXT,
        key TEXT,
        value TEXT,
        PRIMARY KEY (applet, key)
    )",
    [],
)?;
```

- [ ] **Step 2: Commit Core changes**

```bash
git add ray-core/src/lib.rs
git commit -m "chore: add applet_configs table for extension persistence"
```
