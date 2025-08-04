use serde::Serialize;

pub struct WindowPacket(pub Vec<u8>);

#[derive(Debug, Serialize)]
pub struct MessagePack {
    pub pcm: Vec<u8>,
    pub bpm: f64,
}
