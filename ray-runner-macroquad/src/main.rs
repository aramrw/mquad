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

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();
        if let Err(e) = engine.update(dt) {
            eprintln!("Update error: {}", e);
            break;
        }

        if let Err(e) = engine.render() {
            eprintln!("Render error: {}", e);
            break;
        }

        // Draw Tab Bar
        use macroquad::ui::root_ui;
        let names = engine.extension_names();
        
        let bar_height = 40.0;
        let screen_w = screen_width();
        let screen_h = screen_height();

        draw_rectangle(0.0, screen_h - bar_height, screen_w, bar_height, Color::from_rgba(40, 40, 40, 255));
        
        let mut x_off = 10.0;
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

        next_frame().await;
    }
}
