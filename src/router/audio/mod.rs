use crate::YomichanApp;
use macroquad::prelude::*;
use macroquad::ui::Ui;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub struct AudioState {
    pub is_recording: bool,
    pub buffer: Vec<f32>,
    pub spectrum: Vec<f32>,
    pub samples_to_show: usize,
    pub process: Option<Child>,
    pub receiver: Option<mpsc::Receiver<Vec<f32>>>,
    pub error_receiver: Option<mpsc::Receiver<String>>,
    pub audio_texture: Option<Texture2D>,
    pub device_index: i32,
    pub data_received_count: u64,
    pub current_volume: f32,
    pub error_log: String,
    pub device_list: String,
    pub show_log_window: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            is_recording: false,
            buffer: vec![0.0; 1024],
            spectrum: vec![0.0; 512],
            samples_to_show: 1024,
            process: None,
            receiver: None,
            error_receiver: None,
            audio_texture: None,
            device_index: 0,
            data_received_count: 0,
            current_volume: 0.0,
            error_log: String::new(),
            device_list: String::new(),
            show_log_window: false,
        }
    }
}

impl YomichanApp {
    pub fn list_devices(&mut self) {
        let output = Command::new("ffmpeg")
            .args(&["-f", "avfoundation", "-list_devices", "true", "-i", ""])
            .output()
            .expect("Failed to run ffmpeg");

        self.audio_state.device_list = String::from_utf8_lossy(&output.stderr).to_string();
    }

    pub fn start_recording(&mut self) {
        if self.audio_state.is_recording {
            return;
        }

        self.audio_state.error_log.clear();
        let (tx, rx) = mpsc::channel();
        let (etx, erx) = mpsc::channel();
        let device = format!("{}", self.audio_state.device_index);

        println!("[AUDIO] Starting FFmpeg with device index: {}", device);

        // On macOS, -i "none:0" means "no video, audio device 0"
        let device_arg = format!("none:{}", device);

        let mut child = Command::new("ffmpeg")
            .args(&[
                "-f",
                "avfoundation",
                "-i",
                &device_arg,
                "-f",
                "s16le",
                "-ac",
                "1",
                "-ar",
                "44100",
                "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg");

        let stdout = child.stdout.take().expect("Failed to get stdout");
        let stderr = child.stderr.take().expect("Failed to get stderr");

        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                let _ = etx.send(line);
            }
        });

        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut buffer = [0u8; 2048];

            while let Ok(n) = reader.read(&mut buffer) {
                if n == 0 {
                    break;
                }

                let mut samples = Vec::with_capacity(n / 2);
                for i in (0..n).step_by(2) {
                    if i + 1 < n {
                        let sample = i16::from_le_bytes([buffer[i], buffer[i + 1]]);
                        samples.push(sample as f32 / 32768.0);
                    }
                }
                if tx.send(samples).is_err() {
                    break;
                }
            }
        });

        self.audio_state.process = Some(child);
        self.audio_state.receiver = Some(rx);
        self.audio_state.error_receiver = Some(erx);
        self.audio_state.is_recording = true;
    }

    pub fn stop_recording(&mut self) {
        if let Some(mut child) = self.audio_state.process.take() {
            let _ = child.kill();
        }
        self.audio_state.is_recording = false;
        self.audio_state.receiver = None;
        self.audio_state.error_receiver = None;
    }

    pub fn update_audio(&mut self) {
        if let Some(erx) = &self.audio_state.error_receiver {
            while let Ok(err) = erx.try_recv() {
                self.audio_state.error_log.push_str(&err);
                self.audio_state.error_log.push('\n');
                if self.audio_state.error_log.len() > 10000 {
                    self.audio_state.error_log.drain(0..2000);
                }
            }
        }

        if self.audio_state.audio_texture.is_none() {
            let initial_data = vec![127u8; self.audio_state.samples_to_show * 2 * 4];
            self.audio_state.audio_texture = Some(Texture2D::from_rgba8(
                self.audio_state.samples_to_show as u16,
                2,
                &initial_data,
            ));
        }

        if let Some(rx) = &self.audio_state.receiver {
            let mut received = false;
            while let Ok(samples) = rx.try_recv() {
                received = true;

                // Calculate volume of this batch
                let mut batch_vol = 0.0f32;
                if !samples.is_empty() {
                    batch_vol = samples.iter().map(|s| s.abs()).sum::<f32>() / samples.len() as f32;
                    self.audio_state.current_volume = batch_vol;
                }

                // Only count samples if there is actual signal (threshold 0.0001)
                if batch_vol > 0.0001 {
                    self.audio_state.data_received_count += samples.len() as u64;
                }

                for s in samples {
                    self.audio_state.buffer.push(s);
                    if self.audio_state.buffer.len() > self.audio_state.samples_to_show {
                        self.audio_state.buffer.remove(0);
                    }
                }
            }

            if received {
                // Throttled console debug
                static mut LAST_DEBUG_TIME: f64 = 0.0;
                unsafe {
                    let now = get_time();
                    if now - LAST_DEBUG_TIME > 5.0 && self.audio_state.current_volume > 0.0001 {
                        println!(
                            "[AUDIO] Signal detected! Vol: {:.6}",
                            self.audio_state.current_volume
                        );
                        LAST_DEBUG_TIME = now;
                    }
                }

                if self.audio_state.buffer.len() >= 1024 {
                    let mut fft_input = [0.0f32; 1024];
                    for (i, &s) in self.audio_state.buffer.iter().take(1024).enumerate() {
                        let window =
                            0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / 1023.0).cos());
                        fft_input[i] = s * window;
                    }

                    let spectrum = microfft::real::rfft_1024(&mut fft_input);
                    for i in 0..512 {
                        let magnitude = (spectrum[i].re.powi(2) + spectrum[i].im.powi(2)).sqrt();
                        let val = (magnitude * 50.0).log10().max(0.0) * 0.8;
                        self.audio_state.spectrum[i] = val.clamp(0.0, 1.0);
                    }
                }

                if let Some(tex) = &mut self.audio_state.audio_texture {
                    let mut bytes = Vec::with_capacity(self.audio_state.samples_to_show * 2 * 4);

                    for &s in &self.audio_state.buffer {
                        let gain_s = s * 2.0;
                        let val = ((gain_s + 1.0) * 0.5 * 255.0).clamp(0.0, 255.0) as u8;
                        bytes.push(val);
                        bytes.push(val);
                        bytes.push(val);
                        bytes.push(255);
                    }
                    while bytes.len() < self.audio_state.samples_to_show * 4 {
                        bytes.push(127);
                        bytes.push(127);
                        bytes.push(127);
                        bytes.push(255);
                    }

                    for i in 0..self.audio_state.samples_to_show {
                        let spec_idx = (i / 2).min(511);
                        let s = self.audio_state.spectrum[spec_idx];
                        let val = (s * 255.0).clamp(0.0, 255.0) as u8;
                        bytes.push(val);
                        bytes.push(val);
                        bytes.push(val);
                        bytes.push(255);
                    }

                    tex.update_from_bytes(self.audio_state.samples_to_show as u32, 2, &bytes);
                }
            }
        }
    }

    pub fn draw_audio_tab(&mut self, ui: &mut Ui) {
        use macroquad::ui::hash;
        ui.label(None, "Recording Settings");

        let mut idx = self.audio_state.device_index as f32;
        ui.slider(hash!(), "Device Index", 0.0..10.0, &mut idx);
        self.audio_state.device_index = idx as i32;

        if ui.button(None, "List Devices") {
            self.list_devices();
        }
        ui.same_line(0.0);

        let mut recording = self.audio_state.is_recording;
        ui.checkbox(
            hash!("rec_toggle"),
            "Enable Real-time Audio",
            &mut recording,
        );
        if recording != self.audio_state.is_recording {
            if recording {
                self.start_recording();
            } else {
                self.stop_recording();
            }
        }


        ui.same_line(1.0);

        // too many updates
        // ui.label(
        //     None,
        //     &format!("Samples Received: {}", self.audio_state.data_received_count),
        // );

        ui.checkbox(
            hash!("show_ffmpeg_log"),
            "Show Full FFmpeg Log",
            &mut self.audio_state.show_log_window,
        );

        if self.audio_state.show_log_window {
            macroquad::ui::widgets::Window::new(
                hash!("ffmpeg_log_win"),
                macroquad::math::vec2(screen_width() - 450.0, 110.0),
                macroquad::math::vec2(400.0, 500.0),
            )
            .label("FFmpeg Console Output")
            .ui(ui, |ui| {
                if ui.button(None, "Clear Log") {
                    self.audio_state.error_log.clear();
                }
                ui.separator();
                for line in self.audio_state.error_log.lines().rev().take(50) {
                    ui.label(None, line);
                }
            });
        }

        ui.separator();

        if !self.audio_state.device_list.is_empty() {
            macroquad::ui::widgets::Window::new(
                hash!("device_list_win"),
                macroquad::math::vec2(20.0, 450.0),
                macroquad::math::vec2(400.0, 300.0),
            )
            .label("Available Audio Devices")
            .ui(ui, |ui| {
                for line in self.audio_state.device_list.lines() {
                    ui.label(None, line);
                }
            });
        }
        ui.label(None, "Real-time Audio Signal:");

        let canvas_size = vec2(screen_width() - 40.0, 200.0);
        let pos = vec2(20.0, 250.0);

        draw_rectangle(
            pos.x,
            pos.y,
            canvas_size.x,
            canvas_size.y,
            Color::from_rgba(30, 30, 30, 255),
        );

        if self.audio_state.is_recording {
            let step = canvas_size.x / self.audio_state.samples_to_show as f32;
            let mid_y = pos.y + canvas_size.y / 2.0;

            for i in 0..self.audio_state.buffer.len().saturating_sub(1) {
                let x1 = pos.x + i as f32 * step;
                let y1 = mid_y + self.audio_state.buffer[i] * canvas_size.y / 2.0;
                let x2 = pos.x + (i + 1) as f32 * step;
                let y2 = mid_y + self.audio_state.buffer[i + 1] * canvas_size.y / 2.0;
                draw_line(x1, y1, x2, y2, 1.0, GREEN);
            }

            let spec_step = canvas_size.x / 512.0;
            for i in 0..511 {
                let x = pos.x + i as f32 * spec_step;
                let h = self.audio_state.spectrum[i] * canvas_size.y;
                draw_line(
                    x,
                    pos.y + canvas_size.y,
                    x,
                    pos.y + canvas_size.y - h,
                    2.0,
                    Color::from_rgba(255, 0, 255, 100),
                );
            }
        } else {
            ui.label(None, "  (Enable audio to see signal)");
        }

        ui.label(
            None,
            &format!("Buffer size: {}", self.audio_state.buffer.len()),
        );
    }
}
