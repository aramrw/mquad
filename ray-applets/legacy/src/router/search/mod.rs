use macroquad::ui::{Ui, hash, widgets::InputText};

use crate::YomichanApp;

impl YomichanApp {
    fn get_clipboard(&self) -> Option<String> {
        use std::process::Command;
        let output = Command::new("pbpaste").output().ok()?;
        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            None
        }
    }

    pub fn draw_search_content(&mut self, ui: &mut Ui) {
        use macroquad::input::{KeyCode, is_key_down, is_key_pressed};

        // Handle Paste (Cmd + V)
        let super_down = is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper);
        if super_down && is_key_pressed(KeyCode::V) {
            if let Some(content) = self.get_clipboard() {
                self.search_query.push_str(&content);
            }
        }

        InputText::new(hash!("search_input")).ui(ui, &mut self.search_query);

        let search_clicked = ui.button(None, "Search");
        let enter_pressed = is_key_pressed(KeyCode::Enter);

        if (search_clicked || enter_pressed) && !self.search_query.is_empty() {
            if let Ok(res) = self.yomichan.search(&self.search_query) {
                // Set selected segment to the first non-whitespace one
                self.selected_segment = res
                    .segments
                    .iter()
                    .position(|s| !s.text.trim().is_empty())
                    .unwrap_or(0);
                self.search_results = Some(res);
                self.refresh_cache();
            } else {
                self.search_results = None;
                self.cached_entries.clear();
            }
        }

        ui.separator();

        let mut next_segment = None;
        if let Some(res) = &self.search_results {
            let mut rendered_count = 0;
            for (i, segment) in res.segments.iter().enumerate() {
                if segment.text.trim().is_empty() {
                    continue;
                }
                if rendered_count > 0 {
                    ui.same_line(0.0);
                    ui.label(None, "·");
                    ui.same_line(0.0);
                }
                if ui.button(None, segment.text.as_str()) {
                    next_segment = Some(i);
                }
                rendered_count += 1;
            }
            ui.separator();
            ui.separator();
        }

        if let Some(i) = next_segment {
            self.selected_segment = i;
            self.refresh_cache();
        }

        if let Some(res) = &self.search_results {
            if self.cached_entries.is_empty() && !res.segments.is_empty() {
                // Handle edge case where first segment has no entries but wasn't skipped
                if let Some(segment) = res.segments.get(self.selected_segment) {
                    if !segment.text.trim().is_empty() && segment.entries.is_empty() {
                        ui.label(None, "No entries found.");
                    }
                }
            }
        }

        // Render cached segment definitions
        let max_width = macroquad::window::screen_width() - 40.0;
        let char_width = 15.0; // Estimate for CJK/Pixel mixed text
        let max_chars = (max_width / char_width) as usize;

        for entry in &self.cached_entries {
            ui.label(None, &entry.headword);
            ui.same_line(0.0);
            if ui.button(None, format!("[+]").as_str()) {
                let cleaned = entry.headword.trim();
                let mut term = cleaned.to_string();
                let mut reading = String::new();
                if let Some(idx) = cleaned.find(" (") {
                    term = cleaned[0..idx].to_string();
                    let end_idx = cleaned.find(")").unwrap_or(cleaned.len());
                    reading = cleaned[idx + 2..end_idx].to_string();
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
    }
}

