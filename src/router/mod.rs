pub mod search;
pub mod import;

use macroquad::{
    math::vec2,
    ui::root_ui,
    window::screen_width,
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
        self.nav_buttons();
        match self.router.c_route {
            Route::Search => self.draw_search_tab(),
            Route::Import => self.draw_import_tab(),
        }
    }

    fn nav_buttons(&mut self) {
        use macroquad::ui::hash;
        use macroquad::ui::widgets::{ComboBox, Window};

        // Navigation
        Window::new(hash!(), vec2(10., 10.), vec2(screen_width() - 20., 70.))
            .titlebar(true)
            .ui(&mut root_ui(), |ui| {
                if ui.button(None, "Search") {
                    self.router.set(Route::Search);
                }
                ui.same_line(0.0);
                if ui.button(None, "Import") {
                    self.router.set(Route::Import);
                }
                ui.same_line(0.0);
                ui.label(None, "Lang:");
                ui.same_line(0.0);
                let old_lang = self.language_index;
                ComboBox::new(hash!(), &["Japanese", "Spanish"]).ui(ui, &mut self.language_index);
                if old_lang != self.language_index {
                    let iso = if self.language_index == 0 { "ja" } else { "es" };
                    if let Ok(_) = self.yomichan.set_language(iso) {
                        let _ = self.yomichan.save_settings();
                        // clear search results on language change
                        self.search_results = None;
                    }
                }
            });
    }
}
