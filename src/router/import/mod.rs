use macroquad::ui::Ui;
use crate::YomichanApp;

impl YomichanApp {
    pub fn draw_import_content(&mut self, ui: &mut Ui) {
        // Drain pending progress messages
        while let Ok(msg) = self.progress_receiver.try_recv() {
            self.import_status = msg;
        }

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
    }
}
