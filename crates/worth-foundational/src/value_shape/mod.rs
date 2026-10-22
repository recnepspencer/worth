//! Portable wire descriptions and bounded, Query-free value construction.
mod admission;
mod admitted;
mod containers;
mod decode_control;
mod definition;
mod delegates;
mod description;
mod document;
mod inspection;
mod materialization;
mod primitive;
mod wire;

pub use admission::{ValueCodecDenial, ValueDecodeBounds};
pub use admitted::decode_canonical_value_with_admission;
pub use decode_control::{value_decode_error, ValueDecodeAdmission, ValueDecodeControl};
pub use definition::{
    ValueFieldDefinition, ValueShapeBody, ValueShapeDefinition, ValueShapeEdge,
    ValueTaggedVariantDefinition, ValueVariantDefinition,
};
pub use delegates::{
    ArrayDelegate, BoxDelegate, DelegatedValueSeed, OptionalDelegate, PairDelegate,
    SequenceDelegate, UsizeDelegate, ValueDelegate, ValueSeedDelegate,
};
pub use description::{
    ValueFieldShape, ValuePrimitive, ValueShape, ValueShapeDepth, ValueTaggedVariantShape,
    ValueVariantShape,
};
pub use document::CanonicalDocumentCodec;
pub use wire::{
    decode_canonical_value, encode_canonical_value, CanonicalValue, CanonicalValueSeed,
};
