use macroquad::prelude::*;
use ray_core::RayEngine;
use clap::Command;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}, GlobalHotKeyEvent};
use std::str::FromStr;

#[cfg(target_os = "macos")]
use core_foundation::runloop::{CFRunLoopRunInMode, kCFRunLoopDefaultMode};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

fn window_conf() -> Conf {
    Conf {
        window_title: "Ray".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut engine = RayEngine::new("framework_settings.db");
    
    let mut os_hotkey_manager = OsHotkeyManager::new();

    engine.register(ray_applet_yomichan::YomichanApplet::new());
    engine.register(ray_applet_shaders::ShaderApplet::new());
    engine.register(ray_applet_audio::AudioApplet::new());
    engine.register(ray_applet_capture::CaptureApplet::new());

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
    let mut settings_tab = 0; // 0 for Extensions, 1 for Hotkeys
    let mut show_console = false;
    let mut console_logs: Vec<ray_api::LogEvent> = Vec::new();
    let mut last_overlay_active = false;
    let mut last_mini_mode = false;

    loop {
        let frame_start = std::time::Instant::now();
        clear_background(BLACK);

        #[cfg(target_os = "macos")]
        unsafe {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.00001, 0);
        }

        // Handle Overlay State Changes
        if engine.overlay_active != last_overlay_active {
            update_window_overlay(engine.overlay_active);
            last_overlay_active = engine.overlay_active;
        }

        // Handle Mini Mode Changes
        if engine.mini_mode != last_mini_mode {
            if engine.mini_mode {
                request_new_screen_size(200.0, 60.0);
            } else {
                request_new_screen_size(800.0, 600.0);
            }
            last_mini_mode = engine.mini_mode;
        }

        // Sync and Poll OS hotkeys
        if engine.is_hotkey_registry_dirty() {
            os_hotkey_manager.sync(&engine);
            engine.clear_hotkey_registry_dirty();
        }
        os_hotkey_manager.poll(&mut engine);

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
            render_settings_ui(&mut engine, &mut active_settings_idx, &mut settings_tab);
        } else if show_console {
            render_console_ui(&mut console_logs);
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

        if !engine.vsync_enabled {
            let target_fps = 60.0;
            let target_frame_time = std::time::Duration::from_secs_f64(1.0 / target_fps);
            let elapsed = frame_start.elapsed();
            if elapsed < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed);
            }
        }
    }
}

fn ray_to_global_modifiers(m: ray_api::HotkeyModifiers) -> Modifiers {
    let mut out = Modifiers::empty();
    if m.contains(ray_api::HotkeyModifiers::SHIFT) { out |= Modifiers::SHIFT; }
    if m.contains(ray_api::HotkeyModifiers::CTRL) { out |= Modifiers::CONTROL; }
    if m.contains(ray_api::HotkeyModifiers::ALT) { out |= Modifiers::ALT; }
    if m.contains(ray_api::HotkeyModifiers::LOGO) { out |= Modifiers::SUPER; }
    out
}

fn ray_to_global_code(key: &str) -> Option<Code> {
    Code::from_str(&format!("Key{}", key)).ok()
        .or_else(|| Code::from_str(key).ok())
}

struct OsHotkeyManager {
    manager: Option<GlobalHotKeyManager>,
    registered: std::collections::HashMap<(String, String), HotKey>,
}

impl OsHotkeyManager {
    fn new() -> Self {
        let manager = GlobalHotKeyManager::new().map_err(|e| {
            eprintln!("GlobalHotKeyManager error: {:?}", e);
            e
        }).ok();
        Self {
            manager,
            registered: std::collections::HashMap::new(),
        }
    }

    fn sync(&mut self, engine: &RayEngine) {
        let Some(manager) = &self.manager else { return };

        let mut current_global_keys = std::collections::HashSet::new();
        for (key, def) in &engine.hotkey_registry.registered {
            if def.scope == ray_api::HotkeyScope::Global {
                current_global_keys.insert(key.clone());
            }
        }

        // Unregister removed
        let to_remove: Vec<_> = self.registered.keys()
            .filter(|k| !current_global_keys.contains(*k))
            .cloned()
            .collect();

        for key in to_remove {
            if let Some(hotkey) = self.registered.remove(&key) {
                let _ = manager.unregister(hotkey);
            }
        }

        // Register new
        for key in current_global_keys {
            if !self.registered.contains_key(&key) {
                if let Some(def) = engine.hotkey_registry.registered.get(&key) {
                    if let Some(code) = ray_to_global_code(&def.key) {
                        let mods = ray_to_global_modifiers(def.modifiers);
                        let hotkey = HotKey::new(Some(mods), code);
                        if let Ok(_) = manager.register(hotkey) {
                            self.registered.insert(key, hotkey);
                        }
                    }
                }
            }
        }
    }

    fn poll(&self, engine: &mut RayEngine) {
        if self.manager.is_none() { return; }
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == global_hotkey::HotKeyState::Pressed {
                for ((_, hotkey_id), hotkey) in &self.registered {
                    if hotkey.id() == event.id {
                        engine.bus().send(ray_api::RayEvent::HotkeyTriggered(hotkey_id.clone()));
                    }
                }
            }
        }
    }
}

fn render_settings_ui(engine: &mut RayEngine, active_settings_idx: &mut Option<usize>, settings_tab: &mut usize) {
    use macroquad::ui::{root_ui, hash};
    
    macroquad::ui::widgets::Window::new(
        hash!("settings_win"),
        vec2(50.0, 50.0),
        vec2(500.0, 600.0)
    )
    .label(if active_settings_idx.is_some() { "Extension Config" } else { "Framework Settings" })
    .ui(&mut root_ui(), |ui| {
        if let Some(idx) = *active_settings_idx {
            if let Some((name, _)) = engine.get_extension_info(idx) {
                ui.label(None, &format!("Configuring: {}", name));
                if ui.button(None, "<- Back to Settings") {
                    *active_settings_idx = None;
                }
                ui.separator();
                if let Err(e) = engine.render_extension_settings(idx, ui) {
                    ui.label(None, &format!("Error rendering settings: {}", e));
                }
            }
        } else {
            // Tab Buttons
            if ui.button(None, if *settings_tab == 0 { "[ Extensions ]" } else { " Extensions " }) {
                *settings_tab = 0;
            }
            ui.same_line(0.0);
            if ui.button(None, if *settings_tab == 1 { "[ Hotkeys ]" } else { " Hotkeys " }) {
                *settings_tab = 1;
            }
            ui.same_line(0.0);
            if ui.button(None, if *settings_tab == 2 { "[ Framework ]" } else { " Framework " }) {
                *settings_tab = 2;
            }
            ui.separator();

            if *settings_tab == 0 {
                render_extensions_tab(engine, ui, active_settings_idx);
            } else if *settings_tab == 1 {
                render_hotkeys_tab(engine, ui);
            } else {
                render_framework_tab(engine, ui);
            }
        }
    });
}

fn render_framework_tab(engine: &mut RayEngine, ui: &mut macroquad::ui::Ui) {
    use macroquad::ui::hash;
    ui.label(None, "Framework Configuration:");
    ui.separator();
    
    let mut vsync = engine.vsync_enabled;
    ui.checkbox(hash!("vsync_toggle"), "Enable VSync / Frame Limiter", &mut vsync);
    if vsync != engine.vsync_enabled {
        let _ = engine.set_vsync(vsync);
    }
    ui.label(None, "(VSync depends on OS/Driver support)");
}

fn render_extensions_tab(engine: &mut RayEngine, ui: &mut macroquad::ui::Ui, active_settings_idx: &mut Option<usize>) {
    use macroquad::ui::hash;
    ui.label(None, "Registered Extensions:");
    ui.separator();
    
    let count = engine.extension_count();
    for i in 0..count {
        if let Some((name, mut enabled)) = engine.get_extension_info(i) {
            ui.checkbox(hash!(i, "ext_check"), name, &mut enabled);
            
            if engine.extension_has_settings(i) {
                ui.same_line(0.0);
                if ui.button(None, "Configure") {
                    *active_settings_idx = Some(i);
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

fn render_hotkeys_tab(engine: &mut RayEngine, ui: &mut macroquad::ui::Ui) {
    ui.label(None, "Hotkey Registry:");
    ui.separator();

    // Group by applet
    let mut grouped: std::collections::BTreeMap<String, Vec<(&String, &ray_api::HotkeyDefinition)>> = std::collections::BTreeMap::new();
    for ((applet, id), def) in &engine.hotkey_registry.registered {
        grouped.entry(applet.clone()).or_default().push((id, def));
    }

    // Conflicts are now cached in the engine
    let conflicts = &engine.hotkey_registry.conflicts;

    for (applet, mut hotkeys) in grouped {
        ui.label(None, &format!("--- {} ---", applet));
        
        // Sort by scope: Global -> OS -> Local
        hotkeys.sort_by_key(|(_, def)| match def.scope {
            ray_api::HotkeyScope::Global => 0,
            ray_api::HotkeyScope::OS => 1,
            ray_api::HotkeyScope::Local => 2,
        });

        for (id, def) in hotkeys {
            let is_conflict = conflicts.iter().any(|(p1, p2)| {
                (p1.0 == applet && p1.1 == *id) || (p2.0 == applet && p2.1 == *id)
            });
            
            let mut mods_parts = Vec::new();
            if def.modifiers.contains(ray_api::HotkeyModifiers::CTRL) { mods_parts.push("Ctrl"); }
            if def.modifiers.contains(ray_api::HotkeyModifiers::SHIFT) { mods_parts.push("Shift"); }
            if def.modifiers.contains(ray_api::HotkeyModifiers::ALT) { mods_parts.push("Alt"); }
            if def.modifiers.contains(ray_api::HotkeyModifiers::LOGO) { mods_parts.push("Logo"); }
            
            let key_combo = if mods_parts.is_empty() {
                def.key.clone()
            } else {
                format!("{}+{}", mods_parts.join("+"), def.key)
            };

            let label = format!("{}: {} ({}) [{:?}]", id, def.description, key_combo, def.scope);
            
            if is_conflict {
                ui.label(None, &format!("!!! CONFLICT !!! {}", label));
            } else {
                ui.label(None, &label);
            }
        }
        ui.separator();
    }
}

fn render_console_ui(console_logs: &mut Vec<ray_api::LogEvent>) {
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

#[cfg(target_os = "macos")]
fn update_window_overlay(active: bool) {
    use objc::{msg_send, sel, sel_impl, class};
    unsafe {
        let ns_app: *mut objc::runtime::Object = msg_send![class!(NSApplication), sharedApplication];
        let mut ns_window: *mut objc::runtime::Object = msg_send![ns_app, keyWindow];
        
        if ns_window.is_null() {
            let windows: *mut objc::runtime::Object = msg_send![ns_app, windows];
            let count: usize = msg_send![windows, count];
            if count > 0 {
                ns_window = msg_send![windows, objectAtIndex: 0];
            }
        }

        if ns_window.is_null() { return; }
        
        if active {
            // NSWindowLevel: 21 (ScreenSaverWindowLevel)
            let _: () = msg_send![ns_window, setLevel: 21];
            let _: () = msg_send![ns_window, setStyleMask: 0]; // Borderless
            let _: () = msg_send![ns_window, setHasShadow: false];
        } else {
            let _: () = msg_send![ns_window, setLevel: 0]; // Normal
            let _: () = msg_send![ns_window, setStyleMask: 1 | 2 | 4 | 8]; // Titled | Closable | Miniaturizable | Resizable
            let _: () = msg_send![ns_window, setHasShadow: true];
        }
    }
}

#[cfg(windows)]
fn update_window_overlay(active: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == 0 { return; }
        
        if active {
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            SetWindowLongW(hwnd, GWL_STYLE, (style as u32 & !(WS_CAPTION | WS_THICKFRAME)) as i32);
        } else {
            SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            SetWindowLongW(hwnd, GWL_STYLE, (style as u32 | WS_CAPTION | WS_THICKFRAME) as i32);
        }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
fn update_window_overlay(_active: bool) {}
