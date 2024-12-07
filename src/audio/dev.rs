use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use bytes::{Bytes, BytesMut};
use cpal::traits::StreamTrait;
use cpal::Stream;
use dev_utils::{dlog::*, format::*};

use crate::encoding::{Encoder, FSKEncoder};
use crate::proto::Frame;
use super::capture::AudioCapture;
use super::playback::AudioPlayback;

pub struct AudioDev {
    capture: AudioCapture,
    playback: AudioPlayback,
    buffer: Arc<Mutex<Vec<f32>>>,
    sequence: Arc<Mutex<u8>>,  // Track frame sequence numbers
}

impl AudioDev {
    pub fn new(
        capture: AudioCapture,
        playback: AudioPlayback
    ) -> Result<Self, Box<dyn Error>> {
        let buffer = Arc::default();
        let sequence = Arc::new(Mutex::new(0));
        Ok(Self { capture, playback, buffer, sequence })
    }

    /// Sends data by creating a frame and transmitting it
    pub fn send(&self, data: &[u8]) -> Result<Stream, Box<dyn Error>> {
        // Create a new frame with incrementing sequence number
        let mut seq = self.sequence.lock().unwrap();
        let frame = Frame::new(data, *seq)?;
        *seq = seq.wrapping_add(1);  // Increment sequence number
        
        // Serialize frame and transmit
        let frame_bytes = frame.serialize();
        info!("📤 Sending frame with sequence: {}", frame.sequence());
        self.playback.transmit(&frame_bytes)
    }

    /// Listens for incoming frames and processes them
    pub fn listen(&self) -> Result<(Stream, Vec<u8>), Box<dyn Error>> {
        // Start listening for audio samples
        let stream = self.capture.start_listening()?;
        
        // Give some time for samples to be captured
        std::thread::sleep(Duration::from_millis(100));
        
        // Get samples and try to decode them
        let samples = self.capture.get_samples();
        
        // First decode the audio samples into digital data
        let decoded_bytes = self.playback.encoder.decode(&samples)?;
        
        // Then try to deserialize into a frame
        if let Some(frame) = Frame::deserialize(Bytes::from(decoded_bytes))? {
            info!("📥 Received frame with sequence: {}", frame.sequence());
            Ok((stream, frame.payload().to_vec()))
        } else {
            // If no valid frame was found, return empty data
            Ok((stream, Vec::new()))
        }
    }

    /// Process continuous stream of samples looking for frames
    pub fn process_samples(&self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        // First decode the audio samples into digital data using FSK decoder
        if let Ok(decoded_bytes) = self.playback.encoder.decode(samples) {
            // Then try to deserialize into a frame
            if let Ok(Some(frame)) = Frame::deserialize(Bytes::from(decoded_bytes)) {
                info!("📥 Processed frame with sequence: {}", frame.sequence());
                return Ok(Some(frame.payload().to_vec()));
            }
        }
        Ok(None)
    }

    /// Monitors incoming audio continuously
    pub fn monitor(&self) -> Result<Stream, Box<dyn Error>> {
        let stream = self.capture.start_listening()?;
        
        // Create a new encoder specifically for monitoring
        let samples = Arc::clone(&self.capture.samples);
        
        std::thread::spawn(move || {
            // Create a new FSKEncoder instance for this thread
            let decoder = FSKEncoder::default();

            loop {
                // Get accumulated samples
                let current_samples = {
                    let mut samples_lock = samples.lock().unwrap();
                    let result = samples_lock.clone();
                    samples_lock.clear();
                    result
                };
    
                if !current_samples.is_empty() {
                    // Try to decode samples
                    if let Ok(decoded) = decoder.decode(&current_samples) {
                        // Try to find a frame
                        if let Ok(Some(frame)) = Frame::deserialize(Bytes::from(decoded)) {
                            info!("🎵 Detected frame! Sequence: {} Length: {}", 
                                frame.sequence(),
                                frame.payload().len()
                            );
                        }
                    }
                }
                
                std::thread::sleep(Duration::from_millis(100));
            }
        });
    
        Ok(stream)
    }

    // Stop all active streams
    pub fn stop(&self, streams: &[Stream]) -> Result<(), Box<dyn Error>> {
        for stream in streams { stream.pause()?; }
        Ok(())
    }
}