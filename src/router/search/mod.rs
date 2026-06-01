use macroquad::{
    math::vec2,
    ui::root_ui,
    window::{screen_height, screen_width},
};

use crate::YomichanApp;

impl YomichanApp {
    pub fn draw_search_tab(&mut self) {
        use macroquad::ui::hash;
        use macroquad::ui::widgets::{InputText, Window};

        Window::new(
            hash!(),
            vec2(10., 90.),
            vec2(screen_width() - 20., screen_height() - 40.),
        )
        .label("Search")
        .ui(&mut root_ui(), |ui| {
            //ui.label(None, "text:");
            InputText::new(hash!()).ui(ui, &mut self.search_query);

            if ui.button(None, "Search") && !self.search_query.is_empty() {
                println!("Searching for: {}", self.search_query);
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
                // Render segment selection buttons
                //ui.label(None, "Segments:");
                let mut rendered_count = 0;
                for (i, segment) in res.segments.iter().enumerate() {
                    if segment.text.trim().is_empty() {
                        continue;
                    }
                    if rendered_count > 0 {
                        ui.same_line(0.0);
                    }
                    if ui.button(None, segment.text.as_str()) {
                        self.selected_segment = i;
                    }
                    rendered_count += 1;
                }
                ui.separator();

                // Render selected segment definitions
                if let Some(segment) = res.segments.get(self.selected_segment) {
                    ui.label(None, &format!("Selected: {}", segment.text));
                    if segment.entries.is_empty() {
                        ui.label(None, "No results for this segment.");
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
                }
            }
        });
    }
}
