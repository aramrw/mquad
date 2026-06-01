use crate::YomichanApp;
use macroquad::prelude::*;
use macroquad::ui::Ui;

#[derive(PartialEq, Clone, Copy)]
pub enum ThreeDObject {
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
}

impl Default for ThreeDState {
    fn default() -> Self {
        Self {
            selected_shape: ThreeDObject::Cube,
            rotation: 0.0,
            material: None,
            shader_error: None,
            render_target: None,
            resolution_scale: 8,
        }
    }
}

const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

const SHADER_FILE: &str = "shader.frag";

impl YomichanApp {
    pub fn compile_shader(&mut self) {
        let code = match std::fs::read_to_string(SHADER_FILE) {
            Ok(c) => c,
            Err(_) => {
                // If file doesn't exist, create it with default code
                let default_code = r#"#version 100
precision lowp float;
varying vec2 uv;
uniform float Time;
void main() {
    gl_FragColor = vec4(uv.x, uv.y, abs(sin(Time)), 1.0);
}
"#;
                let _ = std::fs::write(SHADER_FILE, default_code);
                default_code.to_string()
            }
        };

        let result = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: &code,
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
                ],
                ..Default::default()
            },
        );

        match result {
            Ok(m) => {
                self.threed_state.material = Some(m);
                self.threed_state.shader_error = None;
            }
            Err(e) => {
                self.threed_state.shader_error = Some(format!("{:?}", e));
            }
        }
    }

    pub fn render_threed_scene(&mut self) {
        let (target_w, target_h) = if self.threed_state.resolution_scale > 1 {
            let scale = self.threed_state.resolution_scale as f32;
            ((screen_width() / scale).max(1.0), (screen_height() / scale).max(1.0))
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

            if let Some(material) = &self.threed_state.material {
                material.set_uniform("Time", get_time() as f32);
                material.set_uniform("Resolution", (target_w, target_h));
                gl_use_material(material);
            }

            draw_grid(10, 1.0, GREEN, GRAY);

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
                ThreeDObject::Fullscreen => unreachable!(),
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
        ui.label(None, "Active Shader");

        if ui.button(None, "Cube") {
            self.threed_state.selected_shape = ThreeDObject::Cube;
        }
        ui.same_line(0.0);
        if ui.button(None, "Sphere") {
            self.threed_state.selected_shape = ThreeDObject::Sphere;
        }
        ui.same_line(0.0);
        if ui.button(None, "Pyramid") {
            self.threed_state.selected_shape = ThreeDObject::Pyramid;
        }
        ui.same_line(0.0);
        if ui.button(None, "Fullscreen") {
            self.threed_state.selected_shape = ThreeDObject::Fullscreen;
        }

        ui.same_line(0.0);
        let res_text = format!("Res: {}x", self.threed_state.resolution_scale);
        if ui.button(None, res_text.as_str()) {
            // bounded between 8-48
            self.threed_state.resolution_scale *= 2;
            if self.threed_state.resolution_scale > 48 {
                self.threed_state.resolution_scale = 8;
            }
        }

        ui.same_line(0.0);
        if ui.button(None, "Compile") {
            self.compile_shader();
        }

        ui.same_line(0.0);
        if ui.button(None, "Clear Shader") {
            self.threed_state.material = None;
            self.threed_state.shader_error = None;
        }

        ui.same_line(0.0);
        if ui.button(None, "cpy_fpath") {
            if let Ok(path) = std::env::current_dir() {
                let abs_path = path.join(SHADER_FILE);
                let path_str = abs_path.to_string_lossy().to_string();

                // Use pbcopy on macOS
                use std::io::Write;
                use std::process::{Command, Stdio};
                if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(path_str.as_bytes());
                    }
                    let _ = child.wait();
                }
            }
        }

        if let Some(err) = &self.threed_state.shader_error {
            ui.label(None, &format!("Error: {}", err));
        }

        ui.separator();
        // thanks for the step by step..
        // ui.label(None, "External Shader Mode:");
        // ui.label(None, "- Edit 'shader.frag' in your favorite editor.");
        // ui.label(None, "- Click 'Compile' to see changes.");
        // ui.label(None, "- Click 'cpy_fpath' to copy the absolute path.");
    }
}
