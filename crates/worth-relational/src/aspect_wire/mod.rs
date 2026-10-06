mod aspect_locator_canonical_codec;
mod aspect_value_canonical_codec;
mod field_patch_canonical_codec;

pub(crate) use aspect_locator_canonical_codec::decode_aspect_field_locator;
pub(crate) use aspect_locator_canonical_codec::encode_aspect_field_locator;
#[cfg(test)]
pub(crate) use aspect_locator_canonical_codec::{
    decode_aspect_value_locator, decode_boundary_source_locator, encode_aspect_value_locator,
    encode_boundary_source_locator,
};
pub(crate) use aspect_locator_canonical_codec::{
    serde_canonical_aspect_field_locator, serde_canonical_aspect_field_locator_arc_slice,
    serde_canonical_aspect_value_locator, serde_canonical_boundary_source_locator,
};
pub(crate) use aspect_value_canonical_codec::{
    decode_aspect_value, encode_aspect_value, encode_length_prefixed_aspect_value, encode_string,
    encode_u32, encoded_aspect_value_len, serde_canonical_aspect_value,
    AspectValueCanonicalCodecError,
};
pub use field_patch_canonical_codec::AspectFieldPatchCodecError;
pub(crate) use field_patch_canonical_codec::{
    decode_aspect_field_patch_canonical_bytes, encode_aspect_field_patch_canonical_bytes,
};

#[cfg(test)]
mod aspect_locator_canonical_codec_tests;
#[cfg(test)]
mod aspect_value_canonical_codec_tests;
