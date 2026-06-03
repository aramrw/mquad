use ray_api::{RayExtension, RayContext, RayEventBus};
use anyhow::Result;

pub struct ExtensionEntry {
    pub instance: Box<dyn RayExtension>,
    pub enabled: bool,
}

pub struct RayEngine {
    extensions: Vec<ExtensionEntry>,
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

    pub fn extension_count(&self) -> usize {
        self.extensions.len()
    }

    pub fn get_extension_info(&self, index: usize) -> Option<(&str, bool)> {
        self.extensions.get(index).map(|e| (e.instance.name(), e.enabled))
    }

    pub fn toggle_extension(&mut self, index: usize) {
        if let Some(e) = self.extensions.get_mut(index) {
            e.enabled = !e.enabled;
        }
    }

    pub fn register<E: RayExtension + 'static>(&mut self, extension: E) {
        self.extensions.push(ExtensionEntry {
            instance: Box::new(extension),
            enabled: true,
        });
    }

    pub fn init(&mut self, args: &clap::ArgMatches) -> Result<()> {
        for entry in &mut self.extensions {
            let mut ctx = RayContext {
                delta_time: 0.0,
                screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
                bus: &self.bus,
            };
            entry.instance.init(&mut ctx, args)?;
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
            for entry in &mut self.extensions {
                if entry.enabled {
                    entry.instance.on_event(&mut ctx, &event)?;
                }
            }
        }

        for entry in &mut self.extensions {
            if entry.enabled {
                entry.instance.update(&mut ctx)?;
            }
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let mut ctx = RayContext {
            delta_time: 0.0,
            screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
            bus: &self.bus,
        };
        
        // Find the nth enabled extension based on active_extension_idx
        let mut current_enabled_count = 0;
        for entry in &mut self.extensions {
            if entry.enabled {
                if current_enabled_count == self.active_extension_idx {
                    entry.instance.render(&mut ctx)?;
                    break;
                }
                current_enabled_count += 1;
            }
        }
        
        Ok(())
    }

    pub fn enabled_extension_names(&self) -> Vec<String> {
        self.extensions.iter()
            .filter(|e| e.enabled)
            .map(|e| e.instance.name().to_string())
            .collect()
    }
}
