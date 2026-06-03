use ray_api::{RayExtension, RayContext, RayEvent, HotkeyDefinition, HotkeyScope, HotkeyModifiers, RayCommand};
use macroquad::prelude::*;
use anyhow::Result;

pub struct CaptureApplet {
    active: bool,
    snapshot: Option<Texture2D>,
    selection_start: Option<Vec2>,
    selection_end: Option<Vec2>,
}

impl CaptureApplet {
    pub fn new() -> Self {
        Self {
            active: false,
            snapshot: None,
            selection_start: None,
            selection_end: None,
        }
    }

    fn capture_screenshot(&mut self) -> Result<()> {
        let screens = xcap::Monitor::all()?;
        if let Some(monitor) = screens.first() {
            let image = monitor.capture_image()?;
            let width = image.width();
            let height = image.height();
            
            // xcap::RgbaImage is image::RgbaImage
            let pixels = image.as_raw();
            let texture = Texture2D::from_rgba8(width as u16, height as u16, pixels);
            self.snapshot = Some(texture);
        }
        Ok(())
    }
}

impl RayExtension for CaptureApplet {
    fn name(&self) -> &str { "Capture" }

    fn init(&mut self, ctx: &mut RayContext, _args: &clap::ArgMatches) -> Result<()> {
        ctx.register_hotkey(HotkeyDefinition {
            id: "region_screenshot".to_string(),
            key: "X".to_string(),
            modifiers: HotkeyModifiers::CTRL | HotkeyModifiers::SHIFT,
            scope: HotkeyScope::Global,
            description: "Capture Region Screenshot".to_string(),
            internal_keycode: None,
        });
        Ok(())
    }

    fn on_event(&mut self, ctx: &mut RayContext, event: &RayEvent) -> Result<()> {
        match event {
            RayEvent::HotkeyTriggered(id) if id == "region_screenshot" => {
                self.active = true;
                ctx.send_command(RayCommand::ToggleOverlay(true));
                self.capture_screenshot()?;
                self.selection_start = None;
                self.selection_end = None;
            }
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut RayContext) -> Result<()> {
        if !self.active { return Ok(()); }

        if is_key_pressed(KeyCode::Escape) {
            self.active = false;
            self.snapshot = None;
            ctx.send_command(RayCommand::ToggleOverlay(false));
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            self.selection_start = Some(vec2(mx, my));
            self.selection_end = Some(vec2(mx, my));
        } else if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            self.selection_end = Some(vec2(mx, my));
        } else if is_mouse_button_released(MouseButton::Left) {
            // Task 3 will implement cropping and saving
            self.active = false;
            ctx.send_command(RayCommand::ToggleOverlay(false));
            self.snapshot = None;
        }

        Ok(())
    }

    fn render(&mut self, _ctx: &mut RayContext) -> Result<()> {
        if !self.active { return Ok(()); }

        if let Some(tex) = &self.snapshot {
            draw_texture_ex(tex, 0.0, 0.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            });
        }

        // Draw selection rect
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let x = start.x.min(end.x);
            let y = start.y.min(end.y);
            let w = (start.x - end.x).abs();
            let h = (start.y - end.y).abs();
            
            // Dim background around selection
            draw_rectangle(0.0, 0.0, screen_width(), y, Color::from_rgba(0, 0, 0, 150)); // Top
            draw_rectangle(0.0, y + h, screen_width(), screen_height() - (y + h), Color::from_rgba(0, 0, 0, 150)); // Bottom
            draw_rectangle(0.0, y, x, h, Color::from_rgba(0, 0, 0, 150)); // Left
            draw_rectangle(x + w, y, screen_width() - (x + w), h, Color::from_rgba(0, 0, 0, 150)); // Right

            draw_rectangle_lines(x, y, w, h, 2.0, RED);
        } else {
            // Full screen dim
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 100));
        }

        Ok(())
    }
}
