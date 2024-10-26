// * INPUT
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct AudioCapture {
    device: cpal::Device,      // The physical input device (microphone)
    config: cpal::StreamConfig, // Device configuration
    samples: Arc<Mutex<Vec<f32>>>, // Buffer for captured audio data
}

impl AudioCapture {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();
        
        // Get default input device (usually the microphone)
        let device = host.default_input_device().ok_or("No input device found")?;

        // Get default config for input device
        let config = device.default_input_config()?.config();

        println!("Input device: {}", device.name()?);
        println!("Default input config: {:?}", config);

        Ok(Self {
            device,
            config,
            samples: Arc::new(Mutex::new(Vec::new())),
        })
    }

    // Start listening for audio input
    pub fn start_listening(&self) -> Result<cpal::Stream, Box<dyn Error>> {
        let samples = Arc::clone(&self.samples);

        // Create an input stream
        let stream = self.device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &_| {
                // This closure is called whenever new audio data arrives
                let mut samples = samples.lock().unwrap();
                samples.extend_from_slice(data);
            },
            |err| eprintln!("Error in input stream: {}", err),
            None,
        )?;

        // Start the stream
        stream.play()?;
        
        Ok(stream)
    }

    // Get captured samples
    pub fn get_samples(&self) -> Vec<f32> {
        let mut samples = self.samples.lock().unwrap();
        let result = samples.clone();
        samples.clear();
        result
    }
}
