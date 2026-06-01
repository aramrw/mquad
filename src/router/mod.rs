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

        // Draw a background for the UI
        if self.router.c_route != Route::ThreeD {
            macroquad::prelude::draw_rectangle(
                0.0,
                0.0,
                macroquad::window::screen_width(),
                macroquad::window::screen_height(),
                macroquad::prelude::Color::from_rgba(20, 20, 20, 255),
            );
        }

        // Consistent bar for top navigation in ALL modes
        macroquad::prelude::draw_rectangle(
            0.0,
            0.0,
            macroquad::window::screen_width(),
            70.0,
            macroquad::prelude::Color::from_rgba(10, 10, 10, 180),
        );

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
        if self.router.c_route == Route::Search {
            //ui.label(None, "Lang:");
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
        }

        ui.separator();
        // --- CONTENT AREA ---
        match self.router.c_route {
            Route::Search => {
                macroquad::ui::widgets::Window::new(
                    hash!("search_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(screen_width(), screen_height() - 100.0),
                )
                .label("Search")
                .ui(ui, |ui| {
                    self.draw_search_content(ui);
                });
            }
            Route::Import => {
                macroquad::ui::widgets::Window::new(
                    hash!("import_win"),
                    macroquad::math::vec2(0.0, 100.0),
                    macroquad::math::vec2(screen_width(), screen_height() - 100.0),
                )
                .label("Import")
                .ui(ui, |ui| {
                    self.draw_import_content(ui);
                });
            }
            Route::ThreeD => {
                macroquad::ui::widgets::Window::new(
                    hash!("threed_win"),
                    macroquad::math::vec2(10.0, 110.0),
                    macroquad::math::vec2(300.0, 180.0),
                )
                .label("Shader Settings")
                .ui(ui, |ui| {
                    self.draw_threed_tab(ui);
                });
            }
        }
    }
}
