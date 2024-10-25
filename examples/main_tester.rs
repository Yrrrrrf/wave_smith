#![allow(unused)]  // silence unused warnings while developing


use wave_smith::{
    some_fn,
    another_fn,
};
use dev_utils::{
    app_dt, error, warn, info, debug, trace,
    dlog::*,
};


fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    some_fn();
    another_fn();

}
