use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{error::Error, sync::Arc};

use crate::encoding::Encoder;


pub struct AudioPlayback {
    device: cpal::Device,      // The physical output device (speakers)
    config: cpal::StreamConfig, // Device configuration
    encoder: Box<dyn Encoder>, // The encoder instance for signal processing
}

impl AudioPlayback {
    /// Creates a new AudioPlayback with the default output device and encoder
    pub fn new(encoder: Box<dyn Encoder>) -> Result<Self, Box<dyn Error>> {
        Self::new_with_device(
            cpal::default_host()
                .default_output_device()
                .ok_or("No output device found")?,
            encoder
        )
    }

    /// Creates a new AudioPlayback with a specific output device and encoder
    pub fn new_with_device(device: cpal::Device, encoder: Box<dyn Encoder>) -> Result<Self, Box<dyn Error>> {
        let config = device.default_output_config()?.config();
        Ok(Self { device, config, encoder })
    }

    /// Send data through the encoder and play it with volume control
    pub fn transmit_with_volume(
        &self, 
        data: &[u8], 
        volume: f32
    ) -> Result<cpal::Stream, Box<dyn Error>> {
        // Encode the data into audio samples
        let samples = self.encoder.encode(data)?;
        
        let channels = self.config.channels as usize;
        let samples = Arc::new(samples);
        let samples_clone = Arc::clone(&samples);

        let stream = self.build_output_stream(samples_clone, channels, volume)?;
        stream.play()?;
        
        Ok(stream)
    }

    /// Send data through the encoder and play it (with default volume = 1.0)
    pub fn transmit(&self, data: &[u8]) -> Result<cpal::Stream, Box<dyn Error>> {
        self.transmit_with_volume(data, 1.0)
    }

    /// Get device information
    pub fn device_info(&self) -> Result<DeviceInfo, Box<dyn Error>> {
        Ok(DeviceInfo {name: self.device.name()?,  config: self.config.clone()})
    }

    // Private helper methods
    fn build_output_stream(
        &self,
        samples: Arc<Vec<f32>>,
        channels: usize,
        volume: f32
    ) -> Result<cpal::Stream, Box<dyn Error>> {
        let mut sample_clock = 0;

        let stream = self.device.build_output_stream(
            &self.config,
            move |data: &mut [f32], _: &_| {
                for frame in data.chunks_mut(channels) {
                    let sample = if sample_clock >= samples.len() {
                        0.0 // Output silence
                    } else {
                        samples[sample_clock] * volume // Apply volume
                    };
                    // Copy sample to all channels
                    frame.iter_mut().for_each(|s| *s = sample);
                    sample_clock += 1;
                }
            },
            |err| eprintln!("Error in output stream: {}", err),
            None,
        )?;

        Ok(stream)
    }
}

/// Structure to hold device information
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub config: cpal::StreamConfig,
}

#[cfg(test)]
mod tests {
    use crate::encoding::FSKEncoder;

    use super::*;

    #[test]
    fn test_default_device() -> Result<(), Box<dyn Error>> {
        let encoder = Box::new(FSKEncoder::default());
        let playback = AudioPlayback::new(encoder)?;
        let info = playback.device_info()?;
        println!("Default device: {}", info.name);
        Ok(())
    }

    #[test]
    fn test_specific_device() -> Result<(), Box<dyn Error>> {
        let host = cpal::default_host();
        if let Some(device) = host.output_devices()?.next() {
            let encoder = Box::new(FSKEncoder::default());
            let playback = AudioPlayback::new_with_device(device, encoder)?;
            let info = playback.device_info()?;
            println!("Specific device: {}", info.name);
            Ok(())
        } else {
            Ok(()) // Skip test if no devices available
        }
    }

    #[test]
    fn test_transmit_data() -> Result<(), Box<dyn Error>> {
        let encoder = Box::new(FSKEncoder::default());
        let playback = AudioPlayback::new(encoder)?;
        
        // Test data transmission
        let test_data = vec![0xAA, 0xBB, 0xCC]; // Test pattern
        let stream = playback.transmit(&test_data)?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }

    #[test]
    fn test_volume_control() -> Result<(), Box<dyn Error>> {
        let encoder = Box::new(FSKEncoder::default());
        let playback = AudioPlayback::new(encoder)?;
        let test_data = vec![0xAA, 0xBB, 0xCC];
        
        // Test different volume levels
        let mut volume = 0.0;
        while volume <= 1.0 {
            let stream = playback.transmit_with_volume(&test_data, volume)?;
            std::thread::sleep(std::time::Duration::from_millis(100));
            volume += 0.1;
        }
        Ok(())
    }
}