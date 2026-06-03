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
    Custom(Box<dyn Any + Send + Sync>),
}

pub enum InputEvent {
    KeyPressed(KeyCode),
    MousePressed(MouseButton, f32, f32),
}

pub enum AudioEvent {
    Level(f32),
    Buffer(Vec<f32>),
}

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
