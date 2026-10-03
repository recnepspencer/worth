mod error;
mod reading;
pub(crate) mod serde_canonical_aspect_value;
mod tags;
mod writing;

pub(crate) use error::AspectValueCanonicalCodecError;
pub(crate) use reading::decode_aspect_value;
pub(crate) use writing::{
    encode_aspect_value, encode_length_prefixed_aspect_value, encoded_aspect_value_len,
};

pub(crate) fn encode_string(bytes: &mut Vec<u8>, value: &str) {
    encode_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

pub(crate) fn encode_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
