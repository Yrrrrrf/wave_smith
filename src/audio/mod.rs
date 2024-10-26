// * external imports
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// * std imports
use std::error::Error;
use std::time::Duration;

// * module related
mod capture;
mod playback;
pub mod wave;

use capture::AudioCapture;
use playback::AudioPlayback;
use crate::encoding::Encoder;
use crate::AudioTransport;


#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 1,
            buffer_size: 1024,
        }
    }
}

pub struct AudioDevice {
    config: AudioConfig,
    encoder: Box<dyn Encoder>,
    capture: AudioCapture,
    playback: AudioPlayback,
}

impl AudioDevice {
    pub fn new(encoder: Box<dyn Encoder>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: AudioConfig::default(),
            encoder,
            capture: AudioCapture::new()?,
            playback: AudioPlayback::new()?,
        })
    }

    // Method to send a simple sync signal
    pub fn send_sync_signal(&mut self) -> Result<(), Box<dyn Error>> {
        // Generate a simple sync signal (440Hz tone for 100ms)
        let sample_rate = self.config.sample_rate;
        let duration = 0.1; // 100ms
        let frequency = 440.0; // A4 note
        
        let samples: Vec<f32> = (0..(sample_rate as f32 * duration) as usize)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (t * frequency * 2.0 * std::f32::consts::PI).sin() * 0.5
            })
            .collect();

        let stream = self.playback.play_samples(samples)?;
        
        // Wait for the sync signal to finish
        std::thread::sleep(Duration::from_secs_f32(duration * 1.5));
        
        Ok(())
    }

    // Method to listen for sync signal
    pub fn listen_for_sync(&mut self) -> Result<bool, Box<dyn Error>> {
        let stream = self.capture.start_listening()?;
        
        // Listen for 200ms (longer than sync signal to ensure capture)
        std::thread::sleep(Duration::from_millis(200));
        
        // Get the captured samples
        let samples = self.capture.get_samples();
        
        // Simple detection: check if any sample is above threshold
        let threshold = 0.1;
        let signal_detected = samples.iter().any(|&s| s.abs() > threshold);
        
        Ok(signal_detected)
    }
}

impl AudioTransport for AudioDevice {
    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        // First, send sync signal
        self.send_sync_signal()?;
        
        // TODO: Implement actual data sending
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        // First, wait for sync signal
        if !self.listen_for_sync()? {
            return Err("No sync signal detected".into());
        }
        
        // TODO: Implement actual data receiving
        Ok(Vec::new())
    }
}
