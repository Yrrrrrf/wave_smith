use dev_utils::{
    app_dt, error, warn, info, debug, trace,
    dlog::*,
    format::*,
};
use wave::{
    audio::{AudioDevice, capture::AudioCapture, wave::PulseDetector},
    config::AudioConfig, 
    encoding::morse::{Morse, Signal}
};

// Example usage in main
fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    // Initialize basic components
    let config = AudioConfig::default();
    let capture = AudioCapture::new().expect("Failed to create capture");
    let detector = PulseDetector::new(config);

    println!("Basic Audio Listener");
    println!("Listening for signals...");

    // Start listening
    if let Ok(stream) = capture.start_listening() {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // Get and analyze samples
            let samples = capture.get_samples();
            let pulses = detector.detect_pulses(&samples);
            
            if !pulses.is_empty() {
                println!("Signal detected! Pulses: {}", pulses.len());
            }
        }
    }
}