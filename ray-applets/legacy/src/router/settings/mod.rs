use macroquad::ui::{Ui, hash};
use crate::YomichanApp;
use std::fs;

impl YomichanApp {
    pub fn scan_system_fonts(&mut self) {
        let font_dirs = [
            "/System/Library/Fonts",
            "/Library/Fonts",
            "/System/Library/Fonts/Supplemental",
            "/Users/aramsamifanni/Library/Fonts",
        ];

        self.discovered_fonts.clear();
        // Add a default/fallback entry if needed
        self.discovered_fonts.push(("Default (Internal)".to_string(), "".to_string()));
        
        // Add local project font if it exists
        if fs::metadata("DotGothic16-Regular.ttf").is_ok() {
            self.discovered_fonts.push(("DotGothic16 (Local)".to_string(), "DotGothic16-Regular.ttf".to_string()));
        }

        for dir in font_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if ext == "ttf" || ext == "otf" || ext == "ttc" {
                            let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                            self.discovered_fonts.push((name, path.to_string_lossy().into_owned()));
                        }
                    }
                }
            }
        }
        
        // Sort by name for easier browsing
        self.discovered_fonts.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    }

    pub fn draw_settings_page(&mut self, ui: &mut Ui) {
        ui.label(None, "Global Settings");
        ui.separator();

        ui.label(None, "Font Selection:");
        if ui.button(None, "Refresh Font List") {
            self.scan_system_fonts();
        }
        ui.same_line(0.0);
        if ui.button(None, "Reset to Default") {
            self.pending_font_update = Some("".to_string());
        }

        ui.separator();

        // Font selection list
        macroquad::ui::widgets::Group::new(hash!("font_list"), macroquad::math::vec2(600.0, 400.0))
            .ui(ui, |ui| {
                for i in 0..self.discovered_fonts.len() {
                    let (name, path) = &self.discovered_fonts[i];
                    let is_selected = self.selected_font_path == *path;
                    
                    let label = if is_selected {
                        format!("> {}", name)
                    } else {
                        name.clone()
                    };

                    if ui.button(None, label.as_str()) {
                        self.pending_font_update = Some(path.clone());
                    }
                }
            });

        ui.separator();
        ui.label(None, &format!("Current Font Path: {}", self.selected_font_path));
    }

    pub fn update_font(&mut self, path: String) {
        self.selected_font_path = path;
        
        let font_bytes = if self.selected_font_path.is_empty() {
            None
        } else {
            std::fs::read(&self.selected_font_path).ok()
        };

        // Update the actual Font object for macroquad drawing if needed
        if let Some(bytes) = &font_bytes {
            self.font = macroquad::text::load_ttf_font_from_bytes(bytes).ok();
        } else {
            self.font = None;
        }

        // Update the UI skin immediately
        self.skin = Self::create_skin(font_bytes.as_deref());
        
        // Save to DB
        let _ = self.save_settings_to_db();
    }
}
