use ray_core::RayEngine;
use clap::Command;

fn main() {
    let mut engine = RayEngine::new();
    
    let matches = Command::new("Ray CLI")
        .version("0.1.0")
        .get_matches();

    if let Err(e) = engine.init(&matches) {
        eprintln!("Initialization failed: {}", e);
        return;
    }

    // CLI runner might run once or in a loop depending on flags
    loop {
        // Placeholder for headless loop
        if let Err(e) = engine.update(0.016) { // 60fps-ish
            eprintln!("Update error: {}", e);
            break;
        }
        // No render in CLI runner usually, or render to string/file
    }
}
