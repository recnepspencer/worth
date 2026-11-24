pub mod borrowed_allocation;
pub mod error;
#[cfg(test)]
mod inline_bytes_tests;
pub mod reading;
pub mod serde_canonical_aspect_value;
pub mod tags;
pub mod writing;

pub use error::AspectValueCanonicalCodecError;
pub use reading::decode_aspect_value;
pub use writing::{
    encode_aspect_value, encode_length_prefixed_aspect_value, encoded_aspect_value_len,
    write_aspect_value,
};

pub fn encode_string(bytes: &mut Vec<u8>, value: &str) {
    encode_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

pub fn encode_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub use borrowed_allocation::{
    body_decoded_bytes as value_body_decoded_bytes,
    framed_decoded_bytes as value_frame_decoded_bytes,
};
