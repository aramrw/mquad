# Hotkey Manager Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Optimize hotkey polling performance by pre-parsing keys into KeyCodes and add documentation to the hotkey logic.

**Architecture:** Update `HotkeyDefinition` (or `HotkeyRegistry`) to store the resolved `macroquad::input::KeyCode` to avoid per-frame string parsing. Add doc comments to helper functions.

**Tech Stack:** Rust, macroquad, ray-api

---

### Task 1: Update HotkeyDefinition in ray-api

**Files:**
- Modify: `ray-api/src/lib.rs`

- [ ] **Step 1: Add internal_keycode field to HotkeyDefinition**

Add an optional `KeyCode` field to `HotkeyDefinition`. Since `ray-api` shouldn't depend on `macroquad` directly if possible, we might need to handle this differently or just accept the dependency if it's already there. 

Wait, let's check `ray-api/Cargo.toml` first.

### Task 2: Optimize string_to_keycode and cache KeyCode

**Files:**
- Modify: `ray-core/src/lib.rs`

- [ ] **Step 1: Add documentation to `string_to_keycode` and `is_hotkey_pressed`**
- [ ] **Step 2: Update `HotkeyRegistry` to store resolved KeyCodes**
- [ ] **Step 3: Pre-parse KeyCode during `register_hotkey`**
- [ ] **Step 4: Update `is_hotkey_pressed` to take `Option<KeyCode>` directly**

---
