//! Portable wire descriptions and bounded, Query-free value construction.
mod admission;
mod admitted;
mod containers;
mod description;
mod primitive;
mod wire;

pub use admission::{ValueCodecDenial, ValueDecodeBounds};
pub use admitted::decode_canonical_value_with_admission;
pub use description::{ValueFieldShape, ValuePrimitive, ValueShape, ValueVariantShape};
pub use wire::{
    decode_canonical_value, encode_canonical_value, CanonicalValue, CanonicalValueSeed,
    ValueDecodeAdmission,
};
