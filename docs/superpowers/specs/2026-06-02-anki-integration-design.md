# Ray UI Design: Anki Integration

## Architecture
- A new `Route::Anki` tab added to the main router.
- Uses `rusqlite` to store a local offline queue table named `anki_queue` alongside settings.
- Leverages `yomichan_rs::anki` for all interactions with AnkiConnect.

## Offline Queue Storage (`anki_queue` table)
- A new table created in `db.ycd` via `rusqlite`.
- Schema:
  - `id` (INTEGER PRIMARY KEY)
  - `headword` (TEXT)
  - `reading` (TEXT)
  - `definition` (TEXT)
  - `sentence` (TEXT)
  - `added_at` (INTEGER)
- This avoids needing to serialize the complex `TermDictionaryEntry` directly, storing only the necessary strings for Note generation.

## Anki Tab View
The Anki tab consists of two main sections:

### 1. Settings Section
- **Sync Anki Button:** Triggers `ycd.anki().update_all_anki_maps()` to fetch the latest models and decks.
- **Deck & Model Selection:** Dropdowns populated by `ycd.anki().deck_names()` and `ycd.anki().model_names()`.
- **Field Mappings:** Dropdowns mapping internal values (Term, Reading, Definition, Sentence) to the selected Anki model's fields (`ycd.anki().field_names(idx)`).
- **Apply/Save:** Uses `ycd.anki().configure_note_creation()` to save the mappings to the `yomichan_rs` profile.

### 2. Offline Queue Section
- Displays all rows currently in the `anki_queue` table.
- **Sync to Anki Button:** Iterates over the queue. For each item:
  - Connects to AnkiConnect via `ycd.anki()`.
  - Builds a note manually using the stored string fields (mimicking `build_note_from_entry`).
  - Pushes the note via `add_notes`.
  - On success, deletes the row from the `anki_queue` table.
- **Manual Delete:** A button next to each queue item to remove it from the list without syncing.

## Search UI Modifications
- Update `CachedEntry` to store `sentence` (from the current search query) and its original `entry_index`.
- Add an **"[+] Add to Anki"** button next to each definition in the search results.
- Clicking the button instantly inserts the extracted term, reading, definition, and the searched sentence into the local `anki_queue` SQLite table.
