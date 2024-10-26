// * mod.rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::error::Error;
use std::time::Duration;

pub mod capture;
pub mod playback;
mod buffer;
pub mod wave;

use capture::AudioCapture;
use playback::AudioPlayback;

use crate::encoding::Encoder;
use crate::config::{AudioConfig, AudioConstant};
use wave::{PulseDetector, WaveGenerator};

pub struct AudioDevice {
    config: AudioConfig,
    encoder: Box<dyn Encoder>,
    capture: AudioCapture,
    playback: AudioPlayback,
    wave_gen: WaveGenerator,
    pulse_detector: PulseDetector,
}

impl AudioDevice {
    pub fn new(encoder: Box<dyn Encoder>) -> Result<Self, Box<dyn Error>> {
        let config = AudioConfig::default();

        Ok(Self {
            encoder,
            capture: AudioCapture::new()?,
            playback: AudioPlayback::new()?,
            wave_gen: WaveGenerator::new(config.clone()),
            pulse_detector: PulseDetector::new(config.clone()),
            config,
        })
    }

    pub fn with_config(encoder: Box<dyn Encoder>, config: AudioConfig) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            encoder,
            capture: AudioCapture::new()?,
            playback: AudioPlayback::new()?,
            wave_gen: WaveGenerator::new(config.clone()),
            pulse_detector: PulseDetector::new(config.clone()),
            config,
        })
    }

    pub fn send_sync_signal(&mut self) -> Result<(), Box<dyn Error>> {
        let sync_samples = self.wave_gen.generate_sync_tone();
        let stream = self.playback.play_samples(sync_samples)?;
        
        // Wait for the sync signal to finish
        std::thread::sleep(Duration::from_secs_f32(
            self.config.sync_duration() * 1.5
        ));
        
        Ok(())
    }

    pub fn listen_for_sync(&mut self) -> Result<bool, Box<dyn Error>> {
        let stream = self.capture.start_listening()?;
        
        // Listen for twice the sync duration to ensure capture
        std::thread::sleep(Duration::from_secs_f32(
            self.config.sync_duration() * 2.0
        ));
        
        let samples = self.capture.get_samples();
        let pulses = self.pulse_detector.detect_pulses(&samples);
        
        Ok(!pulses.is_empty())
    }
}
