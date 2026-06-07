use ray_api::{RayExtension, RayContext, RayEvent, PayloadMessage, PayloadData};
use std::process::Command;
use macroquad::prelude::*;

pub struct CropperApplet {
    target_width: String,
    target_height: String,
    is_processing: bool,
    status: String,
    target_node: String,
}

impl CropperApplet {
    pub fn new() -> Self {
        Self {
            target_width: "1048".to_string(),
            target_height: "1048".to_string(),
            is_processing: false,
            status: "Idle".to_string(),
            target_node: "Trimmer".to_string(),
        }
    }
}

impl RayExtension for CropperApplet {
    fn name(&self) -> &'static str {
        "Cropper Node"
    }

    fn init(&mut self, _ctx: &mut RayContext, _args: &clap::ArgMatches) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_event(&mut self, ctx: &mut RayContext, event: &RayEvent) -> anyhow::Result<()> {
        if let RayEvent::Payload(msg) = event {
            if msg.target == "Cropper" {
                if let PayloadData::File(ref path) = msg.data {
                    self.is_processing = true;
                    self.status = format!("Cropping {:?}", path);
                    
                    let out_path = std::env::temp_dir().join(format!("cropped_{}", path.file_name().unwrap_or_default().to_string_lossy()));
                    
                    let w = self.target_width.clone();
                    let h = self.target_height.clone();
                    
                    // Simple synchronous ffmpeg call for MVP
                    let status = Command::new("ffmpeg")
                        .arg("-y") // Overwrite
                        .arg("-i")
                        .arg(path)
                        .arg("-vf")
                        .arg(format!("crop={}:{}", w, h))
                        .arg(&out_path)
                        .status();
                        
                    self.is_processing = false;
                    
                    if status.is_ok() && status.unwrap().success() {
                        self.status = "Complete".to_string();
                        // Emit payload to the next node
                        ctx.bus.send(RayEvent::Payload(PayloadMessage {
                            source: "Cropper".to_string(),
                            target: self.target_node.clone(),
                            data: PayloadData::File(out_path),
                        }));
                    } else {
                        self.status = "Failed".to_string();
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        use macroquad::ui::{root_ui, hash};
        root_ui().window(hash!("cropper_node"), vec2(100.0, 100.0), vec2(300.0, 200.0), |ui| {
            ui.label(None, "Cropper Node");
            ui.input_text(hash!("crop_w"), "Width", &mut self.target_width);
            ui.input_text(hash!("crop_h"), "Height", &mut self.target_height);
            ui.input_text(hash!("target_node"), "Send To", &mut self.target_node);
            ui.label(None, &format!("Status: {}", self.status));
        });
        Ok(())
    }
}