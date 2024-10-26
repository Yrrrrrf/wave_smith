#![allow(unused)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::time::Duration;

// Import both logging and formatting utilities
use dev_utils::{
    app_dt, error, warn, info, debug, trace,
    dlog::*,
    format::*,  // Add formatting capabilities
};

#[derive(Debug)]
pub enum AudioError {
    DeviceNotFound,
    StreamError(cpal::BuildStreamError),
    PlayStreamError(cpal::PlayStreamError),
}

pub struct AudioSender {
    device: cpal::Device,
    config: cpal::StreamConfig,
}

impl AudioSender {
    pub fn new() -> Result<Self, AudioError> {
        trace!("{}", "Initializing AudioSender...".style(Style::Italic));
        
        let host = cpal::default_host();
        debug!("Using audio host: {:?}", host.id());

        let device = host.default_output_device()
            .ok_or_else(|| {
                error!("{}", "No output device found".color(RED).style(Style::Bold));
                AudioError::DeviceNotFound
            })?;

        let supported_config = device.default_output_config()
            .map_err(|e| {
                error!("Failed to get default output config: {}", e.to_string().color(RED));
                AudioError::DeviceNotFound
            })?;

        // Use box drawing characters for a nice header
        info!("╔══════════════════════════════════════════");
        info!("║ {} {}", 
            "Output Device:".color(GREEN).style(Style::Bold), 
            device.name().unwrap_or_default().color(YELLOW)
        );
        debug!("║ {} {:#?}", 
            "Configuration:".color(GREEN).style(Style::Bold),
            supported_config
        );
        info!("╚══════════════════════════════════════════");

        Ok(Self {
            device,
            config: supported_config.config(),
        })
    }

    pub fn send_message(&self, message: &str) -> Result<cpal::Stream, AudioError> {
        let sample_rate = self.config.sample_rate.0 as f32;
        let channels = self.config.channels as usize;

        // todo: psdffsdfdsfdffdf
        debug!("Message details:");
        debug!("├─ Length: {} characters", message.len().to_string().color(CYAN));
        debug!("├─ Sample rate: {} Hz", sample_rate.to_string().color(CYAN));
        debug!("└─ Channels: {}", channels.to_string().color(CYAN));
        
        let frequency = 440.0;
        let mut sample_clock = 0f32;
        let duration = message.len() as f32 * 0.1;
        
        info!("{}", "Generating audio signal...".color(BLUE).style(Style::Bold));
        trace!("Signal parameters:");
        trace!("├─ Frequency: {}Hz", frequency.to_string().color(MAGENTA));
        trace!("└─ Duration: {}s", duration.to_string().color(MAGENTA));

        let stream = self.device.build_output_stream(
            &self.config,
            move |data: &mut [f32], _: &_| {
                for frame in data.chunks_mut(channels) {
                    let time = sample_clock / sample_rate;
                    if time < duration {
                        let sample = (time * frequency * 2.0 * std::f32::consts::PI).sin() * 0.5;
                        frame.iter_mut().for_each(|s| *s = sample);
                    } else {
                        frame.iter_mut().for_each(|s| *s = 0.0);
                    }
                    sample_clock += 1.0;
                }
            },
            |err| error!("{} {}", "Stream error:".style(Style::Bold), err.to_string().color(RED)),
            None,
        ).map_err(|e| {
            error!("{} {}", 
                "Failed to build output stream:".style(Style::Bold), 
                e.to_string().color(RED)
            );
            AudioError::StreamError(e)
        })?;

        stream.play().map_err(|e| {
            error!("{} {}", 
                "Failed to play stream:".style(Style::Bold), 
                e.to_string().color(RED)
            );
            AudioError::PlayStreamError(e)
        })?;

        info!("{}", "Audio stream started successfully".color(GREEN).style(Style::Bold));
        Ok(stream)
    }
}

fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    // Print a fancy header
    println!("\n{}", "╔═══════════════════════════════════════════════".color(CYAN));
    println!("{} {}", 
        "║".color(CYAN),
        "AUDIO SENDER".color(WHITE).style(Style::Bold)
    );
    println!("{}\n", "╚═══════════════════════════════════════════════".color(CYAN));

    let message = std::env::args().nth(1).unwrap_or_else(|| {
        warn!("{}", "No message provided, using default".color(YELLOW).style(Style::Italic));
        "TEST".to_string()
    });

    info!("Starting transmission:");
    info!("└─ Message: {}", message.color(GREEN).style(Style::Bold));

    match AudioSender::new() {
        Ok(sender) => {
            match sender.send_message(&message) {
                Ok(stream) => {
                    let sleep_duration = message.len() as f32 * 0.1 * 2.0;
                    
                    // Show a progress message
                    info!("{}", "Transmission in progress...".color(BLUE).style(Style::Bold));
                    info!("└─ Duration: {}s", sleep_duration.to_string().color(CYAN));
                    
                    std::thread::sleep(Duration::from_secs_f32(sleep_duration));
                    debug!("{}", "Cleaning up...".style(Style::Italic));
                    drop(stream);
                    
                    // Show success message in a box
                    println!("\n{}", "╔═══════════════════════════════════════".color(GREEN));
                    println!("{} {}", 
                        "║".color(GREEN),
                        "Message sent successfully!".color(GREEN).style(Style::Bold)
                    );
                    println!("{}\n", "╚═══════════════════════════════════════".color(GREEN));
                },
                Err(e) => error!("{} {:?}", 
                    "Failed to send message:".style(Style::Bold), 
                    e
                ),
            }
        },
        Err(e) => error!("{} {:?}", 
            "Failed to initialize sender:".style(Style::Bold), 
            e
        ),
    }
}
