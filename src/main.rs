mod router;
mod tracing_utils;

use macroquad::main;
use macroquad::prelude::*;
use std::sync::{Arc, mpsc};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use yomichan_rs::Yomichan;

use crate::router::Router;
use crate::router::Route;
use crate::tracing_utils::ProgressLayer;

pub struct CachedEntry {
    pub headword: String,
    pub definitions: Vec<String>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ThreeDObject {
    Cube,
    Sphere,
    Pyramid,
}

pub struct ThreeDState {
    pub selected_shape: ThreeDObject,
    pub rotation: f32,
}

impl Default for ThreeDState {
    fn default() -> Self {
        Self {
            selected_shape: ThreeDObject::Cube,
            rotation: 0.0,
        }
    }
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
                .map(|h| format!("{} ({})", h.term, h.reading))
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

    pub fn render_threed_scene(&mut self) {
        self.threed_state.rotation += 0.02;
        let x = self.threed_state.rotation.cos() * 5.0;
        let z = self.threed_state.rotation.sin() * 5.0;

        set_camera(&Camera3D {
            position: vec3(x, 3.0, z),
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            ..Default::default()
        });

        draw_grid(10, 1.0, GREEN, GRAY);

        match self.threed_state.selected_shape {
            ThreeDObject::Cube => {
                draw_cube(vec3(0.0, 1.0, 0.0), vec3(2.0, 2.0, 2.0), None, WHITE);
                draw_cube_wires(vec3(0.0, 1.0, 0.0), vec3(2.0, 2.0, 2.0), MAROON);
            }
            ThreeDObject::Sphere => {
                draw_sphere(vec3(0.0, 1.0, 0.0), 1.0, None, BLUE);
                draw_sphere_wires(vec3(0.0, 1.0, 0.0), 1.0, None, SKYBLUE);
            }
            ThreeDObject::Pyramid => {
                let top = vec3(0.0, 2.0, 0.0);
                let b1 = vec3(-1.0, 0.0, -1.0);
                let b2 = vec3(1.0, 0.0, -1.0);
                let b3 = vec3(1.0, 0.0, 1.0);
                let b4 = vec3(-1.0, 0.0, 1.0);

                draw_line_3d(top, b1, YELLOW);
                draw_line_3d(top, b2, YELLOW);
                draw_line_3d(top, b3, YELLOW);
                draw_line_3d(top, b4, YELLOW);
                draw_line_3d(b1, b2, ORANGE);
                draw_line_3d(b2, b3, ORANGE);
                draw_line_3d(b3, b4, ORANGE);
                draw_line_3d(b4, b1, ORANGE);
            }
        }

        set_default_camera();
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
    };

    loop {
        clear_background(BLACK);
        
        if app.router.c_route == Route::ThreeD {
            app.render_threed_scene();
        }

        app.draw_ui();
        next_frame().await;
    }
}
