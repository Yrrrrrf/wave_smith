#![allow(unused)]
#![allow(unused_imports)]
// todo: Create the dev_utils prelude...

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use cpal::SupportedStreamConfig;
use std::time::Duration;
use dev_utils::{
    print_app_data, error, warn, info, debug, trace,
    dlog::*,
};


use rust_wave::{
    some_fn,
    another_fn,
    morse::MorseConverter,
};


#[derive(Debug)]
enum AudioError {
    DeviceNotFound,
    StreamError(cpal::BuildStreamError),
    PlayStreamError(cpal::PlayStreamError),
    PauseStreamError(cpal::PauseStreamError),
}

fn main() {
    print_app_data(file!());
    set_max_level(Level::Warn);  // Set the max level of logging

    static_test("aaaaaaaa");
    // static_test("Hello, world!");
    // io_test();
}

fn static_test(message: &str) {
    match test_audio_connection(message).unwrap() {
        true => println!("\x1b[32mAudio connection test passed successfully.\x1b[0m"),
        false => println!("\x1b[31mAudio connection test failed.\x1b[0m"),
    }
}

fn io_test() {
    // * Test the audio connection when the user presses Enter
    println!("Testing audio connection...");
    println!("Press Enter to start the test...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    test_audio_connection(&input).unwrap();
}


fn setup_devices() -> Result<(cpal::Device, cpal::Device, cpal::SupportedStreamConfig), AudioError> {
    let host = cpal::default_host();
    let output_device = host.default_output_device().ok_or(AudioError::DeviceNotFound)?;
    let input_device = host.default_input_device().ok_or(AudioError::DeviceNotFound)?;
    let supported_config = input_device.default_input_config().map_err(|_| AudioError::DeviceNotFound)?;

    info!("Output device: {:?}", output_device.name());
    info!("Input device: {:?}\n{:#?}", input_device.name(), supported_config);

    Ok((output_device, input_device, supported_config))
}

fn create_input_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    received_samples: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream, AudioError> {
    device.build_input_stream(
        config,
        move |data: &[f32], _: &_| {
            received_samples.lock().unwrap().extend_from_slice(data);
        },
        |err| eprintln!("Error in input stream: {}", err),
        None,
    ).map_err(AudioError::StreamError)
}

fn create_output_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    message: &str,
) -> Result<cpal::Stream, AudioError> {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    
    // Convert message to morse code
    let morse_code = MorseConverter::text_to_morse(message);
    println!("Source: {}\n", &message);
    println!("Morse:  {}\n", &morse_code);
    let morse_samples = MorseConverter::morse_to_samples(&morse_code, sample_rate);

    // println!("Morse samples: {:#?}", morse_samples);

    let mut sample_clock = Arc::new(Mutex::new(0));
    let sample_clock_clone = Arc::clone(&sample_clock);

    device.build_output_stream(
        config,
        move |data: &mut [f32], _: &_| {
            let mut clock = sample_clock_clone.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = if *clock < morse_samples.len() {
                    morse_samples[*clock]
                } else {
                    0.0
                };
                for sample_slice in frame.iter_mut() {
                    *sample_slice = sample;
                }
                *clock += 1;
            }
        },
        |err| eprintln!("Error in output stream: {}", err),
        None,
    ).map_err(AudioError::StreamError)
}

fn analyze_received_audio(received: &[f32]) -> bool {
    if received.is_empty() {
        println!("No audio data received. Check your audio connection.");
        false
    } else {
        println!("Audio data received successfully!");
        println!("Received {} samples", received.len());        
        match received.iter().position(|&sample| sample.abs() > 0.01) {
            Some(pos) => {
                println!("Signal detected at sample {}", pos);
                true
            },
            None => {
                println!("No significant audio signal detected. Test failed.");
                false
            }
        }
    }
}

fn test_audio_connection(message: &str) -> Result<bool, AudioError> {
    let (output_device, input_device, supported_config) = setup_devices()?;

    let config = supported_config.config();

    let received_samples = Arc::new(Mutex::new(Vec::new()));
    let received_samples_clone = Arc::clone(&received_samples);

    let input_stream = create_input_stream(&input_device, &config, received_samples_clone)?;
    let output_stream = create_output_stream(&output_device, &config, message)?;

    input_stream.play().map_err(AudioError::PlayStreamError)?;
    std::thread::sleep(Duration::from_millis(100)); // Give some time for the input stream to start
    output_stream.play().map_err(AudioError::PlayStreamError)?;

    // Wait for the message to be fully played
    let duration = Duration::from_secs_f32(message.len() as f32 * 0.1 * 2.0); // Rough estimate
    std::thread::sleep(duration);

    output_stream.pause().map_err(AudioError::PauseStreamError)?;
    input_stream.pause().map_err(AudioError::PauseStreamError)?;

    let received = received_samples.lock().unwrap().clone();
    Ok(analyze_received_audio(&received))
}
