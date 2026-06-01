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
                self.search_results = self.yomichan.search(&self.search_query).ok();
            }

            ui.separator();

            if let Some(res) = &self.search_results {
                for segment in res.segments.iter().take(5) {
                    if segment.entries.is_empty() {
                        ui.label(None, &format!("{}...。", segment.text));
                        continue;
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
