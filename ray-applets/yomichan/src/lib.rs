use ray_api::{RayExtension, RayContext};
use macroquad::prelude::*;

pub struct YomichanApplet {
    query: String,
}

impl YomichanApplet {
    pub fn new() -> Self {
        Self {
            query: String::new(),
        }
    }
}

impl RayExtension for YomichanApplet {
    fn name(&self) -> &str {
        "Yomichan"
    }

    fn init(&mut self, _ctx: &mut RayContext, _args: &clap::ArgMatches) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        draw_text("Modular Yomichan Applet", 20.0, 50.0, 30.0, WHITE);
        Ok(())
    }
}
