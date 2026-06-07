use macroquad::prelude::*;
use std::any::Any;

/// The core trait that all Ray extensions must implement.
pub trait RayExtension: Any {
    /// Returns the display name of the extension.
    fn name(&self) -> &str;
    
    /// Called once when the applet is loaded.
    /// Passes CLI arguments relevant to this applet.
    fn init(&mut self, ctx: &mut RayContext, args: &clap::ArgMatches) -> anyhow::Result<()>;
    
    /// Logic update loop, called every frame.
    fn update(&mut self, ctx: &mut RayContext) -> anyhow::Result<()>;
    
    /// Rendering loop, called every frame.
    fn render(&mut self, ctx: &mut RayContext) -> anyhow::Result<()>;

    /// Event handler for processing framework events.
    fn on_event(&mut self, _ctx: &mut RayContext, _event: &RayEvent) -> anyhow::Result<()> {
        Ok(())
    }

    /// Render extension-specific settings UI using Macroquad UI.
    fn settings_ui(&mut self, _ctx: &mut RayContext, _ui: &mut macroquad::ui::Ui) -> anyhow::Result<()> {
        Ok(())
    }

    /// Returns true if this extension provides a custom settings UI.
    fn has_settings(&self) -> bool {
        false
    }
    
    /// Cleanup logic called before the extension is unloaded.
    fn shutdown(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct RayContext<'a> {
    /// Time elapsed since the last frame in seconds.
    pub delta_time: f32,
    /// Current screen dimensions (width, height).
    pub screen_size: (f32, f32),
    /// Shared event bus for inter-extension communication.
    pub bus: &'a RayEventBus,
    /// Name of the applet currently using this context.
    pub applet_name: String,
    /// True if the applet is currently being rendered/focused.
    pub is_active: bool,
    /// Mutable reference to the system clipboard.
    pub clipboard: Option<&'a mut arboard::Clipboard>,
}

impl<'a> RayContext<'a> {
    /// Registers a new hotkey with the framework.
    pub fn register_hotkey(&self, definition: HotkeyDefinition) {
        self.bus.send(RayEvent::Command(RayCommand::RegisterHotkey(self.applet_name.clone(), definition)));
    }

    /// Sends a command to the framework.
    pub fn send_command(&self, command: RayCommand) {
        self.bus.send(RayEvent::Command(command));
    }

    /// Reads text from the system clipboard.
    pub fn clipboard_read(&mut self) -> Option<String> {
        self.clipboard.as_mut().and_then(|cb| cb.get_text().ok())
    }

    /// Writes text to the system clipboard.
    pub fn clipboard_write(&mut self, text: String) {
        if let Some(cb) = self.clipboard.as_mut() {
            if let Err(e) = cb.set_text(text) {
                tracing::error!("Clipboard write error: {}", e);
            }
        }
    }

    /// Writes an image to the system clipboard.
    pub fn clipboard_write_image(&mut self, width: usize, height: usize, rgba_pixels: &[u8]) {
        if let Some(cb) = self.clipboard.as_mut() {
            let image = arboard::ImageData {
                width,
                height,
                bytes: std::borrow::Cow::Borrowed(rgba_pixels),
            };
            if let Err(e) = cb.set_image(image) {
                tracing::error!("Clipboard image write error: {}", e);
            }
        }
    }
}

/// Defines the scope in which a hotkey is active.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum HotkeyScope {
    /// Active everywhere in the operating system.
    Global,
    /// Active when any part of this application is focused.
    OS,
    /// Active only when the specific extension is focused.
    Local,
}

use bitflags::bitflags;

bitflags! {
    /// Keyboard modifiers for hotkeys.
    #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
    #[serde(transparent)]
    pub struct HotkeyModifiers: u32 {
        const NONE = 0;
        const SHIFT = 1 << 0;
        const CTRL = 1 << 1;
        const ALT = 1 << 2;
        const LOGO = 1 << 3;
    }
}

impl Default for HotkeyModifiers {
    fn default() -> Self {
        Self::NONE
    }
}

/// A complete hotkey definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HotkeyDefinition {
    /// Unique identifier for this hotkey.
    pub id: String,
    /// Key name (e.g., "Space", "F1", "A").
    pub key: String, 
    /// Set of modifiers required for this hotkey.
    pub modifiers: HotkeyModifiers,
    /// Scope of the hotkey.
    pub scope: HotkeyScope,
    /// Human-readable description.
    pub description: String,
    /// Internal resolved keycode (not serialized).
    #[serde(skip)]
    pub internal_keycode: Option<KeyCode>,
}

use std::path::PathBuf;

/// The type of data contained in a payload.
#[derive(Clone, Debug)]
pub enum PayloadData {
    File(PathBuf),
    Buffer(Vec<u8>),
    Text(String),
}

/// A message passed between extensions (nodes).
#[derive(Clone, Debug)]
pub struct PayloadMessage {
    pub source: String,
    pub target: String,
    pub data: PayloadData,
}

/// Events dispatched through the Ray framework.
pub enum RayEvent {
    /// Keyboard or mouse input event.
    Input(InputEvent),
    /// Audio-related event (e.g., levels, buffers).
    Audio(AudioEvent),
    /// System or extension log event.
    Log(LogEvent),
    /// Command sent to the framework or other extensions.
    Command(RayCommand),
    /// Fired when a registered hotkey is triggered.
    /// Contains the ID of the hotkey.
    HotkeyTriggered(String),
    /// Custom event for extension-specific communication.
    Custom(Box<dyn Any + Send + Sync>),
    /// Payload sent between nodes.
    Payload(PayloadMessage),
}

/// Commands that can be issued to the framework.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum RayCommand {
    /// Audio control commands.
    Audio(AudioCommand),
    /// Registers a new hotkey with the framework.
    /// (applet_name, definition)
    RegisterHotkey(String, HotkeyDefinition),
    /// Toggles the framework overlay mode.
    ToggleOverlay(bool),
    /// Toggles the framework mini mode (shrunk window).
    MiniMode(bool),
    /// Switches the active extension to the one with the given name.
    SelectExtension(String),
}

/// Audio system control commands.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum AudioCommand {
    /// Start recording audio from the default input device.
    StartRecording,
    /// Stop current audio recording.
    StopRecording,
    /// Toggle between recording and stopped states.
    ToggleRecording,
}

/// Severity level for log messages.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    /// Returns the standard string representation of the log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN ",
            LogLevel::Info => "INFO ",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

/// A structured log event.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEvent {
    /// Log severity.
    pub level: LogLevel,
    /// Target module or component name.
    pub target: String,
    /// The log message content.
    pub message: String,
    /// Unix timestamp or frame time.
    pub timestamp: f64,
}

impl LogEvent {
    /// Convenience method to get the level string.
    pub fn level_str(&self) -> &'static str {
        self.level.as_str()
    }
}

/// Low-level input events.
pub enum InputEvent {
    /// A keyboard key was pressed.
    KeyPressed(KeyCode),
    /// A mouse button was pressed at specific coordinates.
    MousePressed(MouseButton, f32, f32),
}

/// Audio data events.
pub enum AudioEvent {
    /// Current peak audio level (0.0 to 1.0).
    Level(f32),
    /// Raw PCM audio buffer.
    Buffer(Vec<f32>),
    /// Frequency spectrum data.
    Spectrum(Vec<f32>),
    /// Broadcasts the currently selected device index.
    DeviceSelected(i32),
}

/// A thread-safe event bus for communication within the Ray framework.
#[derive(Clone)]
pub struct RayEventBus {
    sender: crossbeam_channel::Sender<RayEvent>,
    receiver: crossbeam_channel::Receiver<RayEvent>,
}

impl RayEventBus {
    /// Creates a new unbounded event bus.
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }

    /// Sends an event to the bus.
    pub fn send(&self, event: RayEvent) {
        let _ = self.sender.send(event);
    }

    /// Attempts to receive a single event from the bus without blocking.
    pub fn poll(&self) -> Option<RayEvent> {
        self.receiver.try_recv().ok()
    }
}
