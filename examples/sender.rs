use std::{
    error::Error,
    thread, time::Duration
};
use cpal::traits::StreamTrait;
use dev_utils::{format::*, read_input};
use wave::{
    encoding::FSKEncoder,
    audio::{
        capture::AudioCapture,
        playback::AudioPlayback,
        select_device,
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup devices
    let input_device = select_device(true)?;
    let output_device = select_device(false)?;
    
    let capture = AudioCapture::new_with_device(input_device)?;
    let playback = AudioPlayback::new_with_device(output_device, Box::new(FSKEncoder::default()))?;

    // Start capture
    let input_stream = capture.start_listening()?;
    println!("\n{}", "Ready to transfer data! Type 'q' to quit".color(YELLOW).style(Style::Dim));

    loop {
        let input = read_input::<String>(Some(&"Send: ".style(Style::Bold)))?;
        if input.trim() == "q" { break; }
        let stream = playback.transmit(input.as_bytes())?;  // Send the data

        // Wait a bit and get the captured samples
        thread::sleep(Duration::from_millis(100));  // 100ms cooldown
        let _received = capture.get_samples();
        stream.pause()?;  // Stop the output stream
    }
    input_stream.pause()?;
    Ok(())
}
