pub mod import;
pub mod search;

use macroquad::{
    math::vec2,
    ui::root_ui,
    window::{screen_height, screen_width},
};

use crate::YomichanApp;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Route {
    #[default]
    Search,
    Import,
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
        use macroquad::ui::widgets::{ComboBox, Window};

        Window::new(
            hash!("ray_main_window"),
            vec2(0., 0.),
            vec2(screen_width(), screen_height()),
        )
        .titlebar(false)
        .movable(false)
        .ui(&mut root_ui(), |ui| {
            ui.separator();

            // Row 2: Mode Selection
            ui.label(None, "Select Mode:");
            if ui.button(None, "Search") {
                self.router.set(Route::Search);
            }
            ui.same_line(0.0);
            if ui.button(None, "Import") {
                self.router.set(Route::Import);
            }

            ui.separator();

            // Row 3: Language Selection
            ui.label(None, "Select Language:");
            let old_lang = self.language_index;
            ComboBox::new(hash!("lang_selector"), &["Japanese", "Spanish"])
                .ui(ui, &mut self.language_index);
if old_lang != self.language_index {
    let iso = if self.language_index == 0 { "ja" } else { "es" };
    if let Ok(_) = self.yomichan.set_language(iso) {
        let _ = self.yomichan.save_settings();
        // clear search results on language change
        self.search_results = None;
        self.cached_entries.clear();
    }
}

            ui.separator();

            // Content Area
            match self.router.c_route {
                Route::Search => self.draw_search_content(ui),
                Route::Import => self.draw_import_content(ui),
            }
        });
    }
}
