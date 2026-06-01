pub mod import;
pub mod search;
pub mod threed;

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
        
        ui.separator();

        // --- LANGUAGE SELECTOR ---
        ui.label(None, "Lang:");
        ui.same_line(0.0);
        let old_lang = self.language_index;
        ComboBox::new(hash!("lang_selector"), &["Japanese", "Spanish"])
            .ui(ui, &mut self.language_index);

        if old_lang != self.language_index {
            let iso = if self.language_index == 0 { "ja" } else { "es" };
            if let Ok(_) = self.yomichan.set_language(iso) {
                let _ = self.yomichan.save_settings();
                self.search_results = None;
                self.cached_entries.clear();
            }
        }

        ui.separator();

        // --- CONTENT AREA ---
        match self.router.c_route {
            Route::Search => self.draw_search_content(ui),
            Route::Import => self.draw_import_content(ui),
            Route::ThreeD => self.draw_threed_tab(ui),
        }
    }
}
