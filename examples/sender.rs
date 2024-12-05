use std::error::Error;
use std::{thread, time::Duration};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dev_utils::{format::*, read_input};
use wave::audio::capture::AudioCapture;
use wave::audio::playback::AudioPlayback;
use wave::encoding::FSKEncoder;

fn select_device(input: bool) -> Result<cpal::Device, Box<dyn Error>> {
    let host = cpal::default_host();
    let devices = match input {
        true => host.input_devices()?,
        false => host.output_devices()?
    };

    println!("\n{}", format!("Available {} Devices:", if input { "Input" } else { "Output" }).color(BLUE).style(Style::Bold));

    let devices: Vec<_> = devices.filter(|d| {match input {
        true => d.supported_input_configs().is_ok(),
        false => d.supported_output_configs().is_ok()
    }}).collect();

    for (idx, device) in devices.iter().enumerate() {
        println!("{}. {}", idx.to_string().color(GREEN), device.name().unwrap().color(WHITE));
    }

    loop {
        let input = read_input::<usize>(Some("Select device number: "))?;
        if input < devices.len() {return Ok(devices[input].clone());}
        println!("Invalid selection. Try again.");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup devices
    let input_device = select_device(true)?;
    let output_device = select_device(false)?;
    
    let capture = AudioCapture::new_with_device(input_device)?;
    let playback = AudioPlayback::new_with_device(output_device, Box::new(FSKEncoder::default()))?;

    println!("\nInput: {}", capture.device_info()?.name.color(GREEN));
    println!("Output: {}", playback.device_info()?.name.color(GREEN));
    // Start capture
    let input_stream = capture.start_listening()?;
    println!("\n{}", "Ready to transfer data! Type 'q' to quit".color(YELLOW));

    loop {
        let input = read_input::<String>(Some("Enter message to send: "))?;
        if input.trim() == "q" { break; }
        // Send the data
        let stream = playback.transmit(input.as_bytes())?;        
        // Wait a bit and get the captured samples
        thread::sleep(Duration::from_millis(500));
        let _received = capture.get_samples();
        // Stop the output stream
        stream.pause()?;
    }
    input_stream.pause()?;
    Ok(())
}
