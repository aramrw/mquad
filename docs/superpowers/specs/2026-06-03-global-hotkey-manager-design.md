# Global Hotkey Manager Design

**Date:** 2026-06-03
**Status:** Draft
**Topic:** Global and Local Hotkey Management for the Ray Framework.

## 1. Purpose
Provide a centralized system for applets and the framework engine to register, manage, and respond to keyboard shortcuts (hotkeys). This system supports three scopes:
- **Framework Global:** Works anywhere within the Ray application.
- **OS Global:** Works even when Ray is minimized or out of focus.
- **Applet Local:** Works only when the specific applet is the active tab.

## 2. Architecture

### 2.1 Components

#### `ray-api` (The Interface)
- **`HotkeyScope`**: Enum defining `Global`, `OS`, and `Local`.
- **`HotkeyModifiers`**: Bitflags for Ctrl, Shift, Alt, Logo.
- **`HotkeyDefinition`**: Struct containing ID, Key, Modifiers, Scope, and a human-readable description.
- **`RayEvent::HotkeyTriggered(String)`**: The event sent when a hotkey is matched.

#### `ray-core` (The Registry)
- **`HotkeyRegistry`**: A collection of all registered hotkeys.
- **Conflict Detection**: A system that identifies when two hotkeys share the same key/modifier combination.
- **Persistence**: Integration with `framework_settings.db` to allow user-defined rebinds.

#### `ray-runner-macroquad` (The Dispatcher)
- **Macroquad Polling**: Checks `is_key_pressed` every frame against the registry.
- **OS-Global Polling**: Owns the `GlobalHotKeyManager` (from the `global-hotkey` crate) and polls its receiver.
- **Event Injection**: Injects `HotkeyTriggered` events into the `RayEventBus`.

### 2.2 Data Flow
1. **Registration**: During `init()`, an applet calls `ctx.register_hotkey(...)`.
2. **Detection**: The runner detects a physical key press.
3. **Validation**: The engine checks the registry:
    - If `OS` or `Global`: Trigger immediately.
    - If `Local`: Only trigger if `active_extension_idx` matches the registering applet.
4. **Dispatch**: A `RayEvent::HotkeyTriggered("my_id")` is sent to the bus.
5. **Handling**: The applet receives the event in `on_event` and acts.

## 3. Conflict Resolution
- **Prioritization**: OS > Global > Local.
- **Warnings**: If a conflict exists, the Engine still registers both but logs a warning. The UI will highlight conflicts in the Settings page.

## 4. Persistence
- Hotkeys are stored in a `hotkeys` table in SQLite.
- Columns: `applet_id`, `hotkey_id`, `key_code`, `modifiers`.
- On startup, the Engine loads these overrides and replaces the default values provided by applets during registration.

## 5. UI Requirements
- **Settings Page**: A new "Hotkeys" tab showing a list of all registered keys, grouped by Applet.
- **Rebinding**: Users can click a hotkey to enter "Record Mode" and press a new combination.
- **Conflict Indicators**: Red text or icons next to conflicting hotkeys.

## 6. Testing Strategy
- **Unit Tests (`ray-core`)**: Test the registry's conflict detection logic and persistence.
- **Integration Tests**: Mock the event bus to ensure `HotkeyTriggered` events are dispatched correctly for different scopes.
- **Manual Verification**: Test OS-global keys with the window minimized.
