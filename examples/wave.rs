#![allow(unused)]  // silence unused warnings while developing

use std::{error::Error, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait};
use dev_utils::{
    app_dt, debug, dlog::*, error, format::*, info, read_input, trace, warn
};
use wave::{audio::{capture::AudioCapture, playback::AudioPlayback, router::AudioDev}, encoding::FSKEncoder};


// import some::*; from parent dir

// Example usage in main
fn main() -> Result<(), Box<dyn Error>> {
    app_dt!(file!());
    set_max_level(Level::Trace);

    // Setup devices
    let input_device = select_device(true)?;
    let output_device = select_device(false)?;
    
    let capture = AudioCapture::new_with_device(input_device)?;
    let playback = AudioPlayback::new_with_device(output_device, Box::new(FSKEncoder::default()))?;

    let dev = AudioDev::new(capture, playback, Box::new(FSKEncoder::default()))?;

    


    Ok(())  // * Return Ok if no errors
}


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
