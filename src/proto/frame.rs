// Constants for protocol
pub const SYNC_PATTERN: [u8; 4] = [0xAA, 0xAA, 0xAA, 0xAA];
pub const END_MARKER: [u8; 2] = [0xFF, 0xFF];
pub const VERSION: u8 = 0x01;

#[derive(Debug)]
pub enum AudioDeviceError {
    CaptureError(String),
    PlaybackError(String),
    SyncError(String),
    EncodingError(String),
    // Add more error types as needed
}

pub struct Frame {
    version: u8,
    pub sequence: u8,
    pub payload: Vec<u8>,
    pub checksum: u16,
}

impl Frame {
    pub fn new(sequence: u8, payload: Vec<u8>) -> Self {
        let mut frame = Self {
            version: VERSION,
            sequence,
            payload,
            checksum: 0,
        };
        frame.checksum = frame.calculate_checksum();
        frame
    }

    fn calculate_checksum(&self) -> u16 {
        // Simple CRC16 implementation
        let mut crc: u16 = 0xFFFF;
        crc = self.update_crc(crc, self.version);
        crc = self.update_crc(crc, self.sequence);
        for byte in &self.payload {
            crc = self.update_crc(crc, *byte);
        }
        crc
    }

    fn update_crc(&self, mut crc: u16, byte: u8) -> u16 {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
        crc
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&SYNC_PATTERN);
        bytes.push(self.version);
        bytes.push(self.sequence);
        bytes.extend_from_slice(&(self.payload.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        bytes.extend_from_slice(&END_MARKER);
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, AudioDeviceError> {
        if data.len() < 9 {
            return Err(AudioDeviceError::EncodingError("Frame too short".into()));
        }

        // Verify sync pattern
        if &data[0..4] != SYNC_PATTERN {
            return Err(AudioDeviceError::SyncError("Invalid sync pattern".into()));
        }

        let version = data[4];
        let sequence = data[5];
        let payload_len = u16::from_be_bytes([data[6], data[7]]) as usize;
        let payload = data[8..8+payload_len].to_vec();
        
        let frame = Self {
            version,
            sequence,
            payload,
            checksum: 0,
        };

        let calculated_checksum = frame.calculate_checksum();
        let received_checksum = u16::from_be_bytes([
            data[8+payload_len], 
            data[8+payload_len+1]
        ]);

        if calculated_checksum != received_checksum {
            return Err(AudioDeviceError::EncodingError("Checksum mismatch".into()));
        }

        Ok(frame)
    }
}
