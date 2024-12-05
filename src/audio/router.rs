use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use cpal::traits::StreamTrait;

use crate::encoding::Encoder;

use super::capture::AudioCapture;
use super::playback::AudioPlayback;

pub struct AudioDev {
    capture: AudioCapture,
    playback: AudioPlayback,
    buffer: Arc<Mutex<Vec<f32>>>,
    pub encoder: Box<dyn Encoder>,
}

impl AudioDev {
    pub fn new(
        capture: AudioCapture, playback: AudioPlayback, encoder: Box<dyn Encoder>
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self { capture, playback, buffer: Arc::new(Mutex::new(Vec::new())), encoder})
    }

    // Start sending audio data through the router
    pub fn send(&self, data: &[u8]) -> Result<cpal::Stream, Box<dyn Error>> {
        // Play the encoded samples through the playback device
        self.playback.transmit(data)
    }

    // Listen for incoming audio data
    pub fn listen(&self) -> Result<(cpal::Stream, Vec<u8>), Box<dyn Error>> {
        // * Give some time for samples to be captured
        std::thread::sleep(Duration::from_millis(100));
        Ok((self.capture.start_listening()?, self.encoder.decode(&self.capture.get_samples())?))
    }

    // Stop all active streams
    pub fn stop(&self, streams: &[cpal::Stream]) -> Result<(), Box<dyn Error>> {
        for stream in streams { stream.pause()?; }
        Ok(())
    }
}
