#![allow(unused)]  // silence unused warnings while developing


use rust_wave::{
    some_fn,
    another_fn,
};
use dev_utils::{
    print_app_data, error, warn, info, debug, trace,
    dlog::*,
};


fn main() {
    print_app_data(file!());
    set_max_level(Level::Trace);

    some_fn();
    another_fn();

}




