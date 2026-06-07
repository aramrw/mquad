use ray_api::{RayExtension, RayContext, RayEvent, PayloadMessage, PayloadData};
use std::process::Command;
use macroquad::prelude::*;
use std::path::PathBuf;

pub struct TrimmerApplet {
    start_time: String,
    duration: String,
    status: String,
}

impl TrimmerApplet {
    pub fn new() -> Self {
        Self {
            start_time: "00:00:00".to_string(),
            duration: "15".to_string(),
            status: "Idle".to_string(),
        }
    }
}

impl RayExtension for TrimmerApplet {
    fn name(&self) -> &'static str {
        "Trimmer Node"
    }

    fn init(&mut self, _ctx: &mut RayContext, _args: &clap::ArgMatches) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_event(&mut self, ctx: &mut RayContext, event: &RayEvent) -> anyhow::Result<()> {
        if let RayEvent::Payload(msg) = event {
            if msg.target == "Trimmer" {
                if let PayloadData::File(ref path) = msg.data {
                    self.status = format!("Trimming {:?}", path);
                    
                    // Output to user's desktop for the final file
                    let desktop_path = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    let out_path = PathBuf::from(desktop_path)
                        .join("Desktop")
                        .join(format!("final_{}", path.file_name().unwrap_or_default().to_string_lossy()));
                    
                    let start = self.start_time.clone();
                    let dur = self.duration.clone();
                    
                    let status = Command::new("ffmpeg")
                        .arg("-y") // Overwrite
                        .arg("-i")
                        .arg(path)
                        .arg("-ss")
                        .arg(&start)
                        .arg("-t")
                        .arg(&dur)
                        .arg("-c")
                        .arg("copy") // Stream copy for speed
                        .arg(&out_path)
                        .status();
                        
                    if status.is_ok() && status.unwrap().success() {
                        self.status = format!("Saved to Desktop");
                        // We could emit a broadcast payload here if needed:
                        // target: "*", data: PayloadData::File(out_path)
                    } else {
                        self.status = "Failed".to_string();
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> { Ok(()) }

    fn render(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        use macroquad::ui::{root_ui, hash};
        root_ui().window(hash!("trimmer_node"), vec2(100.0, 320.0), vec2(300.0, 200.0), |ui| {
            ui.label(None, "Trimmer Node");
            ui.input_text(hash!("trim_start"), "Start Time", &mut self.start_time);
            ui.input_text(hash!("trim_dur"), "Duration (s)", &mut self.duration);
            ui.label(None, &format!("Status: {}", self.status));
        });
        Ok(())
    }
}