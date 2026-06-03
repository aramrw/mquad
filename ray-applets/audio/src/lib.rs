use ray_api::{RayExtension, RayContext, RayEvent, AudioEvent};
use macroquad::prelude::*;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub struct AudioApplet {
    is_recording: bool,
    buffer: Vec<f32>,
    spectrum: Vec<f32>,
    samples_to_show: usize,
    process: Option<Child>,
    receiver: Option<mpsc::Receiver<Vec<f32>>>,
    error_receiver: Option<mpsc::Receiver<String>>,
    device_index: i32,
    current_volume: f32,
    error_log: String,
    device_list: String,
    show_log_window: bool,
}

impl AudioApplet {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            buffer: vec![0.0; 1024],
            spectrum: vec![0.0; 512],
            samples_to_show: 1024,
            process: None,
            receiver: None,
            error_receiver: None,
            device_index: 1, // Default to 1 (usually mic on macOS)
            current_volume: 0.0,
            error_log: String::new(),
            device_list: String::new(),
            show_log_window: false,
        }
    }

    fn list_devices(&mut self) {
        let output = Command::new("ffmpeg")
            .args(&["-f", "avfoundation", "-list_devices", "true", "-i", ""])
            .output()
            .expect("Failed to run ffmpeg");

        self.device_list = String::from_utf8_lossy(&output.stderr).to_string();
    }

    fn start_recording(&mut self) {
        if self.is_recording {
            return;
        }

        self.error_log.clear();
        let (tx, rx) = mpsc::channel();
        let (etx, erx) = mpsc::channel();
        let device = format!("{}", self.device_index);

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

        self.process = Some(child);
        self.receiver = Some(rx);
        self.error_receiver = Some(erx);
        self.is_recording = true;
    }

    fn stop_recording(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
        }
        self.is_recording = false;
        self.receiver = None;
        self.error_receiver = None;
    }
}

impl RayExtension for AudioApplet {
    fn name(&self) -> &str {
        "Audio"
    }

    fn init(&mut self, _ctx: &mut RayContext, _args: &clap::ArgMatches) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_event(&mut self, _ctx: &mut RayContext, event: &RayEvent) -> anyhow::Result<()> {
        if let RayEvent::Command(ray_api::RayCommand::Audio(cmd)) = event {
            match cmd {
                ray_api::AudioCommand::StartRecording => self.start_recording(),
                ray_api::AudioCommand::StopRecording => self.stop_recording(),
                ray_api::AudioCommand::ToggleRecording => {
                    if self.is_recording { self.stop_recording(); }
                    else { self.start_recording(); }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut RayContext) -> anyhow::Result<()> {
        if let Some(erx) = &self.error_receiver {
            while let Ok(err) = erx.try_recv() {
                self.error_log.push_str(&err);
                self.error_log.push('\n');
                if self.error_log.len() > 5000 {
                    self.error_log.drain(0..1000);
                }
            }
        }

        if let Some(rx) = &self.receiver {
            let mut received = false;
            while let Ok(samples) = rx.try_recv() {
                received = true;

                if !samples.is_empty() {
                    self.current_volume = samples.iter().map(|s| s.abs()).sum::<f32>() / samples.len() as f32;
                    ctx.bus.send(RayEvent::Audio(AudioEvent::Level(self.current_volume)));
                    ctx.bus.send(RayEvent::Audio(AudioEvent::Buffer(samples.clone())));
                }

                for s in samples {
                    self.buffer.push(s);
                    if self.buffer.len() > self.samples_to_show {
                        self.buffer.remove(0);
                    }
                }
            }

            if received {
                if self.buffer.len() >= 1024 {
                    let mut fft_input = [0.0f32; 1024];
                    for (i, &s) in self.buffer.iter().take(1024).enumerate() {
                        let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / 1023.0).cos());
                        fft_input[i] = s * window;
                    }

                    let res = microfft::real::rfft_1024(&mut fft_input);
                    for i in 0..512 {
                        let magnitude = (res[i].re.powi(2) + res[i].im.powi(2)).sqrt();
                        let val = (magnitude * 50.0).log10().max(0.0) * 0.8;
                        self.spectrum[i] = val.clamp(0.0, 1.0);
                    }
                    ctx.bus.send(RayEvent::Audio(AudioEvent::Spectrum(self.spectrum.clone())));
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, _ctx: &mut RayContext) -> anyhow::Result<()> {
        use macroquad::ui::{root_ui, hash};

        macroquad::ui::widgets::Window::new(
            hash!("audio_config"),
            vec2(20.0, 20.0),
            vec2(300.0, 200.0)
        )
        .label("Audio Configuration")
        .ui(&mut root_ui(), |ui| {
            let mut idx = self.device_index as f32;
            ui.slider(hash!("dev_idx"), "Device", 0.0..10.0, &mut idx);
            self.device_index = idx as i32;

            if ui.button(None, "List Devices") {
                self.list_devices();
            }

            let mut rec = self.is_recording;
            ui.checkbox(hash!("rec_toggle"), "Record", &mut rec);
            if rec != self.is_recording {
                if rec { self.start_recording(); }
                else { self.stop_recording(); }
            }
        });

        if !self.device_list.is_empty() {
             macroquad::ui::widgets::Window::new(
                hash!("dev_list"),
                vec2(340.0, 20.0),
                vec2(400.0, 300.0)
            )
            .label("Available Devices")
            .ui(&mut root_ui(), |ui| {
                for line in self.device_list.lines() {
                    ui.label(None, line);
                }
            });
        }

        // Visualization
        let pos = vec2(20.0, 300.0);
        let size = vec2(screen_width() - 40.0, 150.0);
        draw_rectangle(pos.x, pos.y, size.x, size.y, Color::from_rgba(20, 20, 20, 255));

        if self.is_recording {
            let step = size.x / self.samples_to_show as f32;
            let mid_y = pos.y + size.y / 2.0;
            for i in 0..self.buffer.len().saturating_sub(1) {
                draw_line(
                    pos.x + i as f32 * step,
                    mid_y + self.buffer[i] * size.y / 2.0,
                    pos.x + (i + 1) as f32 * step,
                    mid_y + self.buffer[i+1] * size.y / 2.0,
                    1.0,
                    GREEN
                );
            }
        }

        Ok(())
    }
}
