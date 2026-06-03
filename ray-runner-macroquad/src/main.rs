use macroquad::prelude::*;
use ray_core::RayEngine;
use clap::Command;

#[macroquad::main("Ray")]
async fn main() {
    let mut engine = RayEngine::new("framework_settings.db");
    
    engine.register(ray_applet_yomichan::YomichanApplet::new());
    engine.register(ray_applet_shaders::ShaderApplet::new());
    engine.register(ray_applet_audio::AudioApplet::new());

    // Load persisted enabled/disabled states after registration
    let _ = engine.load_settings();

    let matches = Command::new("Ray")
        .version("0.1.0")
        .get_matches();

    if let Err(e) = engine.init(&matches) {
        eprintln!("Initialization failed: {}", e);
        return;
    }

    let mut show_settings = false;
    let mut active_settings_idx: Option<usize> = None;
    let mut show_console = false;
    let mut console_logs: Vec<ray_api::LogEvent> = Vec::new();

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();
        
        // Before update, capture logs from the bus to populate console
        while let Some(event) = engine.bus().poll() {
            if let ray_api::RayEvent::Log(log) = event {
                console_logs.push(log);
                if console_logs.len() > 1000 {
                    console_logs.remove(0);
                }
            } else {
                // Re-inject non-log events so engine can process them
                engine.bus().send(event);
                break;
            }
        }

        if let Err(e) = engine.update(dt) {
            eprintln!("Update error: {}", e);
            break;
        }

        if !show_settings && !show_console {
            if let Err(e) = engine.render() {
                eprintln!("Render error: {}", e);
                break;
            }
        } else if show_settings {
            use macroquad::ui::{root_ui, hash};
            
            macroquad::ui::widgets::Window::new(
                hash!("settings_win"),
                vec2(50.0, 50.0),
                vec2(400.0, 500.0)
            )
            .label(if active_settings_idx.is_some() { "Extension Config" } else { "Framework Settings" })
            .ui(&mut root_ui(), |ui| {
                if let Some(idx) = active_settings_idx {
                    if let Some((name, _)) = engine.get_extension_info(idx) {
                        ui.label(None, &format!("Configuring: {}", name));
                        if ui.button(None, "<- Back to Settings") {
                            active_settings_idx = None;
                        }
                        ui.separator();
                        if let Err(e) = engine.render_extension_settings(idx, ui) {
                            ui.label(None, &format!("Error rendering settings: {}", e));
                        }
                    }
                } else {
                    ui.label(None, "Registered Extensions:");
                    ui.separator();
                    
                    let count = engine.extension_count();
                    for i in 0..count {
                        if let Some((name, mut enabled)) = engine.get_extension_info(i) {
                            ui.checkbox(hash!(format!("ext_{}", i)), name, &mut enabled);
                            
                            if engine.extension_has_settings(i) {
                                ui.same_line(0.0);
                                if ui.button(None, "Configure") {
                                    active_settings_idx = Some(i);
                                }
                            }

                            if let Some((_, actual_enabled)) = engine.get_extension_info(i) {
                                if enabled != actual_enabled {
                                    engine.toggle_extension(i);
                                }
                            }
                        }
                    }
                }
            });
        } else if show_console {
            use macroquad::ui::{root_ui, hash};
            let win_width = screen_width() - 100.0;
            let win_height = screen_height() - 150.0;
            
            macroquad::ui::widgets::Window::new(
                hash!("console_win"),
                vec2(50.0, 50.0),
                vec2(win_width, win_height)
            )
            .label("Debug Console")
            .ui(&mut root_ui(), |ui| {
                if ui.button(None, "Clear") {
                    console_logs.clear();
                }
                ui.same_line(0.0);
                if ui.button(None, "Copy All") {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let all_logs: String = console_logs.iter()
                            .map(|log| format!("[{}] [{}] {}", log.level_str(), log.target, log.message))
                            .collect::<Vec<_>>()
                            .join("\n");
                        let _ = clipboard.set_text(all_logs);
                    }
                }
                ui.separator();

                // Roughly estimate characters per line based on window width
                // Default font is roughly 8px wide per char
                let chars_per_line = ((win_width - 100.0) / 8.5) as usize;

                for log in console_logs.iter().rev().take(100) {
                    let full_msg = format!("[{}] [{}] {}", log.level_str(), log.target, log.message);
                    
                    if ui.button(None, "Copy") {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(full_msg.clone());
                        }
                    }
                    ui.same_line(0.0);
                    
                    // Simple word wrap logic
                    if full_msg.len() > chars_per_line {
                        let mut remaining = full_msg.as_str();
                        let mut first = true;
                        while !remaining.is_empty() {
                            let end = remaining.len().min(chars_per_line);
                            let chunk = &remaining[..end];
                            if !first {
                                // Indent wrapped lines slightly and skip the button space
                                ui.label(None, &format!("       {}", chunk));
                            } else {
                                ui.label(None, chunk);
                                first = false;
                            }
                            remaining = &remaining[end..];
                        }
                    } else {
                        ui.label(None, &full_msg);
                    }
                }
            });
        }

        // Draw Tab Bar
        use macroquad::ui::root_ui;
        let names = engine.enabled_extension_names();
        
        let bar_height = 40.0;
        let screen_w = screen_width();
        let screen_h = screen_height();

        draw_rectangle(0.0, screen_h - bar_height, screen_w, bar_height, Color::from_rgba(40, 40, 40, 255));
        
        let mut x_off = 10.0;
        
        // Settings Toggle
        let settings_label = if show_settings { "[ Settings ]" } else { "Settings" };
        if root_ui().button(Some(vec2(x_off, screen_h - bar_height + 5.0)), settings_label) {
            show_settings = !show_settings;
            show_console = false;
        }
        x_off += 120.0;

        // Console Toggle
        let console_label = if show_console { "[ Console ]" } else { "Console" };
        if root_ui().button(Some(vec2(x_off, screen_h - bar_height + 5.0)), console_label) {
            show_console = !show_console;
            show_settings = false;
        }
        x_off += 120.0;

        if !show_settings && !show_console {
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
