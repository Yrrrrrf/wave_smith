#![allow(unused)]
#![allow(dead_code)]

use std::collections::HashMap;
use dev_utils::{debug, error, info, trace, warn};
use lazy_static::lazy_static;

// Macro to create Morse code mapping
macro_rules! morse_map {
    ($($char:expr => $code:expr),* $(,)?) => {{
        let mut map = HashMap::new();  // Create a new HashMap
        $(map.insert($char, $code);)*  // Insert each character and its Morse code
        map  // Return the HashMap
        // todo: Do the same as above but in one line
        // (without declaring the map variable) (this won't benefit performance)
        // (but will make the code look cleaner) (same just for the sake of learning)
    }};
}

lazy_static! {
    static ref MORSE_CODE: HashMap<char, &'static str> = morse_map! {
        'A' => ".-", 
		'B' => "-...", 
		'C' => "-.-.", 
		'D' => "-..",
        'E' => ".", 
		'F' => "..-.", 
		'G' => "--.", 
		'H' => "....",
        'I' => "..", 
		'J' => ".---", 
		'K' => "-.-", 
		'L' => ".-..",
        'M' => "--", 
		'N' => "-.", 
		'O' => "---", 
		'P' => ".--.",
        'Q' => "--.-", 
		'R' => ".-.", 
		'S' => "...", 
		'T' => "-",
        'U' => "..-", 
		'V' => "...-", 
		'W' => ".--", 
		'X' => "-..-",
        'Y' => "-.--", 
		'Z' => "--..", 
		'0' => "-----", 
		'1' => ".----",
        '2' => "..---", 
		'3' => "...--", 
		'4' => "....-", 
		'5' => ".....",
        '6' => "-....", 
		'7' => "--...", 
		'8' => "---..", 
		'9' => "----.",
        ' ' => " ",
    };

    static ref REVERSE_MORSE_CODE: HashMap<&'static str, char> = {
        let mut map = HashMap::new();
        for (k, v) in MORSE_CODE.iter() {
            map.insert(*v, *k);
        }
        map
    };
}

pub struct MorseConverter;

impl MorseConverter {
    // * Convert text to Morse code (modulate text)
    /// Converts text to Morse code and returns both the original text and Morse code aligned
    pub fn str_morse_eq(text: &str) -> (String, String) {
        let morse = text.to_uppercase()
            .chars()
            .map(|c| {
                MORSE_CODE.get(&c)
                    .map(|&code| (c.to_string(), code.to_string()))
                    .unwrap_or((c.to_string(), " ".to_string()))
            })
            .collect::<Vec<(String, String)>>();

        let max_morse_len = morse.iter().map(|(_, code)| code.len()).max().unwrap_or(0);

        let aligned_text: String = morse.iter()
            .map(|(char, _)| format!("{:^width$}", char, width = max_morse_len + 1))
            .collect();

        let aligned_morse: String = morse.iter()
            .map(|(_, code)| format!("{:width$}", code, width = max_morse_len + 1))
            .collect();

        (aligned_text, aligned_morse)
    }

    /// Converts text to Morse code
    pub fn text_to_morse(text: &str) -> String {
        let (aligned_text, aligned_morse) = Self::str_morse_eq(text);
        debug!("src-str: {}", aligned_text);
        debug!("morse:   {}", aligned_morse);
        
        // Return the non-aligned Morse code for further processing
        text.to_uppercase()
            .chars()
            .filter_map(|c| MORSE_CODE.get(&c))
            .cloned()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    // todo: Test this function (demodulate morse code)
    pub fn morse_to_text(morse: &str) -> String {
        morse.split("  ").map(|word| {word.split_whitespace()
            .filter_map(|code| REVERSE_MORSE_CODE.get(code))
            .collect::<String>()
        }).collect::<Vec<String>>().join(" ")
    }

    // ^ Convert Morse code to audio samples
    // ^ This is a new function that we will implement in the next section
    pub fn morse_to_samples(morse: &str, sample_rate: f32) -> Vec<f32> {
        const DOT_DURATION: f32 = 0.1;
        const DASH_DURATION: f32 = DOT_DURATION * 3.0;
        const ELEMENT_GAP: f32 = DOT_DURATION;
        const LETTER_GAP: f32 = DOT_DURATION * 3.0;
        const WORD_GAP: f32 = DOT_DURATION * 7.0;
        const FREQUENCY: f32 = 440.0;

        let mut samples = Vec::new();
        let mut is_first_element = true;

        for word in morse.split("   ") {  // Three spaces between words
            if !is_first_element {
                samples.extend(Self::generate_silence(WORD_GAP, sample_rate));
            }
            
            for (i, letter) in word.split_whitespace().enumerate() {
                if i > 0 {
                    samples.extend(Self::generate_silence(LETTER_GAP, sample_rate));
                }
                
                for (j, symbol) in letter.chars().enumerate() {
                    if j > 0 {
                        samples.extend(Self::generate_silence(ELEMENT_GAP, sample_rate));
                    }
                    
                    let duration = match symbol {
                        '.' => DOT_DURATION,
                        '-' => DASH_DURATION,
                        _ => continue,
                    };
                    
                    samples.extend(Self::generate_tone(duration, FREQUENCY, sample_rate));
                }
            }
            
            is_first_element = false;
        }

        // println!("Samples: {:#?}", samples);

        // create a file with the samples
        let sample_as_text: &str = &samples.iter().map(|s| s.to_string()).collect::<Vec<String>>().join("\n");
        match dev_utils::file::create("./morse_samples.txt", sample_as_text) {
            Ok(_) => info!("File created successfully"),
            Err(e) => error!("Error creating file: {}", e),
        }

        samples
    }
    
    fn generate_tone(duration: f32, frequency: f32, sample_rate: f32) -> Vec<f32> {
        let num_samples = (duration * sample_rate) as usize;
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (t * frequency * 2.0 * std::f32::consts::PI).sin()
            })
            .collect()
    }

    fn generate_silence(duration: f32, sample_rate: f32) -> Vec<f32> {
        vec![0.0; (duration * sample_rate) as usize]
    }




    pub fn samples_to_morse(samples: &[f32], sample_rate: f32) -> String {
        const DOT_DURATION: f32 = 0.1;
        const THRESHOLD: f32 = 0.5;

        let mut morse = String::new();
        let mut current_symbol = String::new();
        let mut silence_count = 0;

        for chunk in samples.chunks((DOT_DURATION * sample_rate) as usize) {
            let avg_amplitude = chunk.iter().map(|&s| s.abs()).sum::<f32>() / chunk.len() as f32;
            
            if avg_amplitude > THRESHOLD {
                silence_count = 0;
                current_symbol.push('.');
            } else {
                silence_count += 1;
                if silence_count == 1 {
                    if current_symbol.len() > 1 {
                        morse.push('-');
                    } else if !current_symbol.is_empty() {
                        morse.push('.');
                    }
                    current_symbol.clear();
                } else if silence_count == 3 {
                    morse.push(' ');
                }
            }
        }

        morse
    }
}
