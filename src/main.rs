mod router;

use macroquad::main;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::sync::{Arc, mpsc};
use tracing::field::{Field, Visit};
use tracing_core::{Event, Subscriber};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::{Context, SubscriberExt};
use yomichan_rs::Yomichan;

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
    search_results: Option<yomichan_rs::SearchResult>,
    selected_segment: usize,
    import_status: String,
    progress_receiver: mpsc::Receiver<String>,
    progress_sender: mpsc::Sender<String>,
    language_index: usize,
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

    fn draw_import_tab(&mut self) {
        use macroquad::ui::hash;
        use macroquad::ui::widgets::Window;

        // Drain pending progress messages
        while let Ok(msg) = self.progress_receiver.try_recv() {
            self.import_status = msg;
        }

        Window::new(hash!(), vec2(10., 70.), vec2(400., 400.))
            .label("Import Dictionary")
            .ui(&mut root_ui(), |ui| {
                ui.label(None, "Select a Yomitan .zip dictionary file:");

                if ui.button(None, "Open File Dialog") {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("zip", &["zip"])
                        .pick_file()
                    {
                        self.import_status = format!("Selected: {:?}", path);
                        let ycd = self.yomichan.clone();
                        let tx = self.progress_sender.clone();

                        std::thread::spawn(move || {
                            let _ = tx.send("Starting import...".into());
                            match ycd.import_dictionaries(&[path]) {
                                Ok(_) => {
                                    let _ = tx.send("Import complete!".into());
                                }
                                Err(e) => {
                                    let _ = tx.send(format!("Error: {:?}", e));
                                }
                            }
                        });
                    }
                }

                ui.separator();
                ui.label(None, "Status:");
                ui.label(None, &self.import_status);
            });
    }

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

    let ycd = Arc::new(Yomichan::new(".").expect("Failed to init database"));

    let mut app = YomichanApp {
        router: Router::default(),
        yomichan: ycd,
        search_query: String::new(),
        search_results: None,
        selected_segment: 0,
        import_status: String::new(),
        progress_receiver: rx,
        progress_sender: tx,
        language_index: 0,
    };

    loop {
        clear_background(BLACK);
        set_default_camera();
        app.draw_ui();
        next_frame().await;
    }
}
