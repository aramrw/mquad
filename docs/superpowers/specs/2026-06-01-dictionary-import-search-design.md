# Ray UI Design: Dictionary Import & Search

## Architecture
- A tabbed UI using `macroquad::ui`.
- Two main tabs/views: **Search** (default) and **Import**.
- Handled via `Route::Search` and `Route::Import` enum variants.

## Dictionary Import & Progress Tracking
- Users switch to the Import tab and click a "Select Dictionary" button.
- A native file dialog opens using `rfd::FileDialog`.
- On selection, a background thread is spawned using `std::thread::spawn` to call `ycd.import_dictionaries(&[path])`.
- To avoid blocking the UI, `Yomichan` instance should be wrapped in an `Arc<RwLock<Yomichan>>` or we use message passing to send status updates back to the main thread. 
- **Tracing / Loading Bar**: Since `yomichan_importer` emits `tracing::info!` messages (e.g., "Processing X tag banks...", "Processing term bank: ..."), we will set up a custom `tracing_subscriber` (e.g., a `tracing::subscriber::Default` with a custom layer) that intercepts these `INFO` messages.
- These intercepted messages will be passed to the main thread via an `std::sync::mpsc::channel`, allowing the macroquad UI to draw a progress bar and label (e.g., `Processing term bank: term_bank_1.json...`).

## Dictionary Search
- Users enter text in a text box.
- A "Search" button (or hitting enter) triggers the lookup: `ycd.search(&query)`.
- The results are displayed in a scrollable `macroquad::ui::widgets::Window`.
- Displays segment by segment, listing headwords and definitions elegantly.
