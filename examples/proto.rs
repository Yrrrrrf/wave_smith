#![allow(unused)]

use std::{thread, time::Duration};
use wave::{encoding::{Encoder, FSKEncoder}, proto::Frame};
use dev_utils::{app_dt, error, warn, info, debug, trace, dlog::*};


fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);
    
    info!("Starting Audio Protocol Tests");
    
    // Run basic FSK codec test from original
    // test_fsk_codec();
    // Run comprehensive device tests
    // test_audio_device();
    test_frame_operations();
}


fn test_frame_operations() {
    info!("Testing Frame operations");

    // Test frame creation
    let test_data = b"Test Frame Data";
    let frame = Frame::new(1, test_data.to_vec());
    
    debug!("Created frame with sequence {}", frame.sequence);
    
    // Test frame serialization
    let frame_bytes = frame.to_bytes();
    debug!("Frame serialized to {} bytes", frame_bytes.len());
    
    // Test frame deserialization
    match Frame::from_bytes(&frame_bytes) {
        Ok(decoded_frame) => {
            assert_eq!(decoded_frame.sequence, 1, "Sequence number mismatch");
            assert_eq!(&decoded_frame.payload, test_data, "Payload mismatch");
            info!("Frame serialization/deserialization test passed");
        }
        Err(e) => {
            error!("Frame deserialization failed: {:?}", e);
        }
    }
}

fn test_fsk_codec() {
    info!("Testing FSK codec");

    let fsk = FSKEncoder::new(48000, 1200.0, 2400.0, 480);

    let data = b"Hello, World!";
    
    match fsk.encode(data) {
        Ok(encoded) => {
            debug!("Data encoded: {} samples", encoded.len());
            match fsk.decode(&encoded) {
                Ok(decoded) => {
                    trace!("\tOriginal data: {:?}", data);
                    trace!("\tDecoded data:  {:?}", decoded);
                    // if decoded == data {
                    //     info!("FSK codec test passed");
                    // } else {
                    //     error!("FSK codec test failed: data mismatch");
                    // }
                }
                Err(e) => error!("Decoding error: {}", e),
            }
        }
        Err(e) => error!("Encoding error: {}", e),
    }
}

fn test_encoder() {
    use wave::encoding::{FSKEncoder, Encoder};
    
    info!("Main tester");

    let fsk = FSKEncoder::new(
        48000,    // 48kHz sample rate
        1200.0,   // 1200 Hz for bit 0
        2400.0,   // 2400 Hz for bit 1
        480,      // 480 samples per bit (100 bps)
    );

    // Encoding
    let data = b"Hello, World!";
    // let encoded = fsk.encode(data)?;
    let encoded = match fsk.encode(data) {
        Ok(encoded) => encoded,
        Err(e) => {error!("Error: {}", e); return;}
    };
    
    // Decoding
    // let decoded = fsk.decode(&encoded)?;
    let decoded = match fsk.decode(&encoded) {
        Ok(decoded) => decoded,
        Err(e) => {error!("Error: {}", e); return;}
    };

    // Compare original and decoded data
    println!("Original data: {:?}", data);
    println!("Decoded data: {:?}", decoded);
    assert_eq!(data.to_vec(), decoded, "Decoded data should match original data");
}
