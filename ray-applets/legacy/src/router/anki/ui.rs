use macroquad::ui::{hash, widgets::ComboBox, Ui};
use crate::YomichanApp;
use yomichan_rs::settings::core::{AnkiTermFieldType, FieldIndex};

impl YomichanApp {
    pub fn draw_anki_tab(&mut self, ui: &mut Ui) {
        ui.label(None, "Anki Settings");
        if ui.button(None, "Sync Anki Maps") {
            let _ = self.yomichan.anki().update_all_anki_maps();
        }
        
        ui.separator();
        
        let anki = self.yomichan.anki();
        
        // Settings Section
        let decks = anki.deck_names();
        let models = anki.model_names();
        
        if models.is_empty() || decks.is_empty() {
            ui.label(None, "No decks or models found. Ensure Anki is open and AnkiConnect is installed, then Sync.");
        } else {
            let deck_refs: Vec<&str> = decks.iter().map(|s| s.as_str()).collect();
            let model_refs: Vec<&str> = models.iter().map(|s| s.as_str()).collect();

            ui.label(None, "Target Deck:");
            ComboBox::new(hash!("deck_combo"), &deck_refs).ui(ui, &mut self.anki_deck_idx);

            ui.label(None, "Target Note Model:");
            ComboBox::new(hash!("model_combo"), &model_refs).ui(ui, &mut self.anki_model_idx);

            ui.separator();
            ui.label(None, "Field Mappings:");

            let fields = anki.field_names(self.anki_model_idx);
            
            if !fields.is_empty() {
                let field_refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();

                ui.label(None, "Term Field:");
                ComboBox::new(hash!("field_term_combo"), &field_refs).ui(ui, &mut self.anki_field_term_idx);

                ui.label(None, "Reading Field:");
                ComboBox::new(hash!("field_reading_combo"), &field_refs).ui(ui, &mut self.anki_field_reading_idx);

                ui.label(None, "Definition Field:");
                ComboBox::new(hash!("field_def_combo"), &field_refs).ui(ui, &mut self.anki_field_def_idx);

                ui.label(None, "Sentence Field:");
                ComboBox::new(hash!("field_sentence_combo"), &field_refs).ui(ui, &mut self.anki_field_sentence_idx);
            }

            ui.separator();

            if ui.button(None, "Apply Configuration") {
                let _ = anki.select_deck(self.anki_deck_idx);
                let _ = anki.select_model(self.anki_model_idx);
                let _ = anki.set_field_mappings(&[
                    FieldIndex::Term(self.anki_field_term_idx),
                    FieldIndex::Reading(self.anki_field_reading_idx),
                    FieldIndex::Definition(self.anki_field_def_idx),
                    FieldIndex::Sentence(self.anki_field_sentence_idx),
                ]);
                let _ = self.yomichan.save_settings(); // Persist to Yomichan DB
                let _ = self.save_anki_settings_to_db(); // Persist UI state to Macroquad DB
            }
        }
        
        ui.separator();
        ui.separator();
        
        ui.label(None, "Offline Queue");
        
        if ui.button(None, "Sync Queue to Anki") {
            if let Ok(queue) = self.get_anki_queue() {
                for item in queue {
                    // Manual note construction mimicking AnkiConnect
                    let mut note_builder = anki_direct::notes::NoteBuilder::default();
                    
                    // We need the configured model and deck name
                    let mut model_name = String::new();
                    let mut deck_name = String::new();
                    let mut field_mappings = Vec::new();
                    
                    if let Ok(profile) = self.yomichan.options().read_arc().get_current_profile() {
                        let pg = profile.read();
                        let anki_opts = pg.anki_options();
                        if let Some(af) = anki_opts.anki_fields() {
                            field_mappings = af.fields().clone();
                            let global_opts = self.yomichan.options().read_arc();
                            let go = global_opts.anki().read();
                            if let Ok((m, _)) = go.get_selected_model(*af.selected_model()) {
                                model_name = m.to_string();
                            }
                            if let Ok((d, _)) = go.get_selected_deck(*af.selected_deck()) {
                                deck_name = d.to_string();
                            }
                        }
                    }
                    
                    if !model_name.is_empty() && !deck_name.is_empty() {
                        note_builder.model_name(model_name).deck_name(deck_name);
                        for mapping in &field_mappings {
                            match mapping {
                                AnkiTermFieldType::Term(f) => { note_builder.field(f, &item.headword); },
                                AnkiTermFieldType::Reading(f) => { note_builder.field(f, &item.reading); },
                                AnkiTermFieldType::Definition(f) => { note_builder.field(f, &item.definition); },
                                AnkiTermFieldType::Sentence(f) => { note_builder.field(f, &item.sentence); },
                                _ => {}
                            }
                        }
                        
                        if let Ok(note) = note_builder.build(Some(anki.client().read_arc().reqwest_client())) {
                            if let Ok(_) = anki.client().read().notes().add_notes(&[note]) {
                                let _ = self.delete_anki_queue_item(item.id);
                            }
                        }
                    }
                }
            }
        }
        
        ui.separator();
        
        if let Ok(queue) = self.get_anki_queue() {
            if queue.is_empty() {
                ui.label(None, "Queue is empty.");
            } else {
                for item in queue {
                    ui.label(None, format!("{} [{}]", item.headword, item.reading).as_str());
                    ui.same_line(0.0);
                    if ui.button(None, format!("Delete##{}", item.id).as_str()) {
                        let _ = self.delete_anki_queue_item(item.id);
                    }
                }
            }
        }
    }
}