use ray_api::{RayExtension, RayContext, RayEventBus};
use anyhow::Result;

pub struct RayEngine {
    extensions: Vec<Box<dyn RayExtension>>,
    bus: RayEventBus,
}

impl RayEngine {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            bus: RayEventBus::new(),
        }
    }

    pub fn register<E: RayExtension + 'static>(&mut self, extension: E) {
        self.extensions.push(Box::new(extension));
    }

    pub fn init(&mut self, args: &clap::ArgMatches) -> Result<()> {
        for ext in &mut self.extensions {
            let mut ctx = RayContext {
                delta_time: 0.0,
                screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
                bus: &self.bus,
            };
            ext.init(&mut ctx, args)?;
        }
        Ok(())
    }

    pub fn update(&mut self, dt: f32) -> Result<()> {
        let mut ctx = RayContext {
            delta_time: dt,
            screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
            bus: &self.bus,
        };
        for ext in &mut self.extensions {
            ext.update(&mut ctx)?;
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let mut ctx = RayContext {
            delta_time: 0.0,
            screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
            bus: &self.bus,
        };
        for ext in &mut self.extensions {
            ext.render(&mut ctx)?;
        }
        Ok(())
    }
}
