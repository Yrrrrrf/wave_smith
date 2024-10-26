use std::error::Error;
use crate::audio::{AudioDevice};
use crate::audio::wave::{WaveGenerator, PulseDetector};

use super::Encoder;

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub enum Signal {
    Dit,
    Dah,
    CharGap,   // Gap between characters (3 units)
    WordGap,   // Gap between words (7 units)
}

macro_rules! morse_patterns {
    ($($char:expr => $pattern:expr),* $(,)?) => {
        impl Signal {
            fn from_char(c: char) -> Option<Self> {
                Some(match c {  // ^ Some is used to avoid Option<Signal>
                    '.' => Signal::Dit,
                    '-' => Signal::Dah,
                    ' ' => Signal::CharGap,
                    _ => return None,  // * ignore invalid characters
                })
            }
        }

        impl Morse {
            fn char_to_signal(c: char) -> Vec<Signal> {
                match c {
                    $($char => $pattern.chars().filter_map(Signal::from_char).collect(),)*
                    _ => vec![]
                }
            }

            fn signals_to_char(signals: &[Signal]) -> Option<char> {
                let pattern: String = signals.iter()
                    .map(|s| match s {
                        Signal::Dit => '.',  // Short signal
                        Signal::Dah => '-',  // Long signal
                        Signal::CharGap => ' ',  // Gap
                        Signal::WordGap => ' ',  // Gap
                    }).collect();
                match pattern.as_str() { $($pattern => Some($char),)* _ => None }
            }
        }
    };
}

pub struct Morse;

morse_patterns! {
    // &Letters
    'A' => ".-", 'B' => "-...", 'C' => "-.-.", 'D' => "-..",
    'E' => ".", 'F' => "..-.", 'G' => "--.", 'H' => "....",
    'I' => "..", 'J' => ".---", 'K' => "-.-", 'L' => ".-..", 'M' => "--", 'N' => "-.", 
    'O' => "---", 'P' => ".--.", 'Q' => "--.-", 'R' => ".-.", 'S' => "...", 'T' => "-",
    'U' => "..-", 'V' => "...-", 'W' => ".--", 'X' => "-..-", 'Y' => "-.--", 'Z' => "--..",
    // &Numbers
    '0' => "-----", '1' => ".----", '2' => "..---", '3' => "...--", '4' => "....-", 
    '5' => ".....", '6' => "-....", '7' => "--...", '8' => "---..", '9' => "----.",
    // &Special characters
    ' ' => " ",  // char gap 
}

impl Morse {
    /// Convert morse signals to text
    pub fn signals_to_text(signals: &[Signal]) -> String {
        let mut result = String::new();
        let mut current_char = Vec::new();
        let mut gap_count = 0;

        for &signal in signals {
            match signal {
                Signal::CharGap | Signal::WordGap => {
                    if !current_char.is_empty() {
                        if let Some(c) = Self::signals_to_char(&current_char) { result.push(c); }
                        current_char.clear();
                    }
                    if signal == Signal::WordGap && !result.ends_with(' ') && gap_count == 0 { 
                        result.push(' '); gap_count = 1; 
                    }
                },
                _ => { gap_count = 0; current_char.push(signal); }
            }
        }

        if !current_char.is_empty() {
            if let Some(c) = Self::signals_to_char(&current_char) { result.push(c); }
        }

        result.trim().to_string()
    }

    /// Convert text to morse signals
    pub fn text_to_signals(text: &str) -> Vec<Signal> {
        text.to_uppercase().chars().filter(|&c| c.is_alphanumeric() || c == ' ')
            .fold((Vec::new(), true), |(mut acc, first), c| {
                match c {
                    ' ' => { acc.push(Signal::WordGap); (acc, true) },
                    _ => {
                        if !first { acc.push(Signal::CharGap); }
                        acc.extend(Self::char_to_signal(c));
                        (acc, false)
                    }
                }
            }
        ).0  // select the signals (discard the boolean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_signals() {
        let s = Signal::Dit;
        let o = Signal::Dah;
        let c = Signal::CharGap;

        let signals = vec![s, s, s, c, o, o, o, c, s, s, s];  // S O S
        assert_eq!(Morse::signals_to_text(&signals), "SOS");
    }

    #[test]
    fn test_simple_words() {
        let input = "HELLO WORLD";
        let signals = Morse::text_to_signals(input);
        let output = Morse::signals_to_text(&signals);
        assert_eq!(output, input);
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(
            Morse::signals_to_text(&Morse::text_to_signals("H@LLO!")),
            "HLLO"
        );
        assert_eq!(Morse::signals_to_text(&Morse::text_to_signals("")), "");
        assert_eq!(Morse::signals_to_text(&Morse::text_to_signals("@#$%")), "");
    }

    #[test]
    fn test_space_handling() {
        assert_eq!(
            Morse::signals_to_text(&Morse::text_to_signals("A  B   C")),
            "A B C"
        );
        assert_eq!(
            Morse::signals_to_text(&Morse::text_to_signals("  SOS  ")),
            "SOS"
        );
    }

    #[test]
    fn test_char_gaps() {
        let signals = vec![
            Signal::Dit, Signal::Dit,  // I
            Signal::CharGap,
            Signal::Dit, Signal::Dah,  // A
            Signal::CharGap,
            Signal::Dah,  // T
        ];
        assert_eq!(Morse::signals_to_text(&signals), "IAT");
    }

    #[test]
    fn test_case_insensitivity() {
        let upper = "HELLO";
        let lower = "hello";
        let mixed = "HeLLo";
        
        let signals_upper = Morse::text_to_signals(upper);
        let signals_lower = Morse::text_to_signals(lower);
        let signals_mixed = Morse::text_to_signals(mixed);
        
        assert_eq!(Morse::signals_to_text(&signals_upper), "HELLO");
        assert_eq!(Morse::signals_to_text(&signals_lower), "HELLO");
        assert_eq!(Morse::signals_to_text(&signals_mixed), "HELLO");
    }

    #[test]
    fn test_alphanumeric() {
        let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let signals = Morse::text_to_signals(input);
        let output = Morse::signals_to_text(&signals);
        assert_eq!(output, input);
    }

    #[test]
    fn test_common_words() {
        let phrases = [
            "THE QUICK BROWN FOX",
            "JUMPS OVER THE LAZY DOG",
            "PACK MY BOX WITH FIVE DOZEN LIQUOR JUGS",
        ];

        for phrase in phrases {
            let signals = Morse::text_to_signals(phrase);
            let output = Morse::signals_to_text(&signals);
            assert_eq!(output, phrase);
        }
    }

    #[test]
    fn test_signal_patterns() {
        // Test a few specific Morse patterns
        let patterns = [
            ("SOS", "... --- ..."),
            ("OK", "--- -.-"),
            ("73", "--... ...--"),  // Common ham radio signals
        ];

        for (text, _pattern) in patterns {
            let signals = Morse::text_to_signals(text);
            let output = Morse::signals_to_text(&signals);
            assert_eq!(output, text);
        }
    }
}


// Encoder trait implementation
impl Encoder for Morse {
    fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
        let text = String::from_utf8_lossy(data);
        let signals = Self::text_to_signals(&text);
        let wave_gen = WaveGenerator::default();
        
        let mut audio_samples = Vec::new();
        
        for signal in signals {
            let samples = match signal {
                Signal::Dit => wave_gen.generate_tone(0.1),
                Signal::Dah => wave_gen.generate_tone(0.3),
                Signal::CharGap => wave_gen.generate_silence(0.3),
                Signal::WordGap => wave_gen.generate_silence(0.7),
            };
            audio_samples.extend(samples);
        }

        Ok(audio_samples)
    }

    fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>> {
        let detector = PulseDetector::default();
        let pulses = detector.detect_pulses(samples);
        
        // * SAME AS ABOVE USING MATCH
        let signals: Vec<Signal> = pulses.into_iter()
            .map(|pulse| match pulse {
                pulse if pulse < detector.min_pulse_samples => Signal::CharGap,
                pulse if pulse <= detector.max_pulse_samples => Signal::Dit,
                _ => Signal::Dah,
            })
            .collect();

        Ok(Self::signals_to_text(&signals).into_bytes())
    }
}

// // Example usage remains clean and simple
// fn main() -> Result<(), Box<dyn Error>> {
//     let morse = Morse;
//     let audio_device = AudioDevice::new(Box::new(morse))?;

//     // Send message
//     audio_device.send(b"SOS")?;
    
//     // Receive message
//     let received = audio_device.receive()?;
//     println!("Received: {}", String::from_utf8_lossy(&received));

//     Ok(())
// }