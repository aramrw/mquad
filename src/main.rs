mod router;
mod tracing_utils;

use macroquad::main;
use macroquad::prelude::*;
use std::sync::{Arc, mpsc};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use yomichan_rs::Yomichan;

use crate::router::Router;
use crate::tracing_utils::ProgressLayer;

pub struct CachedEntry {
    pub headword: String,
    pub definitions: Vec<String>,
}

pub struct YomichanApp {
    pub router: Router,
    pub yomichan: Arc<Yomichan>,
    pub search_query: String,
    pub search_results: Option<yomichan_rs::SearchResult>,
    pub selected_segment: usize,
    pub cached_entries: Vec<CachedEntry>,
    pub import_status: String,
    pub progress_receiver: mpsc::Receiver<String>,
    pub progress_sender: mpsc::Sender<String>,
    pub language_index: usize,
}

impl YomichanApp {
    pub fn refresh_cache(&mut self) {
        self.cached_entries.clear();
        let Some(res) = &self.search_results else {
            return;
        };
        let Some(segment) = res.segments.get(self.selected_segment) else {
            return;
        };

        for entry in &segment.entries {
            let headword = entry
                .headwords
                .iter()
                .map(|h| format!("* {} ({})", h.term, h.reading))
                .collect::<Vec<_>>()
                .join(", ");

            let mut definitions = Vec::new();
            for def in &entry.definitions {
                for group in &def.entries {
                    definitions.push(format!(" {}", group.plain_text));
                }
            }
            self.cached_entries.push(CachedEntry {
                headword,
                definitions,
            });
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Ray".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

#[main(window_conf)]
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
        cached_entries: Vec::new(),
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
