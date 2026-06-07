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