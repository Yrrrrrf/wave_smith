// * config.rs

#[derive(Debug, Clone, Copy)]
pub enum AudioConstant {
    SampleRate = 48000,
    BufferSize = 1024,
    Channels = 1,
    
    // Duration constants (in milliseconds for integers)
    BaseDuration = 100,    // 0.1 seconds
    SyncDuration = 200,    // 0.2 seconds
    
    // Frequency constants (Hz)
    BaseFrequency = 800,
    SyncFrequency = 440,   // A4 note
}

impl AudioConstant {
    // Helper methods to get values in different formats
    pub fn as_f32(&self) -> f32 {
        *self as i32 as f32
    }

    pub fn as_u32(&self) -> u32 {
        *self as i32 as u32
    }

    pub fn as_usize(&self) -> usize {
        *self as i32 as usize
    }

    pub fn as_duration_secs(&self) -> f32 {
        (*self as i32 as f32) / 1000.0  // Convert from milliseconds to seconds
    }
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    sample_rate: u32,
    channels: u16,
    buffer_size: usize,
    base_duration: f32,
    sync_duration: f32,
    base_frequency: f32,
    sync_frequency: f32,
    amplitude: f32,
}

impl AudioConfig {
    // Getters
    pub fn sample_rate(&self) -> u32 { self.sample_rate }
    pub fn channels(&self) -> u16 { self.channels }
    pub fn buffer_size(&self) -> usize { self.buffer_size }
    pub fn base_duration(&self) -> f32 { self.base_duration }
    pub fn sync_duration(&self) -> f32 { self.sync_duration }
    pub fn base_frequency(&self) -> f32 { self.base_frequency }
    pub fn sync_frequency(&self) -> f32 { self.sync_frequency }
    pub fn amplitude(&self) -> f32 { self.amplitude }

    // Builder methods
    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = rate;
        self
    }

    pub fn with_channels(mut self, channels: u16) -> Self {
        self.channels = channels;
        self
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn with_base_duration(mut self, duration: f32) -> Self {
        self.base_duration = duration;
        self
    }

    pub fn with_sync_duration(mut self, duration: f32) -> Self {
        self.sync_duration = duration;
        self
    }

    pub fn with_base_frequency(mut self, freq: f32) -> Self {
        self.base_frequency = freq;
        self
    }

    pub fn with_sync_frequency(mut self, freq: f32) -> Self {
        self.sync_frequency = freq;
        self
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: AudioConstant::SampleRate.as_u32(),
            channels: AudioConstant::Channels as u16,
            buffer_size: AudioConstant::BufferSize.as_usize(),
            base_duration: AudioConstant::BaseDuration.as_duration_secs(),
            sync_duration: AudioConstant::SyncDuration.as_duration_secs(),
            base_frequency: AudioConstant::BaseFrequency.as_f32(),
            sync_frequency: AudioConstant::SyncFrequency.as_f32(),
            amplitude: 0.5, // This could also be moved to AudioConstant if needed
        }
    }
}

// Example usage:
impl AudioConfig {
    pub fn example_usage() {
        // Using defaults
        let config = AudioConfig::default();

        // Using builder pattern
        let custom_config = AudioConfig::default()
            .with_sample_rate(44100)
            .with_amplitude(0.7)
            .with_base_frequency(1000.0);

        // Using constants directly
        let sample_rate = AudioConstant::SampleRate.as_u32();
        let duration_secs = AudioConstant::BaseDuration.as_duration_secs();
    }
}