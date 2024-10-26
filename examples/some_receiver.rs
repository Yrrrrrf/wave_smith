#![allow(unused)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dev_utils::{
    app_dt, error, warn, info, debug, trace,
    dlog::*,
    format::*,
};

#[derive(Debug)]
pub enum AudioError {
    DeviceNotFound,
    StreamError(cpal::BuildStreamError),
    PlayStreamError(cpal::PlayStreamError),
    PauseStreamError(cpal::PauseStreamError),
}

pub struct AudioReceiver {
    device: cpal::Device,
    config: cpal::StreamConfig,
    received_samples: Arc<Mutex<Vec<f32>>>,
    signal_threshold: Arc<Mutex<f32>>,
    peak_signal: Arc<Mutex<f32>>,
}

impl AudioReceiver {
    pub fn new(signal_threshold: f32) -> Result<Self, AudioError> {
        trace!("{}", "Initializing AudioReceiver...".style(Style::Italic));
        
        let host = cpal::default_host();
        debug!("Using audio host: {:?}", host.id());

        let device = host.default_input_device()
            .ok_or_else(|| {
                error!("{}", "No input device found".color(RED).style(Style::Bold));
                AudioError::DeviceNotFound
            })?;

        let supported_config = device.default_input_config()
            .map_err(|e| {
                error!("Failed to get default input config: {}", e.to_string().color(RED));
                AudioError::DeviceNotFound
            })?;

        info!("╔══════════════════════════════════════════");
        info!("║ {} {}", 
            "Input Device:".color(GREEN).style(Style::Bold),
            device.name().unwrap_or_default().color(YELLOW)
        );
        debug!("║ {} {:#?}", 
            "Configuration:".color(GREEN).style(Style::Bold),
            supported_config
        );
        info!("║ {} {}", 
            "Signal Threshold:".color(GREEN).style(Style::Bold),
            signal_threshold.to_string().color(CYAN)
        );
        info!("╚══════════════════════════════════════════");

        Ok(Self {
            device,
            config: supported_config.config(),
            received_samples: Arc::new(Mutex::new(Vec::new())),
            signal_threshold: Arc::new(Mutex::new(signal_threshold)),
            peak_signal: Arc::new(Mutex::new(0.0)),
        })
    }

    pub fn start_listening(&self) -> Result<cpal::Stream, AudioError> {
        let received_samples = Arc::clone(&self.received_samples);
        let threshold = Arc::clone(&self.signal_threshold);
        let peak_signal = Arc::clone(&self.peak_signal);

        self.device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &_| {
                received_samples.lock().unwrap().extend_from_slice(data);
                
                if let Some(&sample) = data.iter().max_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap()) {
                    let current_threshold = *threshold.lock().unwrap();
                    let signal_strength = sample.abs();
                    
                    // Update peak signal if current signal is stronger
                    let mut peak = peak_signal.lock().unwrap();
                    if signal_strength > *peak {
                        *peak = signal_strength;
                    }

                    if signal_strength > current_threshold {
                        // Create a visual signal strength meter
                        let meter_length = (signal_strength * 50.0) as usize;
                        let meter = "█".repeat(meter_length);
                        let signal_color = if signal_strength > current_threshold * 2.0 {
                            RED
                        } else if signal_strength > current_threshold * 1.5 {
                            YELLOW
                        } else {
                            GREEN
                        };

                        info!("▶ Signal: {} {}", 
                            signal_strength.to_string().color(signal_color).style(Style::Bold),
                            meter.color(signal_color)
                        );
                    }
                }
            },
            |err| error!("{} {}", 
                "Stream error:".style(Style::Bold),
                err.to_string().color(RED)
            ),
            None,
        ).map_err(AudioError::StreamError)
    }

    pub fn analyze_received_audio(&self) -> bool {
        let received = self.received_samples.lock().unwrap();
        let threshold = *self.signal_threshold.lock().unwrap();
        
        match received.as_slice() {
            [] => {
                debug!("{}", "No audio data received".color(YELLOW).style(Style::Italic));
                false
            },
            samples => {
                let max_signal = samples.iter().map(|s| s.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
                if max_signal > threshold {
                    info!("╔════════════════════════════════════");
                    info!("║ {} {:.3}", "Peak Signal:".color(GREEN).style(Style::Bold), max_signal);
                    info!("║ {} {:.3}", "Threshold:".color(GREEN).style(Style::Bold), threshold);
                    info!("╚════════════════════════════════════");
                    true
                } else {
                    debug!("Signal below threshold: {:.3} < {:.3}", max_signal, threshold);
                    false
                }
            }
        }
    }

    pub fn adjust_threshold(&self, new_threshold: f32) {
        *self.signal_threshold.lock().unwrap() = new_threshold;
        info!("{} {}", 
            "Adjusted signal threshold to:".color(BLUE).style(Style::Bold),
            new_threshold.to_string().color(CYAN)
        );
    }

    pub fn get_peak_signal(&self) -> f32 {
        *self.peak_signal.lock().unwrap()
    }

    pub fn clear_samples(&self) {
        self.received_samples.lock().unwrap().clear();
        *self.peak_signal.lock().unwrap() = 0.0;
    }
}

fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    let args: Vec<String> = std::env::args().collect();
    let signal_threshold = args.get(1)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.01);

    println!("\n{}", "╔═══════════════════════════════════════════════".color(CYAN));
    println!("{} {}", 
        "║".color(CYAN),
        "AUDIO RECEIVER".color(WHITE).style(Style::Bold)
    );
    println!("{}", "╠═══════════════════════════════════════════════".color(CYAN));
    println!("{} {} {}", 
        "║".color(CYAN),
        "Signal Threshold:".color(WHITE).style(Style::Bold),
        signal_threshold.to_string().color(GREEN)
    );
    println!("{}\n", "╚═══════════════════════════════════════════════".color(CYAN));

    match AudioReceiver::new(signal_threshold) {
        Ok(receiver) => {
            let stream = receiver.start_listening()
                .expect("Failed to start listening");
            
            stream.play().expect("Failed to play stream");

            info!("{}", "Listening for incoming signals...".color(GREEN).style(Style::Bold));
            info!("{}", "Press Ctrl+C to stop".color(YELLOW).style(Style::Italic));

            // Dynamic threshold adjustment based on background noise
            let mut consecutive_silent_periods = 0;
            let mut noise_floor = signal_threshold;

            loop {
                std::thread::sleep(Duration::from_millis(500));
                if receiver.analyze_received_audio() {
                    consecutive_silent_periods = 0;
                } else {
                    consecutive_silent_periods += 1;
                    
                    // Auto-adjust threshold if we see consistent background noise
                    if consecutive_silent_periods > 10 {
                        let peak = receiver.get_peak_signal();
                        if peak > 0.0 && peak < noise_floor {
                            noise_floor = peak * 1.5;
                            receiver.adjust_threshold(noise_floor);
                            consecutive_silent_periods = 0;
                        }
                    }
                }

                // Clear samples periodically to avoid memory growth
                receiver.clear_samples();
            }
        },
        Err(e) => error!("{} {:?}", 
            "Failed to create receiver:".style(Style::Bold),
            e
        ),
    }
}
