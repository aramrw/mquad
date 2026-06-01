mod router;

use macroquad::main;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::sync::{Arc, mpsc};
use yomichan_rs::Yomichan;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::Registry;
use tracing_core::{Subscriber, Event};
use tracing::field::{Visit, Field};

use crate::router::Route;

struct ProgressLayer {
    sender: mpsc::Sender<String>,
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for ProgressLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target().starts_with("yomichan_importer") {
            struct StringVisitor(String);
            impl Visit for StringVisitor {
                fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                    if field.name() == "message" {
                        self.0 = format!("{:?}", value);
                    }
                }
            }
            let mut visitor = StringVisitor(String::new());
            event.record(&mut visitor);
            if !visitor.0.is_empty() {
                let _ = self.sender.send(visitor.0.trim_matches('"').to_string());
            }
        }
    }
}
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
        use macroquad::ui::hash;
        use macroquad::ui::widgets::Window;

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
    let (tx, rx) = mpsc::channel();
    
    // Register the tracing subscriber
    let subscriber = Registry::default().with(ProgressLayer { sender: tx.clone() });
    tracing::subscriber::set_global_default(subscriber).ok();

    let _camera = Camera2D {
        zoom: vec2(1.0, -1.0),
        target: vec2(0.0, 0.0),
        ..Default::default()
    };

    let ycd = Arc::new(Yomichan::new("db.ycd").expect("Failed to init database"));
    
    let mut app = YomichanApp {
        router: Router::default(),
        yomichan: ycd,
        search_query: String::new(),
        import_status: String::new(),
        progress_receiver: rx,
        progress_sender: tx,
    };

    loop {
        clear_background(BLACK);
        set_default_camera();
        app.draw_ui();
        next_frame().await;
    }
}
