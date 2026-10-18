//! Portable wire descriptions and bounded, Query-free value construction.
mod admission;
mod admitted;
mod containers;
mod definition;
mod delegates;
mod description;
mod inspection;
mod materialization;
mod primitive;
mod wire;

pub use admission::{ValueCodecDenial, ValueDecodeBounds};
pub use admitted::decode_canonical_value_with_admission;
pub use definition::{
    ValueFieldDefinition, ValueShapeBody, ValueShapeDefinition, ValueShapeEdge,
    ValueTaggedVariantDefinition, ValueVariantDefinition,
};
pub use delegates::{
    BoxDelegate, DelegatedValueSeed, OptionalDelegate, PairDelegate, SequenceDelegate,
    ValueDelegate, ValueSeedDelegate,
};
pub use description::{
    ValueFieldShape, ValuePrimitive, ValueShape, ValueShapeDepth, ValueTaggedVariantShape,
    ValueVariantShape,
};
pub use wire::{
    decode_canonical_value, encode_canonical_value, CanonicalValue, CanonicalValueSeed,
    ValueDecodeAdmission,
};
