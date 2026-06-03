pub mod import;
pub mod search;
pub mod threed;
pub mod audio;
pub mod settings;
pub mod anki;

use macroquad::{
    ui::root_ui,
    window::{screen_height, screen_width},
};

use crate::YomichanApp;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Route {
    #[default]
    Search,
    Import,
    ThreeD,
    Audio,
    ShaderError,
    Settings,
    Anki,
}

#[derive(Default)]
pub struct Router {
    pub c_route: Route,
}

impl Router {
    pub fn set(&mut self, r: Route) {
        self.c_route = r;
    }
}

impl YomichanApp {
    pub fn draw_ui(&mut self) {
        use macroquad::ui::hash;
        use macroquad::ui::widgets::ComboBox;

        // Draw a background for the UI
        if self.router.c_route != Route::ThreeD && self.router.c_route != Route::Audio && self.router.c_route != Route::ShaderError {
            macroquad::prelude::draw_rectangle(
                0.0,
                0.0,
                macroquad::window::screen_width(),
                macroquad::window::screen_height(),
                macroquad::prelude::Color::from_rgba(20, 20, 20, 255),
            );
        }

        // Consistent bar for top navigation in ALL modes
        macroquad::prelude::draw_rectangle(
            0.0,
            0.0,
            macroquad::window::screen_width(),
            70.0,
            macroquad::prelude::Color::from_rgba(10, 10, 10, 180),
        );

        let ui = &mut root_ui();

        // --- COMPACT NAVIGATION (Top) ---
        if ui.button(None, "Search") {
            self.router.set(Route::Search);
        }
        ui.same_line(0.0);
        if ui.button(None, "Import") {
            self.router.set(Route::Import);
        }
        ui.same_line(0.0);
        if ui.button(None, "Shaders") {
            self.router.set(Route::ThreeD);
        }
        ui.same_line(0.0);
        if ui.button(None, "Audio") {
            self.router.set(Route::Audio);
        }
        ui.same_line(0.0);
        if ui.button(None, "Settings") {
            self.router.set(Route::Settings);
        }
        ui.same_line(0.0);
        if ui.button(None, "Anki") {
            self.router.set(Route::Anki);
        }

        ui.separator();

        // --- LANGUAGE SELECTOR ---
        if self.router.c_route == Route::Search {
            //ui.label(None, "Lang:");
            ui.same_line(0.0);
            let old_lang = self.language_index;
            ComboBox::new(hash!("lang_selector"), &["Japanese", "Spanish"])
                .ui(ui, &mut self.language_index);

            if old_lang != self.language_index {
                let iso = if self.language_index == 0 { "ja" } else { "es" };
                if let Ok(_) = self.yomichan.set_language(iso) {
                    let _ = self.yomichan.save_settings();
                    let _ = self.save_language_to_db(); // Save to our persistent settings table
                    self.search_results = None;
                    self.cached_entries.clear();
                }
            }
        }

        ui.separator();
        // --- CONTENT AREA ---
        match self.router.c_route {
            Route::Search => {
                macroquad::ui::widgets::Window::new(
                    hash!("search_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(screen_width(), screen_height() - 100.0),
                )
                .label("Search")
                .ui(ui, |ui| {
                    self.draw_search_content(ui);
                });
            }
            Route::Import => {
                macroquad::ui::widgets::Window::new(
                    hash!("import_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(screen_width(), screen_height() - 100.0),
                )
                .label("Import")
                .ui(ui, |ui| {
                    self.draw_import_content(ui);
                });
            }
            Route::ThreeD => {
                self.draw_threed_tab(ui);
            }
            Route::Audio => {
                macroquad::ui::widgets::Window::new(
                    hash!("audio_win"),
                    macroquad::math::vec2(10.0, 110.0),
                    macroquad::math::vec2(screen_width() - 20.0, screen_height() - 120.0),
                )
                .label("Audio Recorder")
                .ui(ui, |ui| {
                    self.draw_audio_tab(ui);
                });
            }
            Route::Settings => {
                macroquad::ui::widgets::Window::new(
                    hash!("settings_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(screen_width(), screen_height() - 100.0),
                )
                .label("Settings")
                .ui(ui, |ui| {
                    self.draw_settings_page(ui);
                });
            }
            Route::Anki => {
                macroquad::ui::widgets::Window::new(
                    hash!("anki_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(screen_width(), screen_height() - 100.0),
                )
                .label("Anki Integration")
                .ui(ui, |ui| {
                    self.draw_anki_tab(ui);
                });
            }
            Route::ShaderError => {
                macroquad::ui::widgets::Window::new(
                    hash!("error_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(screen_width(), screen_height() - 100.0),
                )
                .label("Shader Compilation Errors")
                .ui(ui, |ui| {
                    if ui.button(None, "< Back to Shaders") {
                        self.router.set(Route::ThreeD);
                    }
                    ui.same_line(0.0);
                    if ui.button(None, "Copy Errors") {
                        if let Some(err) = &self.threed_state.shader_error {
                            use std::io::Write;
                            use std::process::{Command, Stdio};
                            if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                                if let Some(mut stdin) = child.stdin.take() {
                                    let _ = stdin.write_all(err.as_bytes());
                                }
                                let _ = child.wait();
                            }
                        }
                    }
                    ui.separator();
                    if let Some(err) = &self.threed_state.shader_error {
                        let max_width = screen_width() - 40.0;
                        let char_width = 10.0; // Rough estimate for mono font
                        let max_chars = (max_width / char_width) as usize;

                        for line in err.lines() {
                            let chars: Vec<char> = line.chars().collect();
                            if chars.len() > max_chars {
                                // Simple wrap for long lines
                                let mut start = 0;
                                while start < chars.len() {
                                    let end = (start + max_chars).min(chars.len());
                                    let chunk: String = chars[start..end].iter().collect();
                                    ui.label(None, &chunk);
                                    start = end;
                                }
                            } else {
                                ui.label(None, line);
                            }
                        }
                    } else {
                        ui.label(None, "No errors found.");
                    }
                });
            }
        }
    }
}