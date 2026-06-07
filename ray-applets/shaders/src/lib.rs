use ray_api::{RayExtension, RayContext, RayEvent, AudioEvent, HotkeyDefinition, HotkeyModifiers, HotkeyScope};
use macroquad::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum RenderMode {
    None,
    Cube,
    Sphere,
    Pyramid,
    Fullscreen,
}

pub struct ShaderApplet {
    // Persistent System Config (The "Deep" settings)
    shader_dirs: Vec<PathBuf>,
    
    // Live Session State (The "On-Screen" HUD settings)
    selected_shader: String,
    render_mode: RenderMode,
    resolution_scale: u32,
    rotation: f32,
    show_controls: bool,
    show_library: bool,
    hud_visible: bool,
    
    // Internal cache
    last_modified: Option<SystemTime>,
    material: Option<Material>,
    render_target: Option<RenderTarget>,
    
    // Audio State
    volume: f32,
    spectrum_texture: Option<Texture2D>,
    
    // UI state
    available_shaders: Vec<(String, PathBuf)>,
}

impl ShaderApplet {
    pub fn new() -> Self {
        Self {
            shader_dirs: vec![PathBuf::from("ray-applets/legacy/shaders")],
            selected_shader: "default.frag".to_string(),
            render_mode: RenderMode::Fullscreen,
            resolution_scale: 1,
            rotation: 0.0,
            show_controls: true,
            show_library: true,
            hud_visible: true,
            last_modified: None,
            material: None,
            render_target: None,
            volume: 0.0,
            spectrum_texture: None,
            available_shaders: Vec::new(),
        }
    }

    fn refresh_shader_list(&mut self) {
        self.available_shaders.clear();
        for dir in &self.shader_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".frag") {
                        self.available_shaders.push((name, entry.path()));
                    }
                }
            }
        }
        self.available_shaders.sort_by(|a, b| a.0.cmp(&b.0));
    }

    fn get_current_shader_path(&self) -> Option<PathBuf> {
        self.available_shaders.iter()
            .find(|(name, _)| name == &self.selected_shader)
            .map(|(_, path)| path.clone())
    }

    fn reload_shader(&mut self) {
        let frag_path = match self.get_current_shader_path() {
            Some(p) => p,
            None => return,
        };

        let frag_src = match std::fs::read_to_string(&frag_path) {
            Ok(src) => src,
            Err(e) => {
                tracing::error!("Failed to read shader file: {}", e);
                return;
            }
        };

        let version_line = frag_src.lines().next().unwrap_or("#version 100");
        
        let vert_src = if version_line.contains("400") || version_line.contains("330") || version_line.contains("150") {
            format!(r#"{}
                in vec3 position;
                in vec2 texcoord;
                out vec2 uv;
                uniform mat4 Model;
                uniform mat4 Projection;
                void main() {{
                    gl_Position = Projection * Model * vec4(position, 1.0);
                    uv = texcoord;
                }}"#, version_line)
        } else {
            "#version 100
            attribute vec3 position;
            attribute vec2 texcoord;
            varying lowp vec2 uv;
            uniform mat4 Model;
            uniform mat4 Projection;
            void main() {
                gl_Position = Projection * Model * vec4(position, 1.0);
                uv = texcoord;
            }".to_string()
        };

        match load_material(
            ShaderSource::Glsl {
                vertex: &vert_src,
                fragment: &frag_src,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_write: true,
                    depth_test: Comparison::LessOrEqual,
                    ..Default::default()
                },
                uniforms: vec![
                    UniformDesc::new("iTime", UniformType::Float1),
                    UniformDesc::new("iResolution", UniformType::Float2),
                    UniformDesc::new("iVolume", UniformType::Float1),
                    UniformDesc::new("Time", UniformType::Float1),
                    UniformDesc::new("Resolution", UniformType::Float2),
                    UniformDesc::new("AudioLevel", UniformType::Float1),
                ],
                textures: vec!["iSpectrum".to_string(), "AudioTexture".to_string()],
            },
        ) {
            Ok(mat) => {
                self.material = Some(mat);
                tracing::info!("Successfully compiled shader: {}", self.selected_shader);
            }
            Err(e) => {
                tracing::error!("Shader compilation failed for {}:\n{}", self.selected_shader, e);
            }
        }
    }

    fn check_file_update(&mut self) {
        if let Some(path) = self.get_current_shader_path() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if Some(modified) != self.last_modified {
                        self.last_modified = Some(modified);
                        self.reload_shader();
                    }
                }
            }
        }
    }

    fn update_render_target(&mut self) {
        let (target_w, target_h) = if self.resolution_scale > 1 {
            let scale = self.resolution_scale as f32;
            ((screen_width() / scale).max(1.0), (screen_height() / scale).max(1.0))
        } else {
            (screen_width(), screen_height())
        };

        if self.resolution_scale > 1 {
            let needs_update = match &self.render_target {
                Some(rt) => rt.texture.width() != target_w || rt.texture.height() != target_h,
                None => true,
            };
            if needs_update {
                let rt = render_target(target_w as u32, target_h as u32);
                rt.texture.set_filter(FilterMode::Nearest);
                self.render_target = Some(rt);
            }
        } else {
            self.render_target = None;
        }
    }
}

impl RayExtension for ShaderApplet {
    fn name(&self) -> &str {
        "Shader IDE"
    }

    fn init(&mut self, ctx: &mut RayContext, _args: &clap::ArgMatches) -> anyhow::Result<()> {
        self.refresh_shader_list();
        self.reload_shader();

        ctx.register_hotkey(HotkeyDefinition {
            id: "toggle_hud".to_string(),
            key: "H".to_string(),
            modifiers: HotkeyModifiers::NONE,
            scope: HotkeyScope::Local,
            description: "Toggle Shader IDE HUD".to_string(),
            internal_keycode: None,
        });

        Ok(())
    }

    fn update(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        self.check_file_update();
        self.update_render_target();
        self.rotation += 0.005;
        Ok(())
    }

    fn on_event(&mut self, _ctx: &mut RayContext, event: &RayEvent) -> anyhow::Result<()> {
        match event {
            RayEvent::Audio(audio_ev) => {
                match audio_ev {
                    AudioEvent::Level(vol) => self.volume = *vol,
                    AudioEvent::Spectrum(data) => {
                        if self.spectrum_texture.is_none() {
                            self.spectrum_texture = Some(Texture2D::from_rgba8(512, 1, &vec![0u8; 512 * 4]));
                        }
                        if let Some(tex) = &self.spectrum_texture {
                            let mut bytes = Vec::with_capacity(512 * 4);
                            for &val in data {
                                let b = (val * 255.0).clamp(0.0, 255.0) as u8;
                                bytes.push(b); bytes.push(b); bytes.push(b); bytes.push(255);
                            }
                            tex.update_from_bytes(512, 1, &bytes);
                        }
                    }
                    _ => {}
                }
            }
            RayEvent::HotkeyTriggered(id) => {
                if id == "toggle_hud" {
                    self.hud_visible = !self.hud_visible;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn has_settings(&self) -> bool {
        true
    }

    fn settings_ui(&mut self, _ctx: &mut RayContext, ui: &mut macroquad::ui::Ui) -> anyhow::Result<()> {
        ui.label(None, "System Paths:");
        for dir in &self.shader_dirs {
            ui.label(None, &format!("- {:?}", dir));
        }
        
        if ui.button(None, "Add Default Path") {
            self.shader_dirs.push(PathBuf::from("ray-applets/legacy/shaders"));
            self.refresh_shader_list();
        }

        Ok(())
    }

    fn render(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        let (target_w, target_h) = if let Some(rt) = &self.render_target {
            (rt.texture.width(), rt.texture.height())
        } else {
            (screen_width(), screen_height())
        };

        // 1. Draw the Scene
        if self.render_mode == RenderMode::Fullscreen {
            if let Some(rt) = &self.render_target {
                set_camera(&Camera2D {
                    render_target: Some(rt.clone()),
                    ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, target_w, target_h))
                });
            } else {
                set_default_camera();
            }

            clear_background(BLACK);
            if let Some(mat) = &self.material {
                mat.set_uniform("iTime", get_time() as f32);
                mat.set_uniform("iResolution", vec2(target_w, target_h));
                mat.set_uniform("iVolume", self.volume);
                mat.set_uniform("Time", get_time() as f32);
                mat.set_uniform("Resolution", vec2(target_w, target_h));
                mat.set_uniform("AudioLevel", self.volume);
                if let Some(tex) = &self.spectrum_texture {
                    mat.set_texture("iSpectrum", tex.clone());
                    mat.set_texture("AudioTexture", tex.clone());
                }
                gl_use_material(mat);
            }
            draw_rectangle(0.0, 0.0, target_w, target_h, WHITE);
            gl_use_default_material();
        } else {
            let x = self.rotation.cos() * 5.0;
            let z = self.rotation.sin() * 5.0;
            set_camera(&Camera3D {
                position: vec3(x, 3.0, z),
                up: vec3(0.0, 1.0, 0.0),
                target: vec3(0.0, 0.0, 0.0),
                render_target: self.render_target.clone(),
                aspect: Some(target_w / target_h),
                ..Default::default()
            });

            clear_background(BLACK);
            draw_grid(10, 1.0, GREEN, GRAY);

            if let Some(mat) = &self.material {
                mat.set_uniform("iTime", get_time() as f32);
                mat.set_uniform("iResolution", vec2(target_w, target_h));
                mat.set_uniform("iVolume", self.volume);
                mat.set_uniform("Time", get_time() as f32);
                mat.set_uniform("Resolution", vec2(target_w, target_h));
                mat.set_uniform("AudioLevel", self.volume);
                if let Some(tex) = &self.spectrum_texture {
                    mat.set_texture("iSpectrum", tex.clone());
                    mat.set_texture("AudioTexture", tex.clone());
                }
                gl_use_material(mat);
            }

            match self.render_mode {
                RenderMode::Cube => {
                    draw_cube(vec3(0.0, 1.0, 0.0), vec3(2.0, 2.0, 2.0), None, WHITE);
                    draw_cube_wires(vec3(0.0, 1.0, 0.0), vec3(2.0, 2.0, 2.0), MAROON);
                }
                RenderMode::Sphere => {
                    draw_sphere(vec3(0.0, 1.0, 0.0), 1.0, None, BLUE);
                    draw_sphere_wires(vec3(0.0, 1.0, 0.0), 1.0, None, SKYBLUE);
                }
                RenderMode::Pyramid => {
                    let top = vec3(0.0, 2.0, 0.0);
                    let b1 = vec3(-1.0, 0.0, -1.0);
                    let b2 = vec3(1.0, 0.0, -1.0);
                    let b3 = vec3(1.0, 0.0, 1.0);
                    let b4 = vec3(-1.0, 0.0, 1.0);
                    draw_line_3d(top, b1, YELLOW); draw_line_3d(top, b2, YELLOW);
                    draw_line_3d(top, b3, YELLOW); draw_line_3d(top, b4, YELLOW);
                    draw_line_3d(b1, b2, ORANGE); draw_line_3d(b2, b3, ORANGE);
                    draw_line_3d(b3, b4, ORANGE); draw_line_3d(b4, b1, ORANGE);
                }
                _ => {}
            }
            gl_use_default_material();
        }

        set_default_camera();
        if let Some(rt) = &self.render_target {
            draw_texture_ex(&rt.texture, 0.0, 0.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            });
        }

        // 2. Draw the HUDs
        if self.hud_visible {
            use macroquad::ui::{root_ui, hash};
            
            root_ui().checkbox(hash!("sh_ctrl"), "Controls", &mut self.show_controls);
            root_ui().same_line(0.0);
            root_ui().checkbox(hash!("sh_lib"), "Library", &mut self.show_library);

            if self.show_controls {
                macroquad::ui::widgets::Window::new(hash!("sh_win"), vec2(10.0, 110.0), vec2(300.0, 250.0))
                    .label("Shader Settings")
                    .ui(&mut root_ui(), |ui| {
                        ui.label(None, &format!("Active: {}", self.selected_shader));
                        ui.separator();
                        ui.label(None, "Shape:");
                        if ui.button(None, "Full") { self.render_mode = RenderMode::Fullscreen; }
                        ui.same_line(0.0);
                        if ui.button(None, "Cube") { self.render_mode = RenderMode::Cube; }
                        ui.same_line(0.0);
                        if ui.button(None, "Sphere") { self.render_mode = RenderMode::Sphere; }
                        ui.same_line(0.0);
                        if ui.button(None, "Pyramid") { self.render_mode = RenderMode::Pyramid; }
                        
                        ui.separator();
                        let res_text = format!("Pixel Scale: {}x", self.resolution_scale);
                        ui.label(None, &res_text);
                        if ui.button(None, "Cycle Scale") {
                            self.resolution_scale *= 2;
                            if self.resolution_scale > 32 { self.resolution_scale = 1; }
                        }
                        
                        ui.separator();
                        if ui.button(None, "Toggle Audio") {
                            _ctx.bus.send(RayEvent::Command(ray_api::RayCommand::Audio(ray_api::AudioCommand::ToggleRecording)));
                        }

                        if ui.button(None, "Manual Compile") { self.reload_shader(); }
                    });
            }

            if self.show_library {
                macroquad::ui::widgets::Window::new(hash!("sh_lib_win"), vec2(320.0, 110.0), vec2(250.0, 400.0))
                    .label("Shader Library")
                    .ui(&mut root_ui(), |ui| {
                        if ui.button(None, "Refresh List") { self.refresh_shader_list(); }
                        ui.separator();
                        for (name, _) in self.available_shaders.clone() {
                            let label = if self.selected_shader == name { format!("> {}", name) } else { name.clone() };
                            if ui.button(None, label.as_str()) {
                                self.selected_shader = name;
                                self.last_modified = None;
                            }
                        }
                    });
            }
        }
        
        Ok(())
    }
}
