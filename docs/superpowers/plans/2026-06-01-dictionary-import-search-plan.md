# Dictionary Import & Search Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement dictionary import functionality and a functional search interface in the Ray macroquad UI.

**Architecture:** We will use `Arc<Yomichan>` to share the application logic between the main thread and a background import thread. Progress messages will be communicated to the main thread via an `mpsc` channel. Tracing events will be intercepted by a custom `tracing` visitor/layer to capture `INFO` logs as progress text. A simple tabbed interface will let the user switch between "Search" and "Import" views.

**Tech Stack:** Rust, macroquad, yomichan_rs, rfd, tracing.

---

### Task 1: Update Router and App State

**Files:**
- Modify: `src/router/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add Import route**

Update `src/router/mod.rs` to include an `Import` variant:

```rust
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
```

- [ ] **Step 2: Update App State**

Update `src/main.rs` imports and the `YomichanApp` struct. We'll use an `Arc<yomichan_rs::Yomichan>`, strings for search/status, and channels for progress reporting.

```rust
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
```

- [ ] **Step 3: Fix `draw_ui` logic**

Make sure `draw_ui` matches both `Route::Search` and `Route::Import`. Replace the existing `search_btn()` with a new `nav_buttons()` method inside the `impl YomichanApp`.

```rust
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
```
*(You will need to temporarily remove or replace the old `search_results` body so it doesn't fail compilation)*

- [ ] **Step 4: Check compilation**

Run: `cargo check`
Expected: `YomichanApp::new` will fail if you left it in. You can remove `YomichanApp::new` since we'll initialize manually in Task 3. Comment out `YomichanApp::new` in `main` for now.

### Task 2: Setup Tracing Subscriber for Progress Updates

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement custom tracing layer**

Put this somewhere above `YomichanApp` in `src/main.rs`.

```rust
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::Registry;
use tracing_core::{Subscriber, Event};
use tracing::field::{Visit, Field};

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
```

- [ ] **Step 2: Register subscriber in main**

Update the `main` function to register this layer and correctly instantiate `YomichanApp`.

```rust
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
```

- [ ] **Step 3: Check compilation**

Run: `cargo check`
Expected: Should compile successfully.

### Task 3: Implement Import UI and Background Threading

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Update `import_results` method**

This adds the file dialog and thread spawning. Replace the empty `import_results` method.

```rust
    fn import_results(&mut self) {
        use macroquad::ui::widgets::Window;
        use macroquad::ui::hash;

        // Drain pending progress messages
        while let Ok(msg) = self.progress_receiver.try_recv() {
            self.import_status = msg;
        }

        Window::new(hash!(), vec2(10., 70.), vec2(400., 400.))
            .label("Import Dictionary")
            .ui(&mut root_ui(), |ui| {
                ui.label(None, "Select a Yomitan .zip dictionary file:");
                
                if ui.button(None, "Open File Dialog") {
                    if let Some(path) = rfd::FileDialog::new().add_filter("zip", &["zip"]).pick_file() {
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
```

- [ ] **Step 2: Check compilation**

Run: `cargo check`
Expected: Should compile successfully.

### Task 4: Implement Search UI

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace `search_results` with real search UI**

Update the `search_results` function in `YomichanApp` to accept input and perform searches using the `yomichan` instance.

```rust
    fn search_results(&mut self) {
        use macroquad::ui::widgets::{Window, InputText};
        use macroquad::ui::hash;

        Window::new(hash!(), vec2(10., 70.), vec2(400., 400.))
            .label("Dictionary Search")
            .ui(&mut root_ui(), |ui| {
                ui.label(None, "Enter Japanese text:");
                InputText::new(hash!()).ui(ui, &mut self.search_query);

                if ui.button(None, "Search") && !self.search_query.is_empty() {
                    println!("Searching for: {}", self.search_query);
                }
                
                ui.separator();

                if let Ok(res) = self.yomichan.search(&self.search_query) {
                    for segment in res.segments {
                        if segment.entries.is_empty() {
                            ui.label(None, &format!("No results for: {}", segment.text));
                            continue;
                        }
                        for entry in segment.entries {
                            let headword_str = entry.headwords
                                .iter()
                                .map(|h| format!("{} ({})", h.term.clone(), h.reading.clone()))
                                .collect::<Vec<_>>()
                                .join(", ");
                            ui.label(None, &headword_str);
                            
                            for def in entry.definitions {
                                for group in def.entries {
                                    ui.label(None, &format!("- {}", group.plain_text));
                                }
                            }
                            ui.separator();
                        }
                    }
                }
            });
    }
```

- [ ] **Step 2: Run Application**

Run: `cargo run`
Expected: App runs, you can switch tabs, open file dialog in Import, import a zip file, see progress from tracing output in the UI, and search Japanese text in the Search tab to see results rendered.
