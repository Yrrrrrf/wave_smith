// * Some of the most useful dis-allows (to silence most of the clippy warnings)
#![allow(unused)]

use std::error::Error;

pub mod audio;
pub mod proto;
pub mod encoding;
pub mod error;

pub trait AudioTransport {
    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>>;
    fn receive(&mut self) -> Result<Vec<u8>, Box<dyn Error>>;
}


#[cfg(test)]
mod tests {

    #[test]
    fn some_test() {assert_eq!((2_i32.pow(3))-4, 4);}

    #[test]
    fn some_test_2() {assert_eq!((2_i32.pow(3))-4, 4);}
}

