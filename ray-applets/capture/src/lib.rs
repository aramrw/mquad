use ray_api::{RayExtension, RayContext, RayEvent, HotkeyDefinition, HotkeyScope, HotkeyModifiers, RayCommand, AudioEvent};
use macroquad::prelude::*;
use anyhow::Result;

#[derive(PartialEq)]
enum CaptureMode {
    Screenshot,
    Video,
}

pub struct CaptureApplet {
    active: bool,
    mode: CaptureMode,
    snapshot_tex: Option<Texture2D>,
    snapshot_raw: Option<image::RgbaImage>,
    selection_start: Option<Vec2>,
    selection_end: Option<Vec2>,
    recording_process: Option<std::process::Child>,
    recording_filename: Option<String>,
    crf: i32,
    fps: i32,
    save_dir: String,
    audio_enabled: bool,
    audio_stdin: Option<std::io::BufWriter<std::process::ChildStdin>>,
}

impl CaptureApplet {
    pub fn new() -> Self {
        Self {
            active: false,
            mode: CaptureMode::Screenshot,
            snapshot_tex: None,
            snapshot_raw: None,
            selection_start: None,
            selection_end: None,
            recording_process: None,
            recording_filename: None,
            crf: 23,
            fps: 30,
            save_dir: ".".to_string(),
            audio_enabled: false,
            audio_stdin: None,
        }
    }

    fn save_settings(&self) -> Result<()> {
        let conn = rusqlite::Connection::open("framework_settings.db")?;
        conn.execute(
            "INSERT OR REPLACE INTO applet_configs (applet, key, value) VALUES (?1, ?2, ?3)",
            rusqlite::params!["capture", "audio_enabled", self.audio_enabled.to_string()],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO applet_configs (applet, key, value) VALUES (?1, ?2, ?3)",
            rusqlite::params!["capture", "audio_device_index", self.audio_device_index.to_string()],
        )?;
        Ok(())
    }

    fn load_settings(&mut self) {
        if let Ok(conn) = rusqlite::Connection::open("framework_settings.db") {
            let mut stmt = conn.prepare("SELECT key, value FROM applet_configs WHERE applet = ?1").ok().unwrap();
            let rows = stmt.query_map(rusqlite::params!["capture"], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }).ok().unwrap();

            for row in rows.flatten() {
                match row.0.as_str() {
                    "audio_enabled" => self.audio_enabled = row.1.parse().unwrap_or(false),
                    "audio_device_index" => self.audio_device_index = row.1.parse().unwrap_or(1),
                    _ => {}
                }
            }
        }
    }

    fn start_recording(&mut self, ctx: &mut RayContext) -> Result<()> {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let x = start.x.min(end.x) as u32;
            let y = start.y.min(end.y) as u32;
            let w = (start.x - end.x).abs() as u32;
            let h = (start.y - end.y).abs() as u32;

            if w > 10 && h > 10 {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                let path = std::path::Path::new(&self.save_dir).join(format!("recording_{}.mp4", timestamp));
                let filename = path.to_str().unwrap().to_string();
                
                #[cfg(target_os = "macos")]
                let (input_format, input_device) = ("avfoundation", "1:none");
                #[cfg(target_os = "windows")]
                let (input_format, input_device) = ("gdigrab", "desktop");
                #[cfg(not(any(target_os = "macos", windows)))]
                let (input_format, input_device) = ("x11grab", ":0.0");

                let mut cmd = std::process::Command::new("ffmpeg");
                
                // Input 0: Video
                cmd.args(&["-f", input_format, "-framerate", &self.fps.to_string(), "-i", input_device]);
                
                if self.audio_enabled {
                    // Input 1: Audio from Stdin
                    cmd.args(&["-f", "f32le", "-ar", "44100", "-ac", "1", "-i", "-"]);
                    cmd.stdin(std::process::Stdio::piped());
                    
                    // Trigger Audio extension
                    ctx.bus.send(RayEvent::Command(RayCommand::Audio(ray_api::AudioCommand::StartRecording)));
                }

                let crop_filter = format!("crop={}:{}:{}:{}", w, h, x, y);
                cmd.args(&["-vf", &crop_filter, "-c:v", "libx264", "-crf", &self.crf.to_string(), "-pix_fmt", "yuv420p"]);
                
                if self.audio_enabled {
                    cmd.args(&["-c:a", "aac", "-shortest"]);
                    // Map inputs: 0:v for video, 1:a for audio
                    cmd.args(&["-map", "0:v", "-map", "1:a"]);
                }

                cmd.arg(&filename);

                let mut child = cmd.spawn()?;
                if self.audio_enabled {
                    if let Some(stdin) = child.stdin.take() {
                        self.audio_stdin = Some(std::io::BufWriter::new(stdin));
                    }
                }
                self.recording_process = Some(child);
                self.recording_filename = Some(filename);
                ctx.send_command(RayCommand::MiniMode(true));
                tracing::info!("Started recording to {}", self.recording_filename.as_ref().unwrap());
            }
        }
        Ok(())
    }

    fn stop_recording(&mut self, ctx: &mut RayContext) -> Result<()> {
        self.audio_stdin = None; // Dropping BufWriter closes the pipe
        if let Some(mut child) = self.recording_process.take() {
            let _ = child.kill();
            ctx.send_command(RayCommand::MiniMode(false));
            tracing::info!("Stopped recording. Saved to {:?}", self.recording_filename.take());
        }
        Ok(())
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
                    
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                    let path = std::path::Path::new(&self.save_dir).join(format!("screenshot_{}.png", timestamp));
                    let filename = path.to_str().unwrap().to_string();
                    cropped.save(&filename)?;
                    
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
        self.load_settings();
        ctx.register_hotkey(HotkeyDefinition {
            id: "region_screenshot".to_string(),
            key: "X".to_string(),
            modifiers: HotkeyModifiers::CTRL | HotkeyModifiers::SHIFT,
            scope: HotkeyScope::Global,
            description: "Capture Region Screenshot".to_string(),
            internal_keycode: None,
        });
        ctx.register_hotkey(HotkeyDefinition {
            id: "region_video".to_string(),
            key: "R".to_string(),
            modifiers: HotkeyModifiers::CTRL | HotkeyModifiers::SHIFT,
            scope: HotkeyScope::Global,
            description: "Capture Region Video".to_string(),
            internal_keycode: None,
        });
        Ok(())
    }

    fn on_event(&mut self, ctx: &mut RayContext, event: &RayEvent) -> Result<()> {
        match event {
            RayEvent::HotkeyTriggered(id) if id == \"region_screenshot\" => {
                if self.recording_process.is_some() {
                    self.stop_recording(ctx)?;
                } else {
                    self.active = true;
                    self.mode = CaptureMode::Screenshot;
                    ctx.send_command(RayCommand::SelectExtension(\"Capture\".to_string()));
                    ctx.send_command(RayCommand::ToggleOverlay(true));
                    self.capture_screenshot()?;
                    self.selection_start = None;
                    self.selection_end = None;
                }
            }
            RayEvent::HotkeyTriggered(id) if id == \"region_video\" => {
                if self.recording_process.is_some() {
                    self.stop_recording(ctx)?;
                } else {
                    self.active = true;
                    self.mode = CaptureMode::Video;
                    ctx.send_command(RayCommand::SelectExtension(\"Capture\".to_string()));
                    ctx.send_command(RayCommand::ToggleOverlay(true));
                    self.capture_screenshot()?;
                    self.selection_start = None;
                    self.selection_end = None;
                }
            }
            RayEvent::Audio(AudioEvent::Buffer(samples)) => {
                if let Some(writer) = &mut self.audio_stdin {
                    use std::io::Write;
                    for &sample in samples {
                        let bytes = sample.to_le_bytes();
                        if writer.write_all(&bytes).is_err() {
                            // Pipe might have closed if ffmpeg crashed
                            break;
                        }
                    }
                    let _ = writer.flush();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut RayContext) -> Result<()> {
        if self.recording_process.is_some() {
            // Check if process is still running
            if let Some(proc) = &mut self.recording_process {
                if let Ok(Some(_status)) = proc.try_wait() {
                    self.recording_process = None;
                    self.recording_filename = None;
                    ctx.send_command(RayCommand::MiniMode(false));
                }
            }
        }

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
            if self.mode == CaptureMode::Screenshot {
                self.finalize_selection(ctx)?;
            } else {
                self.start_recording(ctx)?;
            }
            self.active = false;
            ctx.send_command(RayCommand::ToggleOverlay(false));
            self.snapshot_tex = None;
            self.snapshot_raw = None;
        }

        Ok(())
    }

    fn render(&mut self, ctx: &mut RayContext) -> Result<()> {
        if self.recording_process.is_some() {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(40, 40, 40, 255));
            draw_circle(30.0, 30.0, 10.0, RED);
            draw_text("REC", 50.0, 38.0, 30.0, WHITE);
            return Ok(());
        }

        if !self.active { return Ok(()); }
        
        // ... rest of render ...
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
            
            let label = if self.mode == CaptureMode::Screenshot { "SCREENSHOT" } else { "RECORD" };
            draw_text(label, x, y - 5.0, 20.0, RED);
        } else {
            // Full screen dim
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 100));
            draw_text("Select Region (ESC to cancel)", 20.0, 30.0, 25.0, WHITE);
        }

        Ok(())
    }

    fn has_settings(&self) -> bool { true }

    fn settings_ui(&mut self, _ctx: &mut RayContext, ui: &mut macroquad::ui::Ui) -> Result<()> {
        use macroquad::ui::hash;
        ui.label(None, "Capture Settings");
        ui.separator();
        
        ui.label(None, &format!("Save Directory: {}", self.save_dir));
        if ui.button(None, "Change Directory...") {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.save_dir = path.to_string_lossy().into_owned();
            }
        }
        
        ui.separator();
        ui.label(None, "Video Recording:");
        
        let old_audio = self.audio_enabled;
        ui.checkbox(hash!("audio_enabled"), "Include Audio", &mut self.audio_enabled);
        if self.audio_enabled != old_audio {
            let _ = self.save_settings();
        }
        ui.label(None, &format!("Audio Device Index: {}", self.audio_device_index));
        
        let mut crf = self.crf as f32;
        ui.slider(hash!("crf_slider"), "Quality (CRF, lower is better)", 0.0..51.0, &mut crf);
        self.crf = crf as i32;
        
        let mut fps = self.fps as f32;
        ui.slider(hash!("fps_slider"), "Framerate (FPS)", 10.0..60.0, &mut fps);
        self.fps = fps as i32;
        
        Ok(())
    }
}
