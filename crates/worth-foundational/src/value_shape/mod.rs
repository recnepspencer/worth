//! Portable wire descriptions and bounded, Query-free value construction.
mod admission;
mod containers;
mod description;
mod primitive;
mod wire;

pub use admission::{ValueCodecDenial, ValueDecodeBounds};
pub use description::{ValueFieldShape, ValuePrimitive, ValueShape, ValueVariantShape};
pub use wire::{
    decode_canonical_value, encode_canonical_value, CanonicalValue, CanonicalValueSeed,
};
