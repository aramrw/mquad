use ray_api::{RayExtension, RayContext, RayEventBus, RayEvent, LogEvent, LogLevel};
use anyhow::Result;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

pub struct ExtensionEntry {
    pub instance: Box<dyn RayExtension>,
    pub enabled: bool,
}

#[derive(Default)]
pub struct HotkeyRegistry {
    /// Maps (applet_name, hotkey_id) -> HotkeyDefinition
    pub registered: std::collections::HashMap<(String, String), ray_api::HotkeyDefinition>,
    /// Cache of conflicting hotkey pairs: ((applet1, id1), (applet2, id2))
    pub conflicts: Vec<((String, String), (String, String))>,
    /// Flag indicating that the registry has changed and needs sync/re-validation
    pub dirty: bool,
}

impl HotkeyRegistry {
    pub fn update_conflicts(&mut self) {
        self.conflicts.clear();
        let mut key_usage: std::collections::HashMap<(String, ray_api::HotkeyModifiers), Vec<(String, String)>> = std::collections::HashMap::new();
        
        for ((applet, id), def) in &self.registered {
            key_usage.entry((def.key.clone(), def.modifiers)).or_default().push((applet.clone(), id.clone()));
        }

        for usage in key_usage.values() {
            if usage.len() > 1 {
                // Generate pairs for conflicts
                for i in 0..usage.len() {
                    for j in i + 1..usage.len() {
                        self.conflicts.push((usage[i].clone(), usage[j].clone()));
                    }
                }
            }
        }
    }
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
    pub hotkey_registry: HotkeyRegistry,
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
            hotkey_registry: HotkeyRegistry::default(),
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
        conn.execute(
            "CREATE TABLE IF NOT EXISTS hotkey_overrides (
                applet_name TEXT,
                hotkey_id TEXT,
                key_code TEXT,
                modifiers INTEGER,
                PRIMARY KEY (applet_name, hotkey_id)
            )",
            [],
        )?;
        Ok(())
    }

    pub fn register_hotkey(&mut self, applet_name: String, mut definition: ray_api::HotkeyDefinition) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT key_code, modifiers FROM hotkey_overrides WHERE applet_name = ?1 AND hotkey_id = ?2")?;
        let mut rows = stmt.query(rusqlite::params![applet_name, definition.id])?;

        if let Some(row) = rows.next()? {
            definition.key = row.get(0)?;
            let mods: u32 = row.get(1)?;
            definition.modifiers = ray_api::HotkeyModifiers::from_bits_retain(mods);
        }

        // Cache the keycode for faster polling
        definition.internal_keycode = string_to_keycode(&definition.key);

        self.hotkey_registry.registered.insert((applet_name, definition.id.clone()), definition);
        self.hotkey_registry.dirty = true;
        self.hotkey_registry.update_conflicts();
        Ok(())
    }

    pub fn is_hotkey_registry_dirty(&self) -> bool {
        self.hotkey_registry.dirty
    }

    pub fn clear_hotkey_registry_dirty(&mut self) {
        self.hotkey_registry.dirty = false;
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
        if let Some(entry) = self.extensions.get_mut(index) {
            let mut ctx = RayContext {
                delta_time: 0.0,
                screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
                bus: &self.bus,
                applet_name: entry.instance.name().to_string(),
            };
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
                applet_name: entry.instance.name().to_string(),
            };
            entry.instance.init(&mut ctx, args)?;
        }
        Ok(())
    }

    pub fn update(&mut self, dt: f32) -> Result<()> {
        // Poll Local hotkeys
        let active_applet = self.get_active_applet_name();
        let mut triggered_ids = Vec::new();
        for ((applet_name, hotkey_id), def) in &self.hotkey_registry.registered {
            if def.scope == ray_api::HotkeyScope::Local {
                if let Some(active) = &active_applet {
                    if active == applet_name {
                        if is_hotkey_pressed(def) {
                            triggered_ids.push(hotkey_id.clone());
                        }
                    }
                }
            }
        }
        for id in triggered_ids {
            self.bus.send(RayEvent::HotkeyTriggered(id));
        }

        // Drain the bus and dispatch to on_event
        while let Some(event) = self.bus.poll() {
            // Internal framework handling of commands
            if let RayEvent::Command(ray_api::RayCommand::RegisterHotkey(applet_name, definition)) = &event {
                if let Err(e) = self.register_hotkey(applet_name.clone(), definition.clone()) {
                    tracing::error!("Failed to register hotkey: {}", e);
                }
            }

            for entry in &mut self.extensions {
                if entry.enabled {
                    let mut ctx = RayContext {
                        delta_time: dt,
                        screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
                        bus: &self.bus,
                        applet_name: entry.instance.name().to_string(),
                    };
                    if let Err(e) = entry.instance.on_event(&mut ctx, &event) {
                        tracing::error!("[{}] on_event error: {}", entry.instance.name(), e);
                    }
                }
            }
        }

        for entry in &mut self.extensions {
            if entry.enabled {
                let mut ctx = RayContext {
                    delta_time: dt,
                    screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
                    bus: &self.bus,
                    applet_name: entry.instance.name().to_string(),
                };
                if let Err(e) = entry.instance.update(&mut ctx) {
                    tracing::error!("[{}] update error: {}", entry.instance.name(), e);
                }
            }
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        // Find the nth enabled extension based on active_extension_idx
        let mut current_enabled_count = 0;
        for entry in &mut self.extensions {
            if entry.enabled {
                if current_enabled_count == self.active_extension_idx {
                    let mut ctx = RayContext {
                        delta_time: 0.0,
                        screen_size: (macroquad::window::screen_width(), macroquad::window::screen_height()),
                        bus: &self.bus,
                        applet_name: entry.instance.name().to_string(),
                    };
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

    pub fn get_active_applet_name(&self) -> Option<String> {
        let mut current_enabled_count = 0;
        for entry in &self.extensions {
            if entry.enabled {
                if current_enabled_count == self.active_extension_idx {
                    return Some(entry.instance.name().to_string());
                }
                current_enabled_count += 1;
            }
        }
        None
    }
}

/// Converts a string representation of a key to a macroquad KeyCode.
/// Returns None if the key name is not recognized.
fn string_to_keycode(key: &str) -> Option<macroquad::input::KeyCode> {
    use macroquad::input::KeyCode;
    match key.to_uppercase().as_str() {
        "A" => Some(KeyCode::A),
        "B" => Some(KeyCode::B),
        "C" => Some(KeyCode::C),
        "D" => Some(KeyCode::D),
        "E" => Some(KeyCode::E),
        "F" => Some(KeyCode::F),
        "G" => Some(KeyCode::G),
        "H" => Some(KeyCode::H),
        "I" => Some(KeyCode::I),
        "J" => Some(KeyCode::J),
        "K" => Some(KeyCode::K),
        "L" => Some(KeyCode::L),
        "M" => Some(KeyCode::M),
        "N" => Some(KeyCode::N),
        "O" => Some(KeyCode::O),
        "P" => Some(KeyCode::P),
        "Q" => Some(KeyCode::Q),
        "R" => Some(KeyCode::R),
        "S" => Some(KeyCode::S),
        "T" => Some(KeyCode::T),
        "U" => Some(KeyCode::U),
        "V" => Some(KeyCode::V),
        "W" => Some(KeyCode::W),
        "X" => Some(KeyCode::X),
        "Y" => Some(KeyCode::Y),
        "Z" => Some(KeyCode::Z),
        "0" => Some(KeyCode::Key0),
        "1" => Some(KeyCode::Key1),
        "2" => Some(KeyCode::Key2),
        "3" => Some(KeyCode::Key3),
        "4" => Some(KeyCode::Key4),
        "5" => Some(KeyCode::Key5),
        "6" => Some(KeyCode::Key6),
        "7" => Some(KeyCode::Key7),
        "8" => Some(KeyCode::Key8),
        "9" => Some(KeyCode::Key9),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        "SPACE" => Some(KeyCode::Space),
        "ENTER" => Some(KeyCode::Enter),
        "ESCAPE" => Some(KeyCode::Escape),
        "TAB" => Some(KeyCode::Tab),
        "BACKSPACE" => Some(KeyCode::Backspace),
        "INSERT" => Some(KeyCode::Insert),
        "DELETE" => Some(KeyCode::Delete),
        "RIGHT" => Some(KeyCode::Right),
        "LEFT" => Some(KeyCode::Left),
        "DOWN" => Some(KeyCode::Down),
        "UP" => Some(KeyCode::Up),
        "PAGEUP" => Some(KeyCode::PageUp),
        "PAGEDOWN" => Some(KeyCode::PageDown),
        "HOME" => Some(KeyCode::Home),
        "END" => Some(KeyCode::End),
        _ => None,
    }
}

/// Checks if a hotkey is currently being pressed.
/// This function performs strict modifier checking, ensuring that ONLY the
/// specified modifiers are pressed alongside the target key.
fn is_hotkey_pressed(def: &ray_api::HotkeyDefinition) -> bool {
    if let Some(keycode) = def.internal_keycode {
        if macroquad::input::is_key_pressed(keycode) {
            let mods = def.modifiers;
            let shift = macroquad::input::is_key_down(macroquad::input::KeyCode::LeftShift) || macroquad::input::is_key_down(macroquad::input::KeyCode::RightShift);
            let ctrl = macroquad::input::is_key_down(macroquad::input::KeyCode::LeftControl) || macroquad::input::is_key_down(macroquad::input::KeyCode::RightControl);
            let alt = macroquad::input::is_key_down(macroquad::input::KeyCode::LeftAlt) || macroquad::input::is_key_down(macroquad::input::KeyCode::RightAlt);
            let logo = macroquad::input::is_key_down(macroquad::input::KeyCode::LeftSuper) || macroquad::input::is_key_down(macroquad::input::KeyCode::RightSuper);

            if mods.contains(ray_api::HotkeyModifiers::SHIFT) != shift { return false; }
            if mods.contains(ray_api::HotkeyModifiers::CTRL) != ctrl { return false; }
            if mods.contains(ray_api::HotkeyModifiers::ALT) != alt { return false; }
            if mods.contains(ray_api::HotkeyModifiers::LOGO) != logo { return false; }

            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use ray_api::*;

    #[test]
    fn test_hotkey_registration_and_override() -> Result<()> {
        let db_path = "test_hotkeys.db";
        let _ = std::fs::remove_file(db_path);
        let mut engine = RayEngine::new(db_path);

        let def = HotkeyDefinition {
            id: "test_hotkey".to_string(),
            key: "A".to_string(),
            modifiers: HotkeyModifiers::CTRL,
            scope: HotkeyScope::Global,
            description: "Test".to_string(),
            internal_keycode: None,
        };

        // 1. Register without override
        engine.register_hotkey("test_app".to_string(), def.clone())?;
        let registered = engine.hotkey_registry.registered.get(&("test_app".to_string(), "test_hotkey".to_string())).unwrap();
        assert_eq!(registered.key, "A");
        assert_eq!(registered.modifiers, HotkeyModifiers::CTRL);

        // 2. Add override to DB
        {
            let conn = rusqlite::Connection::open(db_path)?;
            conn.execute(
                "INSERT INTO hotkey_overrides (applet_name, hotkey_id, key_code, modifiers) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["test_app", "test_hotkey", "B", HotkeyModifiers::SHIFT.bits()],
            )?;
        }

        // 3. Register again, should pick up override
        engine.register_hotkey("test_app".to_string(), def.clone())?;
        let registered = engine.hotkey_registry.registered.get(&("test_app".to_string(), "test_hotkey".to_string())).unwrap();
        assert_eq!(registered.key, "B");
        assert_eq!(registered.modifiers, HotkeyModifiers::SHIFT);

        let _ = std::fs::remove_file(db_path);
        Ok(())
    }
}

