#![allow(unused)]

use std::{thread, time::Duration};
use wave::{encoding::{Encoder, FSKEncoder}, proto::Frame};
use dev_utils::{app_dt, error, warn, info, debug, trace, dlog::*};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    
    // List all available devices
    println!("\nAvailable Input Devices:");
    for device in host.input_devices()? {
        println!("- {} [{:?}]", device.name()?, device.default_input_config()?);
    }
    
    println!("\nAvailable Output Devices:");
    for device in host.output_devices()? {
        println!("- {} [{:?}]", device.name()?, device.default_output_config()?);
    }
    
    println!("\nSelect test mode:");
    println!("1. Try input device for both send/receive");
    println!("2. Try output device for both send/receive");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => test_input_device(&host)?,
        "2" => test_output_device(&host)?,
        _ => println!("Invalid selection")
    }
    
    Ok(())
}

// Attempt to use input device for both operations
fn test_input_device(host: &cpal::Host) -> Result<(), Box<dyn Error>> {
    println!("\nTesting input device...");
    if let Some(device) = host.default_input_device() {
        println!("Using device: {}", device.name()?);
        
        // Try to build both input and output streams on this device
        let config = device.default_input_config()?.config();
        
        println!("Attempting to create input stream...");
        if let Ok(stream) = device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| {
                println!("Received {} samples", data.len());
            },
            |err| eprintln!("Error: {}", err),
            None
        ) {
            println!("Successfully created input stream!");
            stream.play()?;
        }
        
        println!("Attempting to create output stream on input device...");
        match device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &_| {
                // Try to send a simple tone
                for sample in data.iter_mut() {
                    *sample = 0.5;
                }
            },
            |err| eprintln!("Error: {}", err),
            None
        ) {
            Ok(_) => println!("Successfully created output stream on input device!"),
            Err(e) => println!("Failed to create output stream: {}", e)
        }
    }
    Ok(())
}

// Attempt to use output device for both operations
fn test_output_device(host: &cpal::Host) -> Result<(), Box<dyn Error>> {
    println!("\nTesting output device...");
    if let Some(device) = host.default_output_device() {
        println!("Using device: {}", device.name()?);
        
        // Try to build both input and output streams on this device
        let config = device.default_output_config()?.config();
        
        println!("Attempting to create output stream...");
        if let Ok(stream) = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &_| {
                // Try to send a simple tone
                for sample in data.iter_mut() {
                    *sample = 0.5;
                }
            },
            |err| eprintln!("Error: {}", err),
            None
        ) {
            println!("Successfully created output stream!");
            stream.play()?;
        }
        
        println!("Attempting to create input stream on output device...");
        match device.build_input_stream(&config,
            move |data: &[f32], _: &_| {println!("Received {} samples", data.len());},
            |err| eprintln!("Error: {}", err),
            None
        ) {
            Ok(_) => println!("Successfully created input stream on output device!"),
            Err(e) => println!("Failed to create input stream: {}", e)
        }
    }
    Ok(())
}
