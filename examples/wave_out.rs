use std::time::Duration;
use dev_utils::{app_dt, error, warn, info, debug, trace, dlog::*, format::*};
use wave::{
    audio::{AudioDevice, playback::AudioPlayback, wave::WaveGenerator},
    config::AudioConfig,
    encoding::morse::{Morse, Signal}
};

// Helper function to generate samples for a Morse signal
fn signal_to_samples(signal: &Signal, wave_gen: &WaveGenerator) -> Vec<f32> {
    match signal {
        Signal::Dit => wave_gen.generate_tone(0.1),  // Short beep
        Signal::Dah => wave_gen.generate_tone(0.3),  // Long beep
        Signal::CharGap => wave_gen.generate_silence(0.3),  // Gap between characters
        Signal::WordGap => wave_gen.generate_silence(0.7),  // Gap between words
    }
}

fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    println!("\n{}", "╔═══════════════════════════════════════════════".color(CYAN));
    println!("{} {}", "║".color(CYAN), "MORSE CODE SENDER".color(WHITE).style(Style::Bold));
    println!("{}\n", "╚═══════════════════════════════════════════════".color(CYAN));

    // Initialize components
    let config = AudioConfig::default();
    let playback = match AudioPlayback::new() {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to initialize audio playback: {}", e);
            return;
        }
    };
    let wave_gen = WaveGenerator::new(config);

    loop {
        // Get user input
        info!("\n{}", "Enter text to send (or 'quit' to exit):".color(GREEN));
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let message = input.trim();
        if message.eq_ignore_ascii_case("quit") {
            break;
        }

        // Convert to Morse signals
        let signals = Morse::text_to_signals(message);
        
        // info!("Input  [{:>03}]:  {}", format!("{}", message.len().to_string().style(Style::Dim)), message.style(Style::Italic));
        // info!("Output [{:>03}]: {}", format!("{}", signals.len().to_string().style(Style::Dim)), format!("{}", 
        //     signals.iter().map(|s| s.to_string()).collect::<String>()
        //     ).style(Style::Dim));
        info!("I[{}]:  {}", format!("{:>03}c", message.len()).to_string().style(Style::Dim), message.style(Style::Italic));
        info!("O[{}]: {}", format!("{:>03}c", signals.len()).to_string().style(Style::Dim), format!("{}", 
            signals.iter().map(|s| s.to_string()).collect::<String>()
            ).style(Style::Dim));

        // Generate and concatenate all samples
        let mut all_samples = Vec::new();
        
        for signal in signals.clone() {
            let samples = signal_to_samples(&signal, &wave_gen);
            // debug!("Signal: {:?}, {} samples", signal, samples.len());

            all_samples.extend(samples);
            
            // Add a small gap between signals
            all_samples.extend(wave_gen.generate_silence(0.05));
        }

        // Play the complete message
        info!("{}", "Transmitting morse code...".color(YELLOW));
        
        match playback.play_samples(all_samples) {
            Ok(stream) => {
                // Calculate total duration based on signal count
                let total_duration = signals.len() as f32 * 0.5; // Approximate duration
                
                // Show a nice progress message
                info!("Sending message... {} seconds", 
                    total_duration.to_string().color(CYAN)
                );
                
                std::thread::sleep(Duration::from_secs_f32(total_duration));
                drop(stream);
                
                info!("{}", "✓ Message sent successfully!".color(GREEN).style(Style::Bold));
            },
            Err(e) => error!("Failed to play audio: {}", e),
        }
    }

    info!("{}", "Sender terminated.".color(YELLOW));
}