use macroquad::prelude::*;
use ray_core::RayEngine;
use clap::Command;

#[macroquad::main("Ray")]
async fn main() {
    let mut engine = RayEngine::new();
    
    // In a real scenario, we'd discover applets here
    // engine.register(MyCoolApplet::new());

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

        next_frame().await;
    }
}
