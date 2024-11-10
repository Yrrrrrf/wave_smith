use std::error::Error;
use std::time::Duration;
use std::thread;

use cpal::traits::StreamTrait;

use wave::*;
use wave::encoding::Encoder;
use wave::encoding::FSKEncoder;
use wave::audio::{
    capture::AudioCapture,
    playback::AudioPlayback,
    router::AudioRouter,
    signal::SignalMonitor,
};

const TEST_DATA: &[u8] = &[
    0xAA, 0xBB, 0xCC, 0xDD,  // Test pattern
    b'H', b'e', b'l', b'l', b'o',  // ASCII text
    0x01, 0x02, 0x03, 0x04   // Binary sequence
];


fn main() -> Result<(), Box<dyn Error>> {
    // Run tests
    test_audio_loopback()?;
    // test_signal_strength()?;
    // test_fsk_configs()?;

    Ok(())
}


fn test_audio_loopback() -> Result<(), Box<dyn Error>> {
    // Create FSK encoder with default settings
    let encoder = Box::new(FSKEncoder::default());
    let encoder_clone = Box::new(FSKEncoder::default());

    // Initialize audio devices
    let capture = AudioCapture::new()?;
    let playback = AudioPlayback::new(encoder_clone)?;

    // Create router
    let router = AudioRouter::new(capture, playback, encoder)?;

    // Create signal monitor
    let mut monitor = SignalMonitor::new(50, Box::new(FSKEncoder::default()));
    monitor.print_header();

    // Start the test sequence
    println!("Starting audio loopback test...");
    println!("Test data: {:?}", TEST_DATA);

    // Send data through playback
    let play_stream = router.send(TEST_DATA)?;
    
    // Give some time for the signal to stabilize
    thread::sleep(Duration::from_millis(100));

    // Start listening
    let (capture_stream, received_data) = router.listen()?;

    // Monitor the signal for a short duration
    for _ in 0..10 {
        if let Some(decoded) = monitor.process_samples(&router.encoder.encode(TEST_DATA)?) {
            // println!("Decoded data: {:?}", decoded);
            assert_eq!(decoded, TEST_DATA.to_vec(), "Decoded data doesn't match sent data");
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Verify the received data
    assert_eq!(received_data, TEST_DATA.to_vec(), "Received data doesn't match sent data");

    // Clean up
    play_stream.pause()?;
    capture_stream.pause()?;

    println!("Audio loopback test completed successfully!");
    Ok(())
}

fn test_signal_strength() -> Result<(), Box<dyn Error>> {
    let encoder = Box::new(FSKEncoder::default());
    let playback = AudioPlayback::new(encoder.clone())?;
    let mut monitor = SignalMonitor::new(50, Box::new(FSKEncoder::default()));

    // Print monitor header
    monitor.print_header();

    // Test different signal strengths
    let test_signals = [
        (&[0x00], "0000 0000"),  //   0
        (&[0x01], "0000 0001"),  //   1
        (&[0x03], "0000 0011"),  //   3
        (&[0x07], "0000 0111"),  //   7
        (&[0x0F], "0000 1111"),  //  15
        (&[0x1F], "0001 1111"),  //  31
        (&[0x3F], "0011 1111"),  //  63
        (&[0x7F], "0111 1111"),  // 127
        (&[0xFF], "1111 1111"),  // 255
    ];
    for (signal, description) in test_signals.iter() {
        println!("\nTesting {} ...", description);
        
        // let s = cast signal to &[u8]
        let s = signal.as_ref();
        // Send signal
        let stream = playback.transmit(s)?;
        // Monitor signal strength
        let samples = encoder.encode(s)?;
        monitor.process_samples(&samples);
        
        thread::sleep(Duration::from_millis(250));
        stream.pause()?;
    }

    Ok(())
}

fn test_fsk_configs() -> Result<(), Box<dyn Error>> {
    let configs = [
        (1200.0, 2400.0, "Standard FSK"),
        (800.0, 1600.0, "Low Frequency FSK"),
        (2400.0, 4800.0, "High Frequency FSK"),
    ];

    for (freq_0, freq_1, description) in configs.iter() {
        println!("\nTesting {} (f0={}, f1={})...", description, freq_0, freq_1);

        let encoder = Box::new(FSKEncoder::new(48000, *freq_0, *freq_1, 480));
        let playback = AudioPlayback::new(encoder)?;
        
        // Send test signal
        let stream = playback.transmit(TEST_DATA)?;
        thread::sleep(Duration::from_millis(500));
        stream.pause()?;
    }

    Ok(())
}
