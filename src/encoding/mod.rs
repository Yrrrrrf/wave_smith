
// * std library imports
use std::error::Error;

// * module imports
pub mod morse;
use morse::*;


// Common configuration for all encoders
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    pub sample_rate: u32,
    pub threshold: f32,
    pub min_pulse_samples: usize,
    pub max_pulse_samples: usize,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            threshold: 0.1,
            min_pulse_samples: (0.08 * 48000.0) as usize,  // 80ms
            max_pulse_samples: (0.12 * 48000.0) as usize,  // 120ms
        }
    }
}


pub trait Encoder {
    // Core encoding/decoding methods    // * Encode: bits -> signal
    fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>>;
    // * Decode: signal -> bits
    fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>>;

    // * In Digital Logic, the Encoder & Decoder are some circuit that
    // * converts the input data into a format that is suitable for
    // * transmission over a communication channel.
    // & src -> [Encoder] -> dst (transmission) -> [Decoder] -> src
    // ^ 'dst' is information that is transmitted over a communication channel
    // ^ 'src' is the data that is to be transmitted
}
