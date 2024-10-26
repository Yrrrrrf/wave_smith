// * wave.rs
use std::f32::consts::PI;
use crate::config::{AudioConfig, AudioConstant};

pub struct PulseDetector {
    config: AudioConfig,
    pub threshold: f32,
    pub min_pulse_samples: usize,
    pub max_pulse_samples: usize,
}

impl PulseDetector {
    pub fn new(config: AudioConfig) -> Self {
        let base_samples = config.sample_rate() as f32 * config.base_duration();
        Self {
            threshold: 0.1,
            min_pulse_samples: (base_samples * 0.8) as usize,
            max_pulse_samples: (base_samples * 1.2) as usize,
            config,
        }
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

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
    config: AudioConfig,
}

impl WaveGenerator {
    pub fn new(config: AudioConfig) -> Self {
        Self { config }
    }

    pub fn generate_tone(&self, duration: f32) -> Vec<f32> {
        self.generate_tone_with_frequency(duration, self.config.base_frequency())
    }

    pub fn generate_sync_tone(&self) -> Vec<f32> {
        self.generate_tone_with_frequency(
            self.config.sync_duration(),
            self.config.sync_frequency(),
        )
    }

    pub fn generate_tone_with_frequency(&self, duration: f32, frequency: f32) -> Vec<f32> {
        let num_samples = (duration * self.config.sample_rate() as f32) as usize;
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / self.config.sample_rate() as f32;
                (t * frequency * 2.0 * PI).sin() * self.config.amplitude()
            })
            .collect()
    }

    pub fn generate_silence(&self, duration: f32) -> Vec<f32> {
        vec![0.0; (duration * self.config.sample_rate() as f32) as usize]
    }
}
