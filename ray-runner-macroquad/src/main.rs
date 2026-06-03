use macroquad::prelude::*;
use ray_core::RayEngine;
use clap::Command;

#[macroquad::main("Ray")]
async fn main() {
    let mut engine = RayEngine::new();
    
    engine.register(ray_applet_yomichan::YomichanApplet::new());
    engine.register(ray_applet_shaders::ShaderApplet::new());
    engine.register(ray_applet_audio::AudioApplet::new());

    let matches = Command::new("Ray")
        .version("0.1.0")
        .get_matches();

    if let Err(e) = engine.init(&matches) {
        eprintln!("Initialization failed: {}", e);
        return;
    }

    let mut show_settings = false;

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();
        if let Err(e) = engine.update(dt) {
            eprintln!("Update error: {}", e);
            break;
        }

        if !show_settings {
            if let Err(e) = engine.render() {
                eprintln!("Render error: {}", e);
                break;
            }
        } else {
            // Draw Settings View
            use macroquad::ui::{root_ui, hash};
            
            macroquad::ui::widgets::Window::new(
                hash!("settings_win"),
                vec2(50.0, 50.0),
                vec2(400.0, 500.0)
            )
            .label("Framework Settings")
            .ui(&mut root_ui(), |ui| {
                ui.label(None, "Registered Extensions:");
                ui.separator();
                
                let count = engine.extension_count();
                for i in 0..count {
                    if let Some((name, mut enabled)) = engine.get_extension_info(i) {
                        ui.checkbox(hash!(format!("ext_{}", i)), name, &mut enabled);
                        // If state changed, toggle in engine
                        // (Macroquad checkboxes update the bool in place)
                        // But engine stores it separately, so we check if it matches
                        if let Some((_, actual_enabled)) = engine.get_extension_info(i) {
                            if enabled != actual_enabled {
                                engine.toggle_extension(i);
                            }
                        }
                    }
                }
            });
        }

        // Draw Tab Bar
        use macroquad::ui::{root_ui, hash};
        let names = engine.enabled_extension_names();
        
        let bar_height = 40.0;
        let screen_w = screen_width();
        let screen_h = screen_height();

        draw_rectangle(0.0, screen_h - bar_height, screen_w, bar_height, Color::from_rgba(40, 40, 40, 255));
        
        let mut x_off = 10.0;
        
        // Settings Toggle Button
        let settings_label = if show_settings { "[ Settings ]" } else { "Settings" };
        if root_ui().button(Some(vec2(x_off, screen_h - bar_height + 5.0)), settings_label) {
            show_settings = !show_settings;
        }
        x_off += 120.0;

        if !show_settings {
            for (i, name) in names.iter().enumerate() {
                let label = if i == engine.active_extension_idx {
                    format!("[ {} ]", name)
                } else {
                    name.clone()
                };

                if root_ui().button(Some(vec2(x_off, screen_h - bar_height + 5.0)), label.as_str()) {
                    engine.active_extension_idx = i;
                }
                x_off += 120.0;
            }
        }

        next_frame().await;
    }
}
