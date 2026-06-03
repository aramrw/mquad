use ray_api::{RayExtension, RayContext, RayEvent, HotkeyDefinition, HotkeyScope, HotkeyModifiers, RayCommand};
use macroquad::prelude::*;
use anyhow::Result;

pub struct CaptureApplet {
    active: bool,
    snapshot_tex: Option<Texture2D>,
    snapshot_raw: Option<image::RgbaImage>,
    selection_start: Option<Vec2>,
    selection_end: Option<Vec2>,
}

impl CaptureApplet {
    pub fn new() -> Self {
        Self {
            active: false,
            snapshot_tex: None,
            snapshot_raw: None,
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
            
            let pixels = image.as_raw();
            let texture = Texture2D::from_rgba8(width as u16, height as u16, pixels);
            self.snapshot_tex = Some(texture);
            self.snapshot_raw = Some(image);
        }
        Ok(())
    }

    fn finalize_selection(&mut self, ctx: &mut RayContext) -> Result<()> {
        if let (Some(start), Some(end), Some(raw)) = (self.selection_start, self.selection_end, &self.snapshot_raw) {
            let x = start.x.min(end.x);
            let y = start.y.min(end.y);
            let w = (start.x - end.x).abs();
            let h = (start.y - end.y).abs();

            if w > 1.0 && h > 1.0 {
                let sw = screen_width();
                let sh = screen_height();
                let iw = raw.width() as f32;
                let ih = raw.height() as f32;
                
                let scale_x = iw / sw;
                let scale_y = ih / sh;
                
                let ix = (x * scale_x) as u32;
                let iy = (y * scale_y) as u32;
                let iw_crop = (w * scale_x).min(iw - ix as f32) as u32;
                let ih_crop = (h * scale_y).min(ih - iy as f32) as u32;
                
                if iw_crop > 0 && ih_crop > 0 {
                    use image::GenericImageView;
                    let cropped = raw.view(ix, iy, iw_crop, ih_crop).to_image();
                    
                    // Save to file
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                    let filename = format!("screenshot_{}.png", timestamp);
                    cropped.save(&filename)?;
                    
                    // Copy to clipboard
                    ctx.clipboard_write_image(iw_crop as usize, ih_crop as usize, cropped.as_raw());
                    
                    tracing::info!("Screenshot saved to {} and copied to clipboard", filename);
                }
            }
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
            self.snapshot_tex = None;
            self.snapshot_raw = None;
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
            self.finalize_selection(ctx)?;
            self.active = false;
            ctx.send_command(RayCommand::ToggleOverlay(false));
            self.snapshot_tex = None;
            self.snapshot_raw = None;
        }

        Ok(())
    }

    fn render(&mut self, _ctx: &mut RayContext) -> Result<()> {
        if !self.active { return Ok(()); }

        if let Some(tex) = &self.snapshot_tex {
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
