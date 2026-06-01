mod router;

use macroquad::main;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::sync::{Arc, mpsc};
use yomichan_rs::Yomichan;

use crate::router::Route;
use crate::router::Router;

struct YomichanApp {
    router: Router,
    yomichan: Arc<Yomichan>,
    search_query: String,
    import_status: String,
    progress_receiver: mpsc::Receiver<String>,
    progress_sender: mpsc::Sender<String>,
}

impl YomichanApp {
    pub fn draw_ui(&mut self) {
        self.nav_buttons();
        match self.router.c_route {
            Route::Search => self.search_results(),
            Route::Import => self.import_results(),
        }
    }

    fn nav_buttons(&mut self) {
        use macroquad::ui::widgets::Window;
        use macroquad::ui::hash;
        
        Window::new(hash!(), vec2(10., 10.), vec2(200., 50.))
            .label("Navigation")
            .titlebar(true)
            .ui(&mut root_ui(), |ui| {
                if ui.button(None, "Search") {
                    self.router.set(Route::Search);
                }
                ui.same_line(0.0);
                if ui.button(None, "Import") {
                    self.router.set(Route::Import);
                }
            });
    }

    // Add empty placeholder for now
    fn import_results(&mut self) {}

    // Add empty placeholder for now
    fn search_results(&mut self) {}
}

#[main("...")]
async fn main() {
    // Defines a camera where the screen spans from -1.0 to 1.0
    // and the origin (0, 0) is perfectly in the center.
    let camera = Camera2D {
        // Negative Y zoom flips the axis to match GLSL
        zoom: vec2(1.0, -1.0),
        target: vec2(0.0, 0.0),
        ..Default::default()
    };

    // let mut app = YomichanApp::new(Router::default());

    loop {
        clear_background(BLACK);
        set_default_camera();

        // app.draw_ui();

        next_frame().await;
    }
}
