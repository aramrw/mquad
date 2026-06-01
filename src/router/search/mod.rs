use macroquad::math::vec2;
use macroquad::ui::{Ui, hash, widgets::InputText};
use crate::YomichanApp;

impl YomichanApp {
    pub fn draw_search_content(&mut self, ui: &mut Ui) {
        use macroquad::input::{is_key_pressed, KeyCode};
        use macroquad::ui::widgets::Group;

        InputText::new(hash!("search_input")).ui(ui, &mut self.search_query);

        let search_clicked = ui.button(None, "Search");
        let enter_pressed = is_key_pressed(KeyCode::Enter);

        if (search_clicked || enter_pressed) && !self.search_query.is_empty() {
            if let Ok(res) = self.yomichan.search(&self.search_query) {
                // Set selected segment to the first non-whitespace one
                self.selected_segment = res.segments
                    .iter()
                    .position(|s| !s.text.trim().is_empty())
                    .unwrap_or(0);
                self.search_results = Some(res);
            } else {
                self.search_results = None;
            }
        }

        ui.separator();

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
                    self.selected_segment = i;
                }
                rendered_count += 1;
            }
            ui.separator();

            // Render selected segment definitions in a Group for separation
            if let Some(segment) = res.segments.get(self.selected_segment) {
                use macroquad::window::screen_width;
                Group::new(hash!("results_group"), vec2(screen_width() - 40.0, 300.0)).ui(ui, |ui| {
                    if segment.entries.is_empty() {
                        ui.label(None, "No entries found.");
                    }
                    for entry in &segment.entries {
                        let headword_str = entry
                            .headwords
                            .iter()
                            .map(|h| format!("{} ({})", h.term.clone(), h.reading.clone()))
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.label(None, &headword_str);

                        for def in &entry.definitions {
                            for group in &def.entries {
                                ui.label(None, &format!("- {}", group.plain_text));
                            }
                        }
                        ui.separator();
                    }
                });
            }
        }
    }
}
