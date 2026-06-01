use crate::YomichanApp;
use macroquad::prelude::*;
use macroquad::ui::Ui;
use std::time::SystemTime;

#[derive(PartialEq, Clone, Copy)]
pub enum ThreeDObject {
    None,
    Cube,
    Sphere,
    Pyramid,
    Fullscreen,
}

pub struct ThreeDState {
    pub selected_shape: ThreeDObject,
    pub rotation: f32,
    pub material: Option<Material>,
    pub shader_error: Option<String>,
    pub render_target: Option<RenderTarget>,
    pub resolution_scale: u32,
    pub available_shaders: Vec<String>,
    pub selected_shader: String,
    pub auto_compile: bool,
    pub last_modified: Option<SystemTime>,
    pub show_controls: bool,
    pub show_library: bool,
}

impl Default for ThreeDState {
    fn default() -> Self {
        Self {
            selected_shape: ThreeDObject::None,
            rotation: 0.0,
            material: None,
            shader_error: None,
            render_target: None,
            resolution_scale: 1,
            available_shaders: Vec::new(),
            selected_shader: "default.frag".to_string(),
            auto_compile: true,
            last_modified: None,
            show_controls: true,
            show_library: true,
        }
    }
}

const SHADER_DIR: &str = "shaders";

impl YomichanApp {
    pub fn refresh_shader_list(&mut self) {
        self.threed_state.available_shaders.clear();
        if let Ok(entries) = std::fs::read_dir(SHADER_DIR) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.ends_with(".frag") {
                            self.threed_state.available_shaders.push(name);
                        }
                    }
                }
            }
        }
        self.threed_state.available_shaders.sort();
    }

    pub fn auto_compile_check(&mut self) {
        if !self.threed_state.auto_compile {
            return;
        }

        let path = format!("{}/{}", SHADER_DIR, self.threed_state.selected_shader);
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if self.threed_state.last_modified.is_none()
                    || modified > self.threed_state.last_modified.unwrap()
                {
                    self.threed_state.last_modified = Some(modified);
                    self.compile_shader();
                }
            }
        }
    }

    pub fn compile_shader(&mut self) {
        let frag_path = format!("{}/{}", SHADER_DIR, self.threed_state.selected_shader);
        let vert_path = frag_path.replace(".frag", ".vert");

        let frag_code = std::fs::read_to_string(&frag_path).unwrap_or_else(|_| {
            let default_code = r#"#version 150
precision lowp float;
in vec2 uv;
out vec4 fragColor;
uniform sampler2D AudioTexture;
void main() {
    float audio = texture(AudioTexture, vec2(uv.x, 0.5)).r;
    fragColor = vec4(audio, 0.5, 1.0, 1.0);
}
"#;
            let _ = std::fs::write(&frag_path, default_code);
            default_code.to_string()
        });

        // Try to detect version from frag_code
        let version_line = frag_code.lines().next().unwrap_or("#version 100");

        let vert_code = std::fs::read_to_string(&vert_path).unwrap_or_else(|_| {
            if version_line.contains("400")
                || version_line.contains("330")
                || version_line.contains("150")
            {
                format!(
                    r#"{}
in vec3 position;
in vec2 texcoord;
out vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;

void main() {{
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}}
"#,
                    version_line
                )
            } else {
                r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#
                .to_string()
            }
        });

        let result = load_material(
            ShaderSource::Glsl {
                vertex: &vert_code,
                fragment: &frag_code,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc {
                        name: "Time".to_string(),
                        uniform_type: UniformType::Float1,
                        array_count: 1,
                    },
                    UniformDesc {
                        name: "Resolution".to_string(),
                        uniform_type: UniformType::Float2,
                        array_count: 1,
                    },
                    UniformDesc {
                        name: "AudioLevel".to_string(),
                        uniform_type: UniformType::Float1,
                        array_count: 1,
                    },
                ],
                textures: vec!["AudioTexture".to_string()],
                ..Default::default()
            },
        );

        match result {
            Ok(m) => {
                println!(
                    "SUCCESS: Compiled shader {}",
                    self.threed_state.selected_shader
                );
                self.threed_state.material = Some(m);
                self.threed_state.shader_error = None;
            }
            Err(e) => {
                eprintln!(
                    "ERROR: Failed to compile {}: {}",
                    self.threed_state.selected_shader, e
                );
                self.threed_state.shader_error = Some(format!("{}", e));
            }
        }
    }

    pub fn render_threed_scene(&mut self) {
        self.auto_compile_check();

        let (target_w, target_h) = if self.threed_state.resolution_scale > 1 {
            let scale = self.threed_state.resolution_scale as f32;
            (
                (screen_width() / scale).max(1.0),
                (screen_height() / scale).max(1.0),
            )
        } else {
            (screen_width(), screen_height())
        };

        // Manage render target for pixelation
        if self.threed_state.resolution_scale > 1 {
            let needs_update = match &self.threed_state.render_target {
                Some(rt) => rt.texture.width() != target_w || rt.texture.height() != target_h,
                None => true,
            };
            if needs_update {
                let rt = render_target(target_w as u32, target_h as u32);
                rt.texture.set_filter(FilterMode::Nearest);
                self.threed_state.render_target = Some(rt);
            }
        } else {
            self.threed_state.render_target = None;
        }

        let rt_handle = self.threed_state.render_target.clone();

        // Start rendering
        if self.threed_state.selected_shape == ThreeDObject::Fullscreen {
            // Fullscreen Mode (Pure Shader)
            if let Some(rt) = &rt_handle {
                set_camera(&Camera2D {
                    render_target: Some(rt.clone()),
                    ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, target_w, target_h))
                });
            } else {
                set_default_camera();
            }

            clear_background(BLACK);

            if let Some(material) = &self.threed_state.material {
                material.set_uniform("Time", get_time() as f32);
                material.set_uniform("Resolution", (target_w, target_h));

                let avg_vol: f32 = if !self.audio_state.buffer.is_empty() {
                    self.audio_state.buffer.iter().map(|s| s.abs()).sum::<f32>()
                        / self.audio_state.buffer.len() as f32
                } else {
                    0.0
                };

                if get_frame_time() > 0.0 {
                    static mut LAST_PRINT: f64 = 0.0;
                    unsafe {
                        let now = get_time();
                        if now - LAST_PRINT > 2.0 {
                            println!(
                                "Shader: {}, AudioLevel: {:.4}, Rec: {}",
                                self.threed_state.selected_shader,
                                avg_vol,
                                self.audio_state.is_recording
                            );
                            LAST_PRINT = now;
                        }
                    }
                }

                material.set_uniform("AudioLevel", avg_vol);

                if let Some(audio_tex) = &self.audio_state.audio_texture {
                    material.set_texture("AudioTexture", audio_tex.clone());
                }
                gl_use_material(material);
            }

            draw_rectangle(0.0, 0.0, target_w, target_h, WHITE);

            if self.threed_state.material.is_some() {
                gl_use_default_material();
            }
        } else {
            // 3D Scene Mode
            self.threed_state.rotation += 0.005;
            let x = self.threed_state.rotation.cos() * 5.0;
            let z = self.threed_state.rotation.sin() * 5.0;

            set_camera(&Camera3D {
                position: vec3(x, 3.0, z),
                up: vec3(0.0, 1.0, 0.0),
                target: vec3(0.0, 0.0, 0.0),
                render_target: rt_handle.clone(),
                aspect: Some(target_w / target_h),
                ..Default::default()
            });

            clear_background(BLACK);
            draw_grid(10, 1.0, GREEN, GRAY);

            if let Some(material) = &self.threed_state.material {
                material.set_uniform("Time", get_time() as f32);
                material.set_uniform("Resolution", (target_w, target_h));

                let avg_vol: f32 = if !self.audio_state.buffer.is_empty() {
                    self.audio_state.buffer.iter().map(|s| s.abs()).sum::<f32>()
                        / self.audio_state.buffer.len() as f32
                } else {
                    0.0
                };
                material.set_uniform("AudioLevel", avg_vol);

                if let Some(audio_tex) = &self.audio_state.audio_texture {
                    material.set_texture("AudioTexture", audio_tex.clone());
                }
                gl_use_material(material);
            }

            match self.threed_state.selected_shape {
                ThreeDObject::Cube => {
                    draw_cube(vec3(0.0, 1.0, 0.0), vec3(2.0, 2.0, 2.0), None, WHITE);
                    draw_cube_wires(vec3(0.0, 1.0, 0.0), vec3(2.0, 2.0, 2.0), MAROON);
                }
                ThreeDObject::Sphere => {
                    draw_sphere(vec3(0.0, 1.0, 0.0), 1.0, None, BLUE);
                    draw_sphere_wires(vec3(0.0, 1.0, 0.0), 1.0, None, SKYBLUE);
                }
                ThreeDObject::Pyramid => {
                    let top = vec3(0.0, 2.0, 0.0);
                    let b1 = vec3(-1.0, 0.0, -1.0);
                    let b2 = vec3(1.0, 0.0, -1.0);
                    let b3 = vec3(1.0, 0.0, 1.0);
                    let b4 = vec3(-1.0, 0.0, 1.0);

                    draw_line_3d(top, b1, YELLOW);
                    draw_line_3d(top, b2, YELLOW);
                    draw_line_3d(top, b3, YELLOW);
                    draw_line_3d(top, b4, YELLOW);
                    draw_line_3d(b1, b2, ORANGE);
                    draw_line_3d(b2, b3, ORANGE);
                    draw_line_3d(b3, b4, ORANGE);
                    draw_line_3d(b4, b1, ORANGE);
                }
                ThreeDObject::Fullscreen | ThreeDObject::None => {}
            }

            if self.threed_state.material.is_some() {
                gl_use_default_material();
            }
        }

        set_default_camera();

        // If we were rendering to a target, now draw that target to the screen
        if let Some(rt) = &self.threed_state.render_target {
            draw_texture_ex(
                &rt.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    flip_y: true, // RenderTargets in OpenGL are upside down relative to screen space
                    ..Default::default()
                },
            );
        }
    }

    pub fn draw_threed_tab(&mut self, ui: &mut Ui) {
        use macroquad::ui::hash;

        ui.checkbox(
            hash!("sh_ctrl"),
            "Show Settings",
            &mut self.threed_state.show_controls,
        );
        ui.same_line(0.0);
        ui.checkbox(
            hash!("sh_lib"),
            "Show Library",
            &mut self.threed_state.show_library,
        );

        // Window 1: Shader Controls
        if self.threed_state.show_controls {
            macroquad::ui::widgets::Window::new(
                hash!("threed_controls"),
                macroquad::math::vec2(10.0, 110.0),
                macroquad::math::vec2(320.0, 280.0),
            )
            .label("Shader Settings")
            .ui(ui, |ui| {
                ui.label(
                    None,
                    &format!("Active: {}", self.threed_state.selected_shader),
                );

                ui.separator();
                ui.label(None, "Shape:");
                if ui.button(None, "None") {
                    self.threed_state.selected_shape = ThreeDObject::None;
                }
                ui.same_line(0.0);
                if ui.button(None, "Cube") {
                    self.threed_state.selected_shape = ThreeDObject::Cube;
                }
                ui.same_line(0.0);
                if ui.button(None, "Sphere") {
                    self.threed_state.selected_shape = ThreeDObject::Sphere;
                }

                if ui.button(None, "Pyramid") {
                    self.threed_state.selected_shape = ThreeDObject::Pyramid;
                }
                ui.same_line(0.0);
                if ui.button(None, "Fullscreen") {
                    self.threed_state.selected_shape = ThreeDObject::Fullscreen;
                }
                ui.separator();
                let res_text = format!("Pixel Scale: {}x", self.threed_state.resolution_scale);
                ui.label(None, res_text.as_str());
                ui.same_line(0.0);
                if ui.button(None, "Cycle Scale") {
                    self.threed_state.resolution_scale *= 2;
                    if self.threed_state.resolution_scale > 64 {
                        self.threed_state.resolution_scale = 1;
                    }
                }

                ui.checkbox(
                    hash!("auto_compile"),
                    "Auto-Compile",
                    &mut self.threed_state.auto_compile,
                );

                ui.separator();
                if ui.button(None, "Manual Compile") {
                    self.compile_shader();
                }
                ui.same_line(0.0);
                if ui.button(None, "Clear Material") {
                    self.threed_state.material = None;
                }
                ui.same_line(0.0);
                if ui.button(None, "Copy Path") {
                    if let Ok(path) = std::env::current_dir() {
                        let abs_path = path
                            .join(SHADER_DIR)
                            .join(&self.threed_state.selected_shader);
                        let path_str = abs_path.to_string_lossy().to_string();
                        use std::io::Write;
                        use std::process::{Command, Stdio};
                        if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn()
                        {
                            if let Some(mut stdin) = child.stdin.take() {
                                _ = stdin.write_all(path_str.as_bytes());
                            }
                            _ = child.wait();
                        }
                    }
                }

                if self.threed_state.shader_error.is_some() {
                    ui.separator();
                    if ui.button(None, "!! VIEW SHADER ERRORS !!") {
                        self.router.set(crate::router::Route::ShaderError);
                    }
                }
            });
        }

        // Window 2: Shader Library
        if self.threed_state.show_library {
            macroquad::ui::widgets::Window::new(
                hash!("threed_library"),
                macroquad::math::vec2(340.0, 110.0),
                macroquad::math::vec2(250.0, 400.0),
            )
            .label("Shader Library")
            .ui(ui, |ui| {
                if ui.button(None, "Refresh List") {
                    self.refresh_shader_list();
                }
                ui.same_line(0.0);
                if ui.button(None, "New Shader") {
                    let name = format!("shader_{}.frag", get_time() as u32);
                    self.threed_state.selected_shader = name;
                    self.compile_shader();
                    self.refresh_shader_list();
                }

                ui.separator();

                // Simple list of shaders
                let shaders = self.threed_state.available_shaders.clone();
                for shader in shaders {
                    let is_selected = self.threed_state.selected_shader == shader;
                    let label = if is_selected {
                        format!("> {}", shader)
                    } else {
                        shader.clone()
                    };

                    if ui.button(None, label.as_str()) {
                        self.threed_state.selected_shader = shader;
                        self.threed_state.last_modified = None; // Trigger recompile
                        self.compile_shader();
                    }
                }
            });
        }
    }
}
