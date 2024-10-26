use std::error::Error;

pub struct Frame {
    version: u8,
    sequence: u8,
    flags: u8,
    payload: Vec<u8>,
}

impl Frame {
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            version: 1,
            sequence: 0,
            flags: 0,
            payload,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Implement serialization
        Vec::new()
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        // TODO: Implement deserialization
        Ok(Self::new(Vec::new()))
    }
}
