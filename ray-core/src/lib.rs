use ray_api::{RayExtension, RayContext, RayEventBus, RayEvent, LogEvent, LogLevel};
use anyhow::Result;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

pub struct ExtensionEntry {
    pub instance: Box<dyn RayExtension>,
    pub enabled: bool,
}

pub struct RayLogLayer {
    bus: RayEventBus,
}

impl RayLogLayer {
    pub fn new(bus: RayEventBus) -> Self {
        Self { bus }
    }
}

impl<S> Layer<S> for RayLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut message = String::new();
        let mut visitor = RayLogVisitor { message: &mut message };
        event.record(&mut visitor);

        let level = match *event.metadata().level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Trace,
        };

        self.bus.send(RayEvent::Log(LogEvent {
            level,
            target: event.metadata().target().to_string(),
            message,
            timestamp: macroquad::time::get_time(),
        }));
    }
}

struct RayLogVisitor<'a> {
    message: &'a mut String,
}

impl<'a> tracing::field::Visit for RayLogVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            use std::fmt::Write;
            let _ = write!(self.message, "{:?}", value);
        }
    }
}

pub struct RayEngine {
    extensions: Vec<ExtensionEntry>,
    bus: RayEventBus,
    pub active_extension_idx: usize,
    db_path: String,
}

impl RayEngine {
    pub fn new(db_path: &str) -> Self {
        let bus = RayEventBus::new();
        
        // Initialize global tracing subscriber
        use tracing_subscriber::prelude::*;
        let log_layer = RayLogLayer::new(bus.clone());
        tracing_subscriber::registry()
            .with(log_layer)
            .init();

        let engine = Self {
            extensions: Vec::new(),
            bus,
            active_extension_idx: 0,
            db_path: db_path.to_string(),
        };
        engine.ensure_db_schema().ok();
        engine
    }

    fn ensure_db_schema(&self) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS extension_settings (
                name TEXT PRIMARY KEY,
                enabled INTEGER
            )",
            [],
        )?;
        Ok(())
    }

    pub fn load_settings(&mut self) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT name, enabled FROM extension_settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)? != 0))
        })?;

        let settings: std::collections::HashMap<String, bool> = rows
            .flatten()
            .collect();

        for entry in &mut self.extensions {
            if let Some(&enabled) = settings.get(entry.instance.name()) {
                entry.enabled = enabled;
            }
        }
        Ok(())
    }

    pub fn save_extension_state(&self, name: &str, enabled: bool) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT OR REPLACE INTO extension_settings (name, enabled) VALUES (?1, ?2)",
            rusqlite::params![name, if enabled { 1 } else { 0 }],
        )?;
        Ok(())
    }

    pub fn bus(&self) -> &RayEventBus {
        &self.bus
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
            let name = e.instance.name().to_string();
            let enabled = e.enabled;
            let _ = self.save_extension_state(&name, enabled);
        }
    }

    pub fn extension_has_settings(&self, index: usize) -> bool {
        self.extensions.get(index).map_or(false, |e| e.instance.has_settings())
    }

    pub fn render_extension_settings(&mut self, index: usize, ui: &mut macroquad::ui::Ui) -> Result<()> {
        let mut ctx = RayContext {
            delta_time: 0.0,
            screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
            bus: &self.bus,
        };
        
        if let Some(entry) = self.extensions.get_mut(index) {
            entry.instance.settings_ui(&mut ctx, ui)?;
        }
        Ok(())
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
                    if let Err(e) = entry.instance.on_event(&mut ctx, &event) {
                        tracing::error!("[{}] on_event error: {}", entry.instance.name(), e);
                    }
                }
            }
        }

        for entry in &mut self.extensions {
            if entry.enabled {
                if let Err(e) = entry.instance.update(&mut ctx) {
                    tracing::error!("[{}] update error: {}", entry.instance.name(), e);
                }
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
                    if let Err(e) = entry.instance.render(&mut ctx) {
                        tracing::error!("[{}] render error: {}", entry.instance.name(), e);
                        macroquad::prelude::draw_text(
                            &format!("Applet Error: {}", e),
                            20.0, 50.0, 20.0, macroquad::prelude::RED
                        );
                    }
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
