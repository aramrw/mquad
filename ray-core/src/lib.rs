use ray_api::{RayExtension, RayContext, RayEventBus};
use anyhow::Result;

pub struct RayEngine {
    extensions: Vec<Box<dyn RayExtension>>,
    bus: RayEventBus,
    pub active_extension_idx: usize,
}

impl RayEngine {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            bus: RayEventBus::new(),
            active_extension_idx: 0,
        }
    }

    pub fn extension_names(&self) -> Vec<String> {
        self.extensions.iter().map(|e| e.name().to_string()).collect()
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

        // Drain the bus and dispatch to on_event
        while let Some(event) = self.bus.poll() {
            for ext in &mut self.extensions {
                ext.on_event(&mut ctx, &event)?;
            }
        }

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
        
        if let Some(ext) = self.extensions.get_mut(self.active_extension_idx) {
            ext.render(&mut ctx)?;
        }
        
        Ok(())
    }
}
