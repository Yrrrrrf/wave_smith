use std::time::{Duration, Instant};
use dev_utils::{app_dt, error, warn, info, debug, trace, dlog::*, format::*};
use wave::{
    audio::{AudioDevice, capture::AudioCapture, wave::PulseDetector},
    config::AudioConfig,
    encoding::morse::{Morse, Signal}
};

fn interpolate_color(value: f32, min: f32, max: f32) -> Color {
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
    
    let colors = [
        (0.0, (0, 0, 255)),    // Blue
        (0.3, (0, 255, 0)),    // Green
        (0.6, (255, 255, 0)),  // Yellow
        (1.0, (255, 0, 0)),    // Red
    ];
    
    let mut color1 = colors[0];
    let mut color2 = colors[1];
    
    for window in colors.windows(2) {
        if t >= window[0].0 && t <= window[1].0 {
            color1 = window[0];
            color2 = window[1];
            break;
        }
    }
    
    let factor = (t - color1.0) / (color2.0 - color1.0);
    
    let r = (color1.1.0 as f32 * (1.0 - factor) + color2.1.0 as f32 * factor) as u8;
    let g = (color1.1.1 as f32 * (1.0 - factor) + color2.1.1 as f32 * factor) as u8;
    let b = (color1.1.2 as f32 * (1.0 - factor) + color2.1.2 as f32 * factor) as u8;
    
    Color::from((r, g, b))
}

fn format_time(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = duration.subsec_millis();
    
    format!("[{:02}:{:02}:{:02}.{:03}]", hours, minutes, seconds, millis).style(Style::Dim).style(Style::Italic)
}

fn create_gradient_meter(value: f32, width: usize, peak_pos: Option<usize>) -> String {
    let meter_width = (value * width as f32 * 2.0) as usize;
    let meter_width = meter_width.min(width);
    let mut meter = String::with_capacity(width * 3);

    // Create gradient bar with peak indicator
    for i in 0..width {
        if i < meter_width {
            let segment_value = i as f32 / width as f32;
            let color = interpolate_color(segment_value, 0.0, 1.0);
            meter.push_str(&"█".color(color));
        } else if Some(i) == peak_pos {
            meter.push_str(&"▌".color(WHITE).style(Style::Bold)); // Peak indicator
        } else {
            meter.push(' ');
        }
    }
    format!("│{}│", meter)
}

fn format_signal_value(value: f32) -> String {
    let color = interpolate_color(value, 0.0, 0.1);
    format!("{:>10.8}", value).color(color).style(Style::Bold).to_string()
}

fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    let start_time = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let signal_threshold = args.get(1)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.01);

    println!("\n{}", "╔═══════════════════════════════════════════════".color(CYAN));
    println!("{} {}", 
        "║".color(CYAN),
        "AUDIO RECEIVER".color(WHITE).style(Style::Bold)
    );
    println!("{}", "╠═══════════════════════════════════════════════".color(CYAN));
    println!("{} {} {}", 
        "║".color(CYAN),
        "Signal Threshold:".color(WHITE).style(Style::Bold),
        signal_threshold.to_string().color(GREEN)
    );
    println!("{}\n", "╚═══════════════════════════════════════════════".color(CYAN));

    let config = AudioConfig::default();
    let capture = match AudioCapture::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create capture: {}", e);
            return;
        }
    };
    let detector = PulseDetector::new(config);

    info!("{}", "Starting audio capture...".color(GREEN).style(Style::Bold));

    match capture.start_listening() {
        Ok(_stream) => {
            info!("{}", "Successfully started listening".color(GREEN));
            
            let mut peak_value = 0.0f32;
            let mut samples_count = 0usize;
            let mut last_peak_pos = None;
            let display_width = 50;
            
            // Create gradient scale
            println!("Signal Strength Scale:");
            let demo_gradient = (0..display_width)
                .map(|i| {
                    let t = i as f32 / display_width as f32;
                    "█".color(interpolate_color(t, 0.0, 1.0))
                })
                .collect::<String>();
            println!("│{demo_gradient}│\n");
            
            loop {
                std::thread::sleep(Duration::from_millis(50));
                let samples = capture.get_samples();
                samples_count += samples.len();

                if let Some(max_sample) = samples.iter().map(|s| s.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()) {
                    // Update peak tracking
                    if max_sample > peak_value {
                        peak_value = max_sample;
                        last_peak_pos = Some((peak_value * display_width as f32 * 2.0) as usize);
                    }

                    if max_sample > 0.001 {
                        print!("\x1B[2K"); // Clear line
                        print!("\x1B[1G"); // Move to start of line
                        
                        let elapsed = format_time(start_time.elapsed());
                        let meter = create_gradient_meter(max_sample, display_width, last_peak_pos);
                        let value = format_signal_value(max_sample);
                        let peak = format_signal_value(peak_value);
                        
                        println!("{} {} {} {} │ Peak: {}", 
                            elapsed,
                            "●".color(if samples_count % 2 == 0 { GREEN } else { YELLOW }),
                            meter,
                            value,
                            peak
                        );
                    }
                }

                // Reset peak periodically
                if samples_count > 48000 {
                    peak_value *= 0.8; // Gradual decay instead of instant reset
                    samples_count = 0;
                    if peak_value < 0.001 {
                        peak_value = 0.0;
                        last_peak_pos = None;
                    }
                }
            }
        },
        Err(e) => error!("Failed to start listening: {}", e),
    }
}
