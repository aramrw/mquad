# Clipboard Integration Design

A unified, synchronous API for reading from and writing to the system clipboard within the Ray framework.

## 1. Overview
The Clipboard Integration provides a thread-safe (main-thread only) wrapper around the `arboard` crate. It allows extensions to interact with the OS clipboard without managing their own dependencies or handling platform-specific locking issues.

## 2. Architecture

### 2.1 Component: RayEngine
The `RayEngine` will act as the owner of the clipboard instance.
- **Field:** `clipboard: Option<arboard::Clipboard>`
- **Reasoning:** `arboard` instances are not `Send` or `Sync` on all platforms. Keeping it in the `RayEngine` ensures it remains on the main thread where Macroquad and extension loops execute.

### 2.2 Component: RayContext
The `RayContext` will be updated to carry a mutable reference to the clipboard.
- **Field:** `clipboard: Option<&'a mut arboard::Clipboard>`
- **Methods:**
    - `clipboard_read(&mut self) -> Option<String>`
    - `clipboard_write(&mut self, text: String)`

## 3. Data Flow

### 3.1 Reading
1. Extension calls `ctx.clipboard_read()`.
2. `RayContext` attempts to use the internal `arboard` instance to fetch text.
3. If successful, returns `Some(String)`.
4. If failure (locked) or non-text data, returns `None`.

### 3.2 Writing
1. Extension calls `ctx.clipboard_write("text")`.
2. `RayContext` attempts to push text to the OS via `arboard`.
3. Errors are logged to the `RayEventBus` and visible in the Debug Console.

## 4. Implementation Details

### 4.1 Dependency Management
`arboard` is already a workspace dependency and used by `ray-runner-macroquad`. It will be added as a dependency for `ray-core`.

### 4.2 Lifecycle
- **Initialization:** Created in `RayEngine::new()`.
- **Shutdown:** Automatically dropped when `RayEngine` is dropped.

## 5. Testing Strategy
- **Manual Verification:** Use a test extension that reads the clipboard on a hotkey and logs the result to the console, and writes a test string on another hotkey.
- **Unit Tests:** Mock the `RayContext` in `ray-core` to verify that read/write calls propagate correctly (if possible given `arboard` constraints).
