mod router;
mod tracing_utils;

use macroquad::main;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::sync::{Arc, mpsc};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use yomichan_rs::Yomichan;

use crate::router::Route;
use crate::router::Router;
use crate::router::threed::ThreeDState;
use crate::router::audio::AudioState;
use crate::tracing_utils::ProgressLayer;

pub struct CachedEntry {
    pub headword: String,
    pub definitions: Vec<String>,
    pub sentence: String,
    pub entry_index: usize,
}

pub struct YomichanApp {
    pub router: Router,
    pub yomichan: Arc<Yomichan>,
    pub search_query: String,
    pub search_results: Option<yomichan_rs::SearchResult>,
    pub selected_segment: usize,
    pub cached_entries: Vec<CachedEntry>,
    pub import_status: String,
    pub progress_receiver: mpsc::Receiver<String>,
    pub progress_sender: mpsc::Sender<String>,
    pub language_index: usize,
    pub threed_state: ThreeDState,
    pub audio_state: AudioState,
    pub skin: macroquad::ui::Skin,
    pub show_ui: bool,
    pub font: Option<Font>,
    pub discovered_fonts: Vec<(String, String)>, // (Name, Path)
    pub selected_font_path: String,
    pub pending_font_update: Option<String>,
    pub anki_deck_idx: usize,
    pub anki_model_idx: usize,
    pub anki_field_term_idx: usize,
    pub anki_field_reading_idx: usize,
    pub anki_field_def_idx: usize,
    pub anki_field_sentence_idx: usize,
}

const DEFAULT_FONT_SIZE: u8 = 24;

impl YomichanApp {
    pub fn create_skin(font_bytes: Option<&[u8]>) -> macroquad::ui::Skin {
        use macroquad::ui::root_ui;

        let mut label_builder = root_ui().style_builder();
        if let Some(bytes) = font_bytes {
            if let Ok(b) = label_builder.font(bytes) {
                label_builder = b;
            } else {
                label_builder = root_ui().style_builder();
            }
        }
        let label_style = label_builder
            .text_color(Color::from_rgba(220, 220, 220, 255))
            .font_size(DEFAULT_FONT_SIZE as u16)
            .build();

        let mut window_builder = root_ui().style_builder();
        if let Some(bytes) = font_bytes {
            if let Ok(b) = window_builder.font(bytes) {
                window_builder = b;
            } else {
                window_builder = root_ui().style_builder();
            }
        }
        let window_style = window_builder
            .color(Color::from_rgba(30, 30, 30, 255))
            .text_color(Color::from_rgba(255, 255, 255, 255))
            .font_size(20)
            .build();

        let mut button_builder = root_ui().style_builder();
        if let Some(bytes) = font_bytes {
            if let Ok(b) = button_builder.font(bytes) {
                button_builder = b;
            } else {
                button_builder = root_ui().style_builder();
            }
        }
        let button_style = button_builder
            .color(Color::from_rgba(50, 50, 50, 255))
            .color_hovered(Color::from_rgba(70, 70, 70, 255))
            .color_clicked(Color::from_rgba(90, 90, 90, 255))
            .text_color(Color::from_rgba(255, 255, 255, 255))
            .font_size(DEFAULT_FONT_SIZE as u16)
            .build();

        let mut editbox_builder = root_ui().style_builder();
        if let Some(bytes) = font_bytes {
            if let Ok(b) = editbox_builder.font(bytes) {
                editbox_builder = b;
            } else {
                editbox_builder = root_ui().style_builder();
            }
        }
        let editbox_style = editbox_builder
            .color(Color::from_rgba(40, 40, 40, 255))
            .text_color(Color::from_rgba(240, 240, 240, 255))
            .font_size(DEFAULT_FONT_SIZE as u16)
            .build();

        macroquad::ui::Skin {
            label_style,
            window_style,
            button_style,
            editbox_style,
            ..root_ui().default_skin().clone()
        }
    }

    pub fn refresh_cache(&mut self) {
        self.cached_entries.clear();
        let Some(res) = &self.search_results else {
            return;
        };
        let Some(segment) = res.segments.get(self.selected_segment) else {
            return;
        };

        let sentence = self.search_query.clone();

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

    pub fn save_settings_to_db(&self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ray_settings (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO ray_settings (key, value) VALUES (?1, ?2)",
            ["selected_font_path", &self.selected_font_path],
        )?;
        Ok(())
    }

    pub fn save_language_to_db(&self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ray_settings (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO ray_settings (key, value) VALUES (?1, ?2)",
            ["language_index", &self.language_index.to_string()],
        )?;
        Ok(())
    }

    pub fn save_anki_settings_to_db(&self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ray_settings (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )?;
        conn.execute("INSERT OR REPLACE INTO ray_settings (key, value) VALUES (?1, ?2)", ["anki_deck_idx", &self.anki_deck_idx.to_string()])?;
        conn.execute("INSERT OR REPLACE INTO ray_settings (key, value) VALUES (?1, ?2)", ["anki_model_idx", &self.anki_model_idx.to_string()])?;
        conn.execute("INSERT OR REPLACE INTO ray_settings (key, value) VALUES (?1, ?2)", ["anki_field_term_idx", &self.anki_field_term_idx.to_string()])?;
        conn.execute("INSERT OR REPLACE INTO ray_settings (key, value) VALUES (?1, ?2)", ["anki_field_reading_idx", &self.anki_field_reading_idx.to_string()])?;
        conn.execute("INSERT OR REPLACE INTO ray_settings (key, value) VALUES (?1, ?2)", ["anki_field_def_idx", &self.anki_field_def_idx.to_string()])?;
        conn.execute("INSERT OR REPLACE INTO ray_settings (key, value) VALUES (?1, ?2)", ["anki_field_sentence_idx", &self.anki_field_sentence_idx.to_string()])?;
        Ok(())
    }

    pub fn load_settings_from_db(&mut self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        // Check if table exists
        let table_exists: bool = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='ray_settings'",
            [],
            |row| row.get(0),
        )?;

        if table_exists {
            let mut stmt = conn.prepare("SELECT value FROM ray_settings WHERE key = ?1")?;
            
            // Load Font
            {
                let mut rows = stmt.query(["selected_font_path"])?;
                if let Some(row) = rows.next()? {
                    let path: String = row.get(0)?;
                    if !path.is_empty() {
                        self.update_font(path);
                    }
                }
            }

            // Load Language Index
            {
                let mut rows = stmt.query(["language_index"])?;
                if let Some(row) = rows.next()? {
                    let idx_str: String = row.get(0)?;
                    if let Ok(idx) = idx_str.parse::<usize>() {
                        self.language_index = idx;
                        let iso = if self.language_index == 0 { "ja" } else { "es" };
                        let _ = self.yomichan.set_language(iso);
                    }
                } else {
                    // Fallback to ja if no settings exist yet
                    let _ = self.yomichan.set_language("ja");
                }
            }

            // Load Anki Settings
            {
                let mut load_idx = |key: &str| -> Option<usize> {
                    if let Ok(mut rows) = stmt.query([key]) {
                        if let Ok(Some(row)) = rows.next() {
                            if let Ok(val_str) = row.get::<_, String>(0) {
                                return val_str.parse::<usize>().ok();
                            }
                        }
                    }
                    None
                };

                if let Some(idx) = load_idx("anki_deck_idx") { self.anki_deck_idx = idx; }
                if let Some(idx) = load_idx("anki_model_idx") { self.anki_model_idx = idx; }
                if let Some(idx) = load_idx("anki_field_term_idx") { self.anki_field_term_idx = idx; }
                if let Some(idx) = load_idx("anki_field_reading_idx") { self.anki_field_reading_idx = idx; }
                if let Some(idx) = load_idx("anki_field_def_idx") { self.anki_field_def_idx = idx; }
                if let Some(idx) = load_idx("anki_field_sentence_idx") { self.anki_field_sentence_idx = idx; }
            }
        } else {
            // Table doesn't exist, default to ja
            let _ = self.yomichan.set_language("ja");
        }
        Ok(())
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Ray".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

#[main(window_conf)]
async fn main() {
    let (tx, rx) = mpsc::channel();

    let subscriber = Registry::default().with(ProgressLayer { sender: tx.clone() });
    tracing::subscriber::set_global_default(subscriber).ok();

    let ycd = Arc::new(Yomichan::new(".").expect("Failed to init database"));

    let mut app = YomichanApp {
        router: Router::default(),
        yomichan: ycd,
        search_query: String::new(),
        search_results: None,
        selected_segment: 0,
        cached_entries: Vec::new(),
        import_status: String::new(),
        progress_receiver: rx,
        progress_sender: tx,
        language_index: 0,
        threed_state: ThreeDState::default(),
        audio_state: AudioState::default(),
        skin: YomichanApp::create_skin(None),
        show_ui: true,
        font: None,
        discovered_fonts: Vec::new(),
        selected_font_path: String::new(),
        pending_font_update: None,
        anki_deck_idx: 0,
        anki_model_idx: 0,
        anki_field_term_idx: 0,
        anki_field_reading_idx: 0,
        anki_field_def_idx: 0,
        anki_field_sentence_idx: 0,
    };

    app.scan_system_fonts();
    let _ = app.load_settings_from_db();

    app.refresh_shader_list();
    app.compile_shader();

    loop {
        clear_background(BLACK);

        if let Some(path) = app.pending_font_update.take() {
            app.update_font(path);
        }

        if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::H) {
            app.show_ui = !app.show_ui;
        }

        app.update_audio();

        if app.router.c_route == Route::ThreeD {
            app.render_threed_scene();
        }

        if app.show_ui {
            root_ui().push_skin(&app.skin);
            app.draw_ui();
            root_ui().pop_skin();
        }

        next_frame().await;
    }
}
