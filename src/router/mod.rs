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
            vec2(10., 10.),
            vec2(screen_width() - 20., screen_height() - 30.),
        )
        .label("Ray")
        .titlebar(true)
        .ui(&mut root_ui(), |ui| {
            // Navigation Row
            ui.label(None, "Navigation:");
            if ui.button(None, "Search") {
                self.router.set(Route::Search);
            }
            ui.same_line(0.0);
            if ui.button(None, "Import") {
                self.router.set(Route::Import);
            }
            ui.same_line(0.0);
            ui.label(None, " | Lang:");
            //ui.same_line(0.0);
            let old_lang = self.language_index;
            ComboBox::new(hash!("lang_selector"), &["Japanese", "Spanish"])
                .ui(ui, &mut self.language_index);

            if old_lang != self.language_index {
                let iso = if self.language_index == 0 { "ja" } else { "es" };
                if let Ok(_) = self.yomichan.set_language(iso) {
                    let _ = self.yomichan.save_settings();
                    self.search_results = None;
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
