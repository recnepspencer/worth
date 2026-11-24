pub mod encoding_sink;
pub use encoding_sink::{CanonicalAspectOutput, CollectCanonicalAspect};
pub mod aspect_locator_canonical_codec;
pub mod aspect_value_canonical_codec;
pub mod canonical_byte_input;
pub mod field_patch_canonical_codec;
pub use canonical_byte_input::read_canonical_bytes;

pub use aspect_locator_canonical_codec::encode_aspect_field_locator;
pub use aspect_locator_canonical_codec::{decode_aspect_field_locator, locator_decoded_bytes};
#[cfg(test)]
pub use aspect_locator_canonical_codec::{
    decode_aspect_value_locator, decode_boundary_source_locator, encode_aspect_value_locator,
    encode_boundary_source_locator,
};
pub use aspect_locator_canonical_codec::{
    serde_canonical_aspect_field_locator, serde_canonical_aspect_field_locator_arc_slice,
    serde_canonical_aspect_value_locator,
};
pub use aspect_value_canonical_codec::{
    decode_aspect_value, encode_aspect_value, encode_length_prefixed_aspect_value, encode_string,
    encode_u32, encoded_aspect_value_len, serde_canonical_aspect_value, value_body_decoded_bytes,
    value_frame_decoded_bytes, AspectValueCanonicalCodecError,
};
pub use field_patch_canonical_codec::AspectFieldPatchCodecError;
pub use field_patch_canonical_codec::{
    decode_aspect_field_patch_canonical_bytes, encode_aspect_field_patch_canonical_bytes,
    field_patch_decoded_bytes,
};

#[cfg(test)]
mod aspect_locator_canonical_codec_tests;
#[cfg(test)]
mod aspect_value_canonical_codec_tests;

/// Concrete binary-owner storage; transient bytes are additional to retained.
#[derive(Clone, Copy, Default)]
pub struct CanonicalDecodeStorage {
    pub retained_bytes: u64,
    pub transient_bytes: u64,
}

pub use aspect_locator_canonical_codec::write_aspect_field_locator;
pub use aspect_value_canonical_codec::write_aspect_value;
