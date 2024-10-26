use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AudioProtocolError {
    AudioDeviceError(String),
    EncodingError(String),
    ProtocolError(String),
}

impl fmt::Display for AudioProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AudioDeviceError(msg) => write!(f, "Audio device error: {}", msg),
            Self::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            Self::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl Error for AudioProtocolError {}
