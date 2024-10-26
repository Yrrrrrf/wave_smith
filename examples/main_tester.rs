#![allow(unused)]  // silence unused warnings while developing


// use encoding::morse::{Morse, Signal};


use std::time::Duration;

use dev_utils::{
    app_dt, error, warn, info, debug, trace,
    dlog::*,
    format::*,
};
use wave::{
    audio::{AudioConfig, AudioDevice}, 
    encoding::morse::{Morse, Signal}
};

// Example usage in main
fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    let config = AudioConfig::default();

    // match AudioDevice::new(Some(config)) {
    //     Ok(device) => {
    //         // Example: Basic sync test
    //         if std::env::args().any(|arg| arg == "--send") {
    //             info!("Running in sender mode");
    //             if let Ok(stream) = device.send_sync_signal() {
    //                 std::thread::sleep(Duration::from_millis(200));
    //                 drop(stream);
    //                 info!("{}", "Sync signal sent".color(GREEN).style(Style::Bold));
    //             }
    //         } else {
    //             info!("Running in receiver mode");
    //             if let Ok(stream) = device.start_listening() {
    //                 info!("{}", "Listening for sync signal...".color(GREEN).style(Style::Bold));
                    
    //                 loop {
    //                     std::thread::sleep(Duration::from_millis(100));
    //                     if device.detect_sync() {
    //                         info!("{}", "Sync detected!".color(GREEN).style(Style::Bold));
    //                         device.clear_samples();
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     Err(e) => error!("Failed to initialize audio device: {:?}", e),
    // }
}

fn f() {
    info!("Main tester");

    let s = Signal::Dit;
    let o = Signal::Dah;
    let c = Signal::CharGap;
    // * Any char must be separated by a CharGap to avoid ambiguity
    let signals = vec![s, s, s, c, o, o, o, c, s, s, s];  // SOS
    let some = Morse::signals_to_text(&signals);

    println!("{:?}", some == "SOS");

}
