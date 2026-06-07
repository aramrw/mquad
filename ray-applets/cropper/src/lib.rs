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