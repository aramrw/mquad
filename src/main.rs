mod router;
mod tracing_utils;

use macroquad::main;
use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::sync::{Arc, mpsc};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use yomichan_rs::Yomichan;

use crate::router::Route;
use crate::router::Router;
use crate::router::threed::ThreeDState;
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
    pub threed_state: ThreeDState,
    pub skin: macroquad::ui::Skin,
    pub show_ui: bool,
}

const DEFAULT_FONT_SIZE: u8 = 24;

impl YomichanApp {
    pub fn create_skin() -> macroquad::ui::Skin {
        use macroquad::ui::root_ui;

        let label_style = root_ui()
            .style_builder()
            .text_color(Color::from_rgba(220, 220, 220, 255))
            .font_size(DEFAULT_FONT_SIZE as u16)
            .build();

        let window_style = root_ui()
            .style_builder()
            .color(Color::from_rgba(30, 30, 30, 255))
            .text_color(Color::from_rgba(255, 255, 255, 255))
            .font_size(20)
            .build();

        let button_style = root_ui()
            .style_builder()
            .color(Color::from_rgba(50, 50, 50, 255))
            .color_hovered(Color::from_rgba(70, 70, 70, 255))
            .color_clicked(Color::from_rgba(90, 90, 90, 255))
            .text_color(Color::from_rgba(255, 255, 255, 255))
            .font_size(DEFAULT_FONT_SIZE as u16)
            .build();

        let editbox_style = root_ui()
            .style_builder()
            .color(Color::from_rgba(40, 40, 40, 255))
            .text_color(Color::from_rgba(240, 240, 240, 255))
            .font_size(DEFAULT_FONT_SIZE as u16)
            .build();

        macroquad::ui::Skin {
            label_style,
            window_style,
            button_style,
            editbox_style,
            ..root_ui().default_skin().clone()
        }
    }

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
                .map(|h| format!("  {} ({})", h.term, h.reading))
                .collect::<Vec<_>>()
                .join(", ");

            let mut definitions = Vec::new();
            for def in &entry.definitions {
                for group in &def.entries {
                    definitions.push(format!("   {}", group.plain_text));
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

    let subscriber = Registry::default().with(ProgressLayer { sender: tx.clone() });
    tracing::subscriber::set_global_default(subscriber).ok();

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
        threed_state: ThreeDState::default(),
        skin: YomichanApp::create_skin(),
        show_ui: true,
    };

    loop {
        clear_background(BLACK);

        if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::H) {
            app.show_ui = !app.show_ui;
        }

        if app.router.c_route == Route::ThreeD {
            app.render_threed_scene();
        }

        if app.show_ui {
            root_ui().push_skin(&app.skin);
            app.draw_ui();
            root_ui().pop_skin();
        }

        next_frame().await;
    }
}
