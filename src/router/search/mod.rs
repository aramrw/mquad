use macroquad::ui::{Ui, hash, widgets::InputText};

use crate::YomichanApp;

impl YomichanApp {
    pub fn draw_search_content(&mut self, ui: &mut Ui) {
        use macroquad::input::{KeyCode, is_key_pressed};

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
        for entry in &self.cached_entries {
            ui.label(None, &entry.headword);
            for def in &entry.definitions {
                ui.label(None, def);
            }
            ui.separator();
        }
    }
}
