use crate::aspects::{
    AbsenceLaw, AspectEquivalenceBasis, AspectEvolutionPolicy, FieldRequirement, OpaqueAspectType,
    ReferenceAspectType,
};
use crate::canonicalization::{CanonicalDigestAspectShapeKind, CanonicalDigestMaskMode};
use crate::values::ScalarAspectType;

pub(super) fn digest_shape_name(shape: CanonicalDigestAspectShapeKind) -> &'static str {
    match shape {
        CanonicalDigestAspectShapeKind::Scalar => "scalar",
        CanonicalDigestAspectShapeKind::Struct => "struct",
        CanonicalDigestAspectShapeKind::Opaque => "opaque",
        CanonicalDigestAspectShapeKind::Reference => "reference",
    }
}

pub(super) fn digest_mask_mode_name(mode: CanonicalDigestMaskMode) -> &'static str {
    match mode {
        CanonicalDigestMaskMode::Projection => "projection",
        CanonicalDigestMaskMode::Mutation => "mutation",
        CanonicalDigestMaskMode::Diagnostic => "diagnostic",
    }
}

pub(super) fn scalar_aspect_type_name(value: ScalarAspectType) -> &'static str {
    match value {
        ScalarAspectType::Null => "null",
        ScalarAspectType::Bool => "bool",
        ScalarAspectType::Int8 => "int8",
        ScalarAspectType::Int16 => "int16",
        ScalarAspectType::Int32 => "int32",
        ScalarAspectType::Int64 => "int64",
        ScalarAspectType::UInt8 => "uint8",
        ScalarAspectType::UInt16 => "uint16",
        ScalarAspectType::UInt32 => "uint32",
        ScalarAspectType::UInt64 => "uint64",
        ScalarAspectType::Float32 => "float32",
        ScalarAspectType::Float64 => "float64",
        ScalarAspectType::Decimal => "decimal",
        ScalarAspectType::BigInt => "big_int",
        ScalarAspectType::Rational => "rational",
        ScalarAspectType::String => "string",
        ScalarAspectType::Uuid => "uuid",
        ScalarAspectType::Date => "date",
        ScalarAspectType::Time => "time",
        ScalarAspectType::Timestamp => "timestamp",
        ScalarAspectType::TimestampTz => "timestamp_tz",
        ScalarAspectType::EntityRef => "entity_ref",
        ScalarAspectType::ContentRef => "content_ref",
        ScalarAspectType::InlineBytes => "inline_bytes",
    }
}

pub(super) fn opaque_aspect_type_name(value: OpaqueAspectType) -> &'static str {
    match value {
        OpaqueAspectType::Token => "token",
    }
}

pub(super) fn reference_aspect_type_name(value: ReferenceAspectType) -> &'static str {
    match value {
        ReferenceAspectType::Entity => "entity",
    }
}

pub(super) fn field_requirement_name(value: FieldRequirement) -> &'static str {
    match value {
        FieldRequirement::Required => "required",
        FieldRequirement::Optional => "optional",
        FieldRequirement::Defaulted => "defaulted",
    }
}

pub(super) fn absence_law_name(value: AbsenceLaw) -> &'static str {
    match value {
        AbsenceLaw::Required => "required",
        AbsenceLaw::Optional => "optional",
        AbsenceLaw::Defaulted => "defaulted",
    }
}

pub(super) fn aspect_evolution_policy_name(value: AspectEvolutionPolicy) -> &'static str {
    match value {
        AspectEvolutionPolicy::Frozen => "frozen",
        AspectEvolutionPolicy::AdditiveFieldsAllowed => "additive_fields_allowed",
        AspectEvolutionPolicy::WideningAllowed => "widening_allowed",
        AspectEvolutionPolicy::ExplicitBreakRequired => "explicit_break_required",
    }
}

pub(super) fn aspect_equivalence_basis_name(value: AspectEquivalenceBasis) -> &'static str {
    match value {
        AspectEquivalenceBasis::ExactCanonicalValue => "exact_canonical_value",
        AspectEquivalenceBasis::DeclaredStructFields => "declared_struct_fields",
        AspectEquivalenceBasis::OpaqueIdentity => "opaque_identity",
        AspectEquivalenceBasis::ReferenceIdentity => "reference_identity",
        AspectEquivalenceBasis::ContentIdentity => "content_identity",
    }
}
