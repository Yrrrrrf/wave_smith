mod frame;
mod packet;
mod segment;

pub use frame::Frame;
pub use packet::Packet;
pub use segment::Segment;


pub struct Message {
    frames: Vec<Frame>,
}


impl Message {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }
}
