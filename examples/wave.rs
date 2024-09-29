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
    // set_max_level(Level::Trace);  // Only print the trace macros
    set_max_level(Level::Warn);  // trace, debug, info, warn
    // the Level::Error will print ALL the logs

    // io_test();
    // test_audio_connection("CACAHUATE");
    // test_audio_connection("ABCDEFGHIJKLMNOPQRSTUVWXYZ 0123456789");
    test_audio_connection("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
}

fn io_test() {
    // * Test the audio connection when the user presses Enter
    println!("Testing audio connection...");
    println!("Press Enter to start the test...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.pop();  // rm newline char (Enter key)
    test_audio_connection(&input).unwrap();
}


fn setup_devices() -> Result<(cpal::Device, cpal::Device, cpal::SupportedStreamConfig), AudioError> {
    let host = cpal::default_host();
    let output_device = host.default_output_device().ok_or(AudioError::DeviceNotFound)?;
    let input_device = host.default_input_device().ok_or(AudioError::DeviceNotFound)?;
    let supported_config = input_device.default_input_config().map_err(|_| AudioError::DeviceNotFound)?;

    trace!("Output device: {:?}", output_device.name());
    trace!("Input device: {:?}\n{:#?}", input_device.name(), supported_config);

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
    let morse_samples = MorseConverter::morse_to_samples(&morse_code, sample_rate);

    let mut sample_clock = Arc::new(Mutex::new(0));
    let sample_clock_clone = Arc::clone(&sample_clock);

    device.build_output_stream(
        config,
        move |data: &mut [f32], _: &_| {
            let mut clock = sample_clock_clone.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                // Determine the sample value based on the current clock position
                let sample = match clock.cmp(&morse_samples.len()) {
                    std::cmp::Ordering::Less => morse_samples[*clock],  // continue playing the morse code
                    _ => 0.0,  // if the clock is out of bounds, output silence
                };
                // Apply the sample value to all channels in the current frame
                frame.iter_mut().for_each(|sample_slice| *sample_slice = sample);
                *clock += 1;  // Increment the clock
            }
        },
        |err| eprintln!("Error in output stream: {}", err),
        None,
    ).map_err(AudioError::StreamError)
}

// todo: Valiadte that the audio was received correctly
// todo: For now it just checks if there is any audio signal
// todo: CREATE THE FRAME TO HANDLE THE AUDIO SIGNAL
// todo: Improve this to handle the EXACT duration of the message
fn analyze_received_audio(received: &[f32]) -> bool {
    match received {
        [] => {
            warn!("No audio data received. Check your audio connection.");
            false
        },
        samples => {
            info!("Audio data received successfully! {} samples captured.", samples.len());

            samples.iter().enumerate().find(|&(_, &sample)| sample.abs() > 0.01).map_or_else(
                || {warn!("No significant audio signal detected. Test failed."); false},
                |(pos, _)| {info!("Signal detected at sample {pos}");
                    // todo: HANDLE A WAY TO USE THIS TO CHECK THE BEGINNING OF THE SIGNAL FRAME
                    trace!("First 10 samples: {:?}", &samples[..10.min(samples.len())]);
                    true
            })
        }
    }
}


fn test_audio_connection(message: &str) -> Result<bool, AudioError> {
    let (output_device, input_device, supported_config) = setup_devices()?;

    let config = supported_config.config();

    let received_samples = Arc::new(Mutex::new(Vec::new()));
    let received_samples_clone = Arc::clone(&received_samples);

    // * INPUT STREAM (receive the message)
    let input_stream = create_input_stream(&input_device, &config, received_samples_clone)?;
    // * OUTPUT STEAM (send the message)
    let output_stream = create_output_stream(&output_device, &config, message)?;

    // ^ Start the input stream first to avoid losing the first samples
    input_stream.play().map_err(AudioError::PlayStreamError)?;
    // ^ Wait a bit before starting the output stream
    std::thread::sleep(Duration::from_millis(100));
    // ^ Start the output stream (send the message)
    output_stream.play().map_err(AudioError::PlayStreamError)?;
    // todo: Create the audio signal FRAME (trama) to be sent
    // todo: Improve this to handle the EXACT duration of the message
    std::thread::sleep(Duration::from_secs_f32(message.len() as f32 * 0.1 * 2.0));

    // * Pause the streams
    output_stream.pause().map_err(AudioError::PauseStreamError)?;
    input_stream.pause().map_err(AudioError::PauseStreamError)?;

    // * Analyze the received audio
    Ok(analyze_received_audio(received_samples.clone().lock().unwrap().as_slice()))
    // * same as above but with a clone (to avoid locking the mutex)
    // let received = received_samples.lock().unwrap().clone();
    // Ok(analyze_received_audio(&received))
}
