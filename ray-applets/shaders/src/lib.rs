use ray_api::{RayExtension, RayContext, RayEvent, AudioEvent};
use macroquad::prelude::*;

pub struct ShaderApplet {
    last_volume: f32,
}

impl ShaderApplet {
    pub fn new() -> Self {
        Self {
            last_volume: 0.0,
        }
    }
}

impl RayExtension for ShaderApplet {
    fn name(&self) -> &str {
        "Shaders"
    }

    fn init(&mut self, _ctx: &mut RayContext, _args: &clap::ArgMatches) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_event(&mut self, _ctx: &mut RayContext, event: &RayEvent) -> anyhow::Result<()> {
        if let RayEvent::Audio(audio_ev) = event {
            match audio_ev {
                AudioEvent::Level(vol) => {
                    self.last_volume = *vol;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn render(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        let color = if self.last_volume > 0.05 { GREEN } else { DARKGRAY };
        draw_text(&format!("Modular Shader Applet - Volume: {:.4}", self.last_volume), 20.0, 100.0, 30.0, color);
        
        if self.last_volume > 0.01 {
             draw_circle(screen_width()/2.0, screen_height()/2.0, self.last_volume * 1000.0, color);
        }
        
        Ok(())
    }
}
