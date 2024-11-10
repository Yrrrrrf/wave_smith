use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct AudioCapture {
    device: cpal::Device,      // The physical input device (microphone)
    config: cpal::StreamConfig, // Device configuration
    samples: Arc<Mutex<Vec<f32>>>, // Buffer for captured audio data
}

impl AudioCapture {
    /// Creates a new AudioCapture with the default input device
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device found")?;
        
        Self::new_with_device(device)
    }

    /// Creates a new AudioCapture with a specific input device
    pub fn new_with_device(device: cpal::Device) -> Result<Self, Box<dyn Error>> {
        let config = device
            .default_input_config()?
            .config();

        Ok(Self {
            device,
            config,
            samples: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Start listening for audio input
    pub fn start_listening(&self) -> Result<cpal::Stream, Box<dyn Error>> {
        let samples = Arc::clone(&self.samples);

        let stream = self.device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &_| {
                let mut samples = samples.lock().unwrap();
                samples.extend_from_slice(data);
            },
            |err| eprintln!("Error in input stream: {}", err),
            None,
        )?;

        stream.play()?;
        Ok(stream)
    }

    /// Get captured samples and clear the buffer
    pub fn get_samples(&self) -> Vec<f32> {
        let mut samples = self.samples.lock().unwrap();
        let result = samples.clone();
        samples.clear();
        result
    }

    /// Get device information
    pub fn device_info(&self) -> Result<DeviceInfo, Box<dyn Error>> {
        Ok(DeviceInfo {name: self.device.name()?, config: self.config.clone()})
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
    use super::*;

    #[test]
    fn test_default_device() -> Result<(), Box<dyn Error>> {
        let capture = AudioCapture::new()?;
        let info = capture.device_info()?;
        println!("Default device: {}", info.name);
        Ok(())
    }

    #[test]
    fn test_specific_device() -> Result<(), Box<dyn Error>> {
        let host = cpal::default_host();
        if let Some(device) = host.input_devices()?.next() {
            let capture = AudioCapture::new_with_device(device)?;
            let info = capture.device_info()?;
            println!("Specific device: {}", info.name);
            Ok(())
        } else {
            Ok(()) // Skip test if no devices available
        }
    }
}
