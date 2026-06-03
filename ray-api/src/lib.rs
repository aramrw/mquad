use macroquad::prelude::*;
use std::any::Any;

pub trait RayExtension: Any {
    fn name(&self) -> &str;
    
    /// Called once when the applet is loaded.
    /// Passes CLI arguments relevant to this applet.
    fn init(&mut self, ctx: &mut RayContext, args: &clap::ArgMatches) -> anyhow::Result<()>;
    
    /// Logic update loop.
    fn update(&mut self, ctx: &mut RayContext) -> anyhow::Result<()>;
    
    /// Rendering loop.
    fn render(&mut self, ctx: &mut RayContext) -> anyhow::Result<()>;

    /// Event handler.
    fn on_event(&mut self, _ctx: &mut RayContext, _event: &RayEvent) -> anyhow::Result<()> {
        Ok(())
    }

    /// Render extension-specific settings UI.
    fn settings_ui(&mut self, _ctx: &mut RayContext, _ui: &mut macroquad::ui::Ui) -> anyhow::Result<()> {
        Ok(())
    }

    /// Does this extension have a settings page?
    fn has_settings(&self) -> bool {
        false
    }
    
    /// Cleanup before the applet is dropped.
    fn shutdown(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct RayContext<'a> {
    pub delta_time: f32,
    pub screen_size: (f32, f32),
    pub bus: &'a RayEventBus,
    // Add more shared state here as needed (input state, etc.)
}

pub enum RayEvent {
    Input(InputEvent),
    Audio(AudioEvent),
    Log(LogEvent),
    Command(RayCommand),
    Custom(Box<dyn Any + Send + Sync>),
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum RayCommand {
    Audio(AudioCommand),
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum AudioCommand {
    StartRecording,
    StopRecording,
    ToggleRecording,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEvent {
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub timestamp: f64,
}

impl LogEvent {
    pub fn level_str(&self) -> &'static str {
        self.level.as_str()
    }
}

pub enum InputEvent {
    KeyPressed(KeyCode),
    MousePressed(MouseButton, f32, f32),
}

pub enum AudioEvent {
    Level(f32),
    Buffer(Vec<f32>),
    Spectrum(Vec<f32>),
}

#[derive(Clone)]
pub struct RayEventBus {
    sender: crossbeam_channel::Sender<RayEvent>,
    receiver: crossbeam_channel::Receiver<RayEvent>,
}

impl RayEventBus {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }

    pub fn send(&self, event: RayEvent) {
        let _ = self.sender.send(event);
    }

    pub fn poll(&self) -> Option<RayEvent> {
        self.receiver.try_recv().ok()
    }
}
