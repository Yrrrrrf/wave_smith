
use std::f32::consts::PI;

const SAMPLE_RATE: f32 = 48000.0;
const BASE_DURATION: f32 = 0.1;  // seconds
const FREQUENCY: f32 = 800.0;    // Hz
const AMPLITUDE: f32 = 0.5;

pub struct PulseDetector {
    pub threshold: f32,
    pub min_pulse_samples: usize,
    pub max_pulse_samples: usize,
}

impl Default for PulseDetector {
    fn default() -> Self {
        Self {
            threshold: 0.1,
            min_pulse_samples: (SAMPLE_RATE * BASE_DURATION * 0.8) as usize,
            max_pulse_samples: (SAMPLE_RATE * BASE_DURATION * 1.2) as usize,
        }
    }
}

impl PulseDetector {
    pub fn detect_pulses(&self, samples: &[f32]) -> Vec<usize> {
        let mut pulses = Vec::new();
        let mut current_pulse = 0;
        let mut in_pulse = false;

        for &sample in samples {
            if sample.abs() > self.threshold {
                current_pulse += 1;
                in_pulse = true;
            } else if in_pulse {
                pulses.push(current_pulse);
                current_pulse = 0;
                in_pulse = false;
            }
        }

        if in_pulse && current_pulse > 0 {
            pulses.push(current_pulse);
        }

        pulses
    }
}

pub struct WaveGenerator {
    sample_rate: f32,
    frequency: f32,
    amplitude: f32,
}

impl Default for WaveGenerator {
    fn default() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            frequency: FREQUENCY,
            amplitude: AMPLITUDE,
        }
    }
}

impl WaveGenerator {
    pub fn generate_tone(&self, duration: f32) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate) as usize;
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / self.sample_rate;
                (t * self.frequency * 2.0 * PI).sin() * self.amplitude
            })
            .collect()
    }

    pub fn generate_silence(&self, duration: f32) -> Vec<f32> {
        vec![0.0; (duration * self.sample_rate) as usize]
    }
}