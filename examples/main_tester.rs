#![allow(unused)]  // silence unused warnings while developing


// use encoding::morse::{Morse, Signal};


use std::time::Duration;

use dev_utils::{
    app_dt, error, warn, info, debug, trace,
    dlog::*,
    format::*,
};
use wave::{
    audio::{AudioDevice}, 
    encoding::morse::{Morse, Signal}
};

// Example usage in main
fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    f();
}

fn f() {
    info!("Main tester");

    // * Any char must be separated by a CharGap to avoid ambiguity
    let (s, o, c) = (Signal::Dit, Signal::Dah, Signal::CharGap);
    let signals = vec![s, s, s, c, o, o, o, c, s, s, s];  // SOS

    let signals = Morse::text_to_signals("SOS");

    let some = Morse::signals_to_text(&signals);

    println!("{:?}", some == "SOS");

}
