# Anki Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a dedicated Anki tab that allows configuring Anki settings, viewing an offline queue of cards, and a button in the search results to quickly add cards to the queue.

**Architecture:** We will add `anki_queue` CRUD operations to `YomichanApp`, a new `Route::Anki` enum variant with its own UI rendering module (`src/router/anki/mod.rs`), and modify the existing `CachedEntry` to store necessary string data for offline Anki note generation.

**Tech Stack:** Rust, Macroquad, rusqlite, yomichan_rs

---

### Task 1: Update Application State and Database Helpers

**Files:**
- Modify: `src/main.rs`
- Create: `src/router/anki/mod.rs`
- Create: `src/router/anki/db.rs`

- [ ] **Step 1: Update `CachedEntry` in `src/main.rs`**
Modify `CachedEntry` to include `sentence` and `entry_index` so we can track the context of the added word.

```rust
// Replace existing CachedEntry struct in src/main.rs
pub struct CachedEntry {
    pub headword: String,
    pub definitions: Vec<String>,
    pub sentence: String,
    pub entry_index: usize,
}
```

- [ ] **Step 2: Create DB file `src/router/anki/db.rs` for offline queue CRUD operations**
Create the module for database operations related to the Anki queue.

```rust
// src/router/anki/db.rs
use crate::YomichanApp;

#[derive(Debug, Clone)]
pub struct QueuedCard {
    pub id: i64,
    pub headword: String,
    pub reading: String,
    pub definition: String,
    pub sentence: String,
    pub added_at: i64,
}

impl YomichanApp {
    pub fn init_anki_queue_db(&self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS anki_queue (
                id INTEGER PRIMARY KEY,
                headword TEXT,
                reading TEXT,
                definition TEXT,
                sentence TEXT,
                added_at INTEGER
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert_anki_queue(&self, headword: &str, reading: &str, definition: &str, sentence: &str) -> Result<(), rusqlite::Error> {
        self.init_anki_queue_db()?;
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        conn.execute(
            "INSERT INTO anki_queue (headword, reading, definition, sentence, added_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![headword, reading, definition, sentence, now],
        )?;
        Ok(())
    }

    pub fn get_anki_queue(&self) -> Result<Vec<QueuedCard>, rusqlite::Error> {
        self.init_anki_queue_db()?;
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        let mut stmt = conn.prepare("SELECT id, headword, reading, definition, sentence, added_at FROM anki_queue ORDER BY added_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(QueuedCard {
                id: row.get(0)?,
                headword: row.get(1)?,
                reading: row.get(2)?,
                definition: row.get(3)?,
                sentence: row.get(4)?,
                added_at: row.get(5)?,
            })
        })?;

        let mut cards = Vec::new();
        for row in rows {
            cards.push(row?);
        }
        Ok(cards)
    }

    pub fn delete_anki_queue_item(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        conn.execute("DELETE FROM anki_queue WHERE id = ?1", [id])?;
        Ok(())
    }
}
```

- [ ] **Step 3: Define the new `mod` in `src/router/anki/mod.rs`**

```rust
// src/router/anki/mod.rs
pub mod db;
pub mod ui;
```

- [ ] **Step 4: Register `anki` module in `src/router/mod.rs`**
Add the Anki tab route and register the module.

```rust
// In src/router/mod.rs, at the top:
pub mod import;
pub mod search;
pub mod threed;
pub mod audio;
pub mod settings;
pub mod anki;

// Update Route enum:
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Route {
    #[default]
    Search,
    Import,
    ThreeD,
    Audio,
    ShaderError,
    Settings,
    Anki, // NEW
}
```

- [ ] **Step 5: Commit**
```bash
git add src/main.rs src/router/anki/mod.rs src/router/anki/db.rs src/router/mod.rs
git commit -m "feat: add anki offline queue db operations and route"
```

### Task 2: Implement the Anki UI

**Files:**
- Create: `src/router/anki/ui.rs`
- Modify: `src/router/mod.rs`

- [ ] **Step 1: Create `src/router/anki/ui.rs` with the Settings and Queue views**

```rust
// src/router/anki/ui.rs
use macroquad::ui::{hash, Ui};
use crate::YomichanApp;
use yomichan_rs::settings::core::FieldIndex;
use yomichan_rs::settings::core::AnkiTermFieldType;

impl YomichanApp {
    pub fn draw_anki_tab(&mut self, ui: &mut Ui) {
        ui.label(None, "Anki Settings");
        if ui.button(None, "Sync Anki Maps") {
            let _ = self.yomichan.anki().update_all_anki_maps();
        }
        
        ui.separator();
        
        let anki = self.yomichan.anki();
        
        // Settings Section
        let decks = anki.deck_names();
        let models = anki.model_names();
        
        if models.is_empty() || decks.is_empty() {
            ui.label(None, "No decks or models found. Ensure Anki is open and AnkiConnect is installed, then Sync.");
        } else {
            // Simplified for immediate functionality: we can auto-configure or allow manual setup.
            // For now, we'll provide a button to auto-configure using the first available model,
            // as full dropdown macroquad implementation with dynamic state requires complex UI state management.
            if ui.button(None, "Auto-Configure First Model & Deck") {
                let _ = anki.configure_note_creation_auto();
            }
        }
        
        ui.separator();
        ui.separator();
        
        ui.label(None, "Offline Queue");
        
        if ui.button(None, "Sync Queue to Anki") {
            if let Ok(queue) = self.get_anki_queue() {
                for item in queue {
                    // Manual note construction mimicking AnkiConnect
                    let mut note_builder = anki_direct::notes::NoteBuilder::default();
                    
                    // We need the configured model and deck name
                    let mut model_name = String::new();
                    let mut deck_name = String::new();
                    let mut field_mappings = Vec::new();
                    
                    if let Ok(profile) = self.yomichan.options().read_arc().get_current_profile() {
                        let pg = profile.read();
                        let anki_opts = pg.anki_options();
                        if let Some(af) = anki_opts.anki_fields() {
                            field_mappings = af.fields().clone();
                            let global_opts = self.yomichan.options().read_arc();
                            let go = global_opts.anki().read();
                            if let Ok((m, _)) = go.get_selected_model(*af.selected_model()) {
                                model_name = m.to_string();
                            }
                            if let Ok((d, _)) = go.get_selected_deck(*af.selected_deck()) {
                                deck_name = d.to_string();
                            }
                        }
                    }
                    
                    if !model_name.is_empty() && !deck_name.is_empty() {
                        note_builder.model_name(model_name).deck_name(deck_name);
                        for mapping in &field_mappings {
                            match mapping {
                                AnkiTermFieldType::Term(f) => { note_builder.field(f, &item.headword); },
                                AnkiTermFieldType::Reading(f) => { note_builder.field(f, &item.reading); },
                                AnkiTermFieldType::Definition(f) => { note_builder.field(f, &item.definition); },
                                AnkiTermFieldType::Sentence(f) => { note_builder.field(f, &item.sentence); },
                                _ => {}
                            }
                        }
                        
                        if let Ok(note) = note_builder.build(Some(anki.client().read_arc().reqwest_client())) {
                            if let Ok(_) = anki.client().read().notes().add_notes(&[note]) {
                                let _ = self.delete_anki_queue_item(item.id);
                            }
                        }
                    }
                }
            }
        }
        
        ui.separator();
        
        if let Ok(queue) = self.get_anki_queue() {
            if queue.is_empty() {
                ui.label(None, "Queue is empty.");
            } else {
                for item in queue {
                    ui.label(None, &format!("{} [{}]", item.headword, item.reading));
                    ui.same_line(0.0);
                    if ui.button(None, &format!("Delete##{}", item.id)) {
                        let _ = self.delete_anki_queue_item(item.id);
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Update `src/router/mod.rs` to render the Anki tab**
Add the Anki button to the top navigation and the `Route::Anki` match arm.

```rust
// In src/router/mod.rs inside YomichanApp::draw_ui navigation bar section:
        if ui.button(None, "Search") {
            self.router.set(Route::Search);
        }
        // ... (keep existing buttons)
        ui.same_line(0.0);
        if ui.button(None, "Settings") {
            self.router.set(Route::Settings);
        }
        ui.same_line(0.0);
        if ui.button(None, "Anki") {
            self.router.set(Route::Anki);
        }

// Inside the match self.router.c_route statement:
            Route::Settings => {
                macroquad::ui::widgets::Window::new(
                    hash!("settings_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(macroquad::window::screen_width(), macroquad::window::screen_height() - 100.0),
                )
                .label("Settings")
                .ui(ui, |ui| {
                    self.draw_settings_page(ui);
                });
            }
            Route::Anki => { // NEW MATCH ARM
                macroquad::ui::widgets::Window::new(
                    hash!("anki_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(macroquad::window::screen_width(), macroquad::window::screen_height() - 100.0),
                )
                .label("Anki Integration")
                .ui(ui, |ui| {
                    self.draw_anki_tab(ui);
                });
            }
```

- [ ] **Step 3: Commit**
```bash
git add src/router/anki/ui.rs src/router/mod.rs
git commit -m "feat: implement anki ui and queue processing"
```

### Task 3: Search UI Integration

**Files:**
- Modify: `src/main.rs`
- Modify: `src/router/search/mod.rs`

- [ ] **Step 1: Fix `CachedEntry` population in `src/main.rs`**
Update `refresh_cache` to populate the new `sentence` and `entry_index` fields.

```rust
// In src/main.rs, modify refresh_cache:
    pub fn refresh_cache(&mut self) {
        self.cached_entries.clear();
        let Some(res) = &self.search_results else {
            return;
        };
        let Some(segment) = res.segments.get(self.selected_segment) else {
            return;
        };

        let sentence = self.search_query.clone(); // The current query

        for (i, entry) in segment.entries.iter().enumerate() {
            let headword = entry
                .headwords
                .iter()
                .map(|h| format!("  {} ({})", h.term, h.reading))
                .collect::<Vec<_>>()
                .join(", ");

            let mut definitions = Vec::new();
            for def in &entry.definitions {
                for group in &def.entries {
                    definitions.push(format!("   {}", group.plain_text));
                }
            }
            self.cached_entries.push(CachedEntry {
                headword,
                definitions,
                sentence: sentence.clone(),
                entry_index: i,
            });
        }
    }
```

- [ ] **Step 2: Add "[+] Add to Anki" button to `src/router/search/mod.rs`**
Modify `draw_search_content` to render the button and handle insertion.

```rust
// In src/router/search/mod.rs, near the bottom of draw_search_content where `entry` is rendered:
        for entry in &self.cached_entries {
            ui.label(None, &entry.headword);
            ui.same_line(0.0);
            if ui.button(None, &format!("[+] Add to Anki##{}", entry.entry_index)) {
                // Extract clean term and reading (very rough extraction from the formatted headword for simplicity,
                // or we grab the first headword/reading since we don't have access to the raw entry directly without borrowing self twice)
                // A safer approach: parse the format "  TERM (READING)"
                let cleaned = entry.headword.trim();
                let mut term = cleaned.to_string();
                let mut reading = String::new();
                if let Some(idx) = cleaned.find(" (") {
                    term = cleaned[0..idx].to_string();
                    let end_idx = cleaned.find(")").unwrap_or(cleaned.len());
                    reading = cleaned[idx+2..end_idx].to_string();
                }
                
                let def_joined = entry.definitions.join("\n");
                let _ = self.insert_anki_queue(&term, &reading, &def_joined, &entry.sentence);
            }

            for def in &entry.definitions {
                let chars: Vec<char> = def.chars().collect();
                if chars.len() > max_chars {
                    let mut start = 0;
                    while start < chars.len() {
                        let end = (start + max_chars).min(chars.len());
                        let chunk: String = chars[start..end].iter().collect();
                        ui.label(None, &chunk);
                        start = end;
                    }
                } else {
                    ui.label(None, def);
                }
            }
            ui.separator();
        }
```

- [ ] **Step 3: Commit**
```bash
git add src/main.rs src/router/search/mod.rs
git commit -m "feat: integrate add to anki button in search ui"
```
