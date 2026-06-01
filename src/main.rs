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

pub struct YomichanApp {
    pub router: Router,
    pub yomichan: Arc<Yomichan>,
    pub search_query: String,
    pub search_results: Option<yomichan_rs::SearchResult>,
    pub selected_segment: usize,
    pub import_status: String,
    pub progress_receiver: mpsc::Receiver<String>,
    pub progress_sender: mpsc::Sender<String>,
    pub language_index: usize,
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
