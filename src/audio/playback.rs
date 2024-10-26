// * OUTPUT
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{error::Error, sync::Arc};

pub struct AudioPlayback {
    device: cpal::Device,      // The physical output device (speakers)
    config: cpal::StreamConfig, // Device configuration
}

impl AudioPlayback {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();
        // Get default output device (usually the speakers)
        let device = host.default_output_device().ok_or("No output device found")?;
        let config = device.default_output_config()?.config();  // Get default config for output device

        println!("Output device: {}", device.name()?);
        println!("Default output config: {:?}", config);

        Ok(Self {
            device,
            config,
        })
    }

    // Play audio data
    pub fn play_samples(&self, samples: Vec<f32>) -> Result<cpal::Stream, Box<dyn Error>> {
        let sample_rate = self.config.sample_rate.0 as f32;
        let channels = self.config.channels as usize;
        
        let mut sample_clock = 0;
        let samples = Arc::new(samples);
        let samples_clone = Arc::clone(&samples);

        // Create an output stream
        let stream = self.device.build_output_stream(
            &self.config,
            move |data: &mut [f32], _: &_| {
                // This closure is called when the device needs more audio data
                for frame in data.chunks_mut(channels) {
                    if sample_clock >= samples_clone.len() {
                        // If we've played all samples, output silence
                        frame.iter_mut().for_each(|sample| *sample = 0.0);
                    } else {
                        // Copy our sample to all channels
                        frame.iter_mut().for_each(|sample| {
                            *sample = samples_clone[sample_clock];
                        });
                    }
                    sample_clock += 1;
                }
            },
            |err| eprintln!("Error in output stream: {}", err),
            None,
        )?;

        // Start the stream
        stream.play()?;
        
        Ok(stream)
    }
}
