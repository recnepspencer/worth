use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectMask, AspectShape, CanonicalFieldPath, FieldDeclaration, FieldKey,
    FieldRequirement, ProjectionMask, ScalarAspectType, StructAspectShape,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::decode_budget::RecordDecodeAttempt;
use super::sequence::{
    decode_sequence, require_canonical_sequence, require_canonical_sequence_by, write_sequence,
};

mod contract;
pub(super) use contract::{decode_aspect_contract, write_aspect_contract};

pub(super) fn write_absence(
    output: &mut dyn BinaryEncodingSink,
    absence: AbsenceLaw,
) -> Result<(), Denial> {
    output.u16(absence_tag(absence))
}
pub(super) fn decode_absence(input: &mut BinaryInput<'_>) -> Result<AbsenceLaw, Denial> {
    absence_from_tag(input.u16()?)
}

pub(super) fn write_projection_mask(
    output: &mut dyn BinaryEncodingSink,
    mask: &AspectMask<ProjectionMask>,
) -> Result<(), Denial> {
    write_sequence(output, mask.paths(), |output, path| {
        write_field_path(output, path)
    })
}

pub(super) fn decode_projection_mask(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<AspectMask<ProjectionMask>, Denial> {
    let paths = decode_sequence(input, budget, 8, |input, budget| {
        decode_field_path(input, budget)
    })?;
    require_canonical_sequence(&paths)?;
    Ok(if paths.is_empty() {
        AspectMask::whole_aspect()
    } else {
        AspectMask::new(paths)
    })
}

pub(super) fn write_field_path(
    output: &mut dyn BinaryEncodingSink,
    path: &CanonicalFieldPath,
) -> Result<(), Denial> {
    write_sequence(output, path.fields(), |output, field| {
        output.text(field.as_str())
    })
}

pub(super) fn decode_field_path(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<CanonicalFieldPath, Denial> {
    let fields = decode_sequence(input, budget, 5, |input, _| {
        FieldKey::new(input.text()?.to_owned()).ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
    })?;
    CanonicalFieldPath::new(fields).ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
}

pub(super) fn write_scalar_type(
    output: &mut dyn BinaryEncodingSink,
    value: ScalarAspectType,
) -> Result<(), Denial> {
    output.u16(match value {
        ScalarAspectType::Null => 1,
        ScalarAspectType::Bool => 2,
        ScalarAspectType::Int8 => 3,
        ScalarAspectType::Int16 => 4,
        ScalarAspectType::Int32 => 5,
        ScalarAspectType::Int64 => 6,
        ScalarAspectType::UInt8 => 7,
        ScalarAspectType::UInt16 => 8,
        ScalarAspectType::UInt32 => 9,
        ScalarAspectType::UInt64 => 10,
        ScalarAspectType::Float32 => 11,
        ScalarAspectType::Float64 => 12,
        ScalarAspectType::Decimal => 13,
        ScalarAspectType::BigInt => 14,
        ScalarAspectType::Rational => 15,
        ScalarAspectType::String => 16,
        ScalarAspectType::Bytes => 17,
        ScalarAspectType::Uuid => 18,
        ScalarAspectType::Date => 19,
        ScalarAspectType::Time => 20,
        ScalarAspectType::Timestamp => 21,
        ScalarAspectType::TimestampTz => 22,
        ScalarAspectType::EntityRef => 23,
        ScalarAspectType::ContentRef => 24,
    })
}

pub(super) fn decode_scalar_type(input: &mut BinaryInput<'_>) -> Result<ScalarAspectType, Denial> {
    Ok(match input.u16()? {
        1 => ScalarAspectType::Null,
        2 => ScalarAspectType::Bool,
        3 => ScalarAspectType::Int8,
        4 => ScalarAspectType::Int16,
        5 => ScalarAspectType::Int32,
        6 => ScalarAspectType::Int64,
        7 => ScalarAspectType::UInt8,
        8 => ScalarAspectType::UInt16,
        9 => ScalarAspectType::UInt32,
        10 => ScalarAspectType::UInt64,
        11 => ScalarAspectType::Float32,
        12 => ScalarAspectType::Float64,
        13 => ScalarAspectType::Decimal,
        14 => ScalarAspectType::BigInt,
        15 => ScalarAspectType::Rational,
        16 => ScalarAspectType::String,
        17 => ScalarAspectType::Bytes,
        18 => ScalarAspectType::Uuid,
        19 => ScalarAspectType::Date,
        20 => ScalarAspectType::Time,
        21 => ScalarAspectType::Timestamp,
        22 => ScalarAspectType::TimestampTz,
        23 => ScalarAspectType::EntityRef,
        24 => ScalarAspectType::ContentRef,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

fn write_field_declaration(
    output: &mut dyn BinaryEncodingSink,
    field: &FieldDeclaration,
) -> Result<(), Denial> {
    if field.codec().is_some() {
        return Err(Denial::new(Kind::UnsupportedCanonicalValueContract));
    }
    output.text(field.key().as_str())?;
    write_scalar_type_dyn(output, field.value_type())?;
    output.u16(requirement_tag(field.requirement()))?;
    output.u16(absence_tag(field.absence()))?;
    output.u16(evolution_tag(field.evolution()))
}

fn decode_field_declaration(input: &mut BinaryInput<'_>) -> Result<FieldDeclaration, Denial> {
    let key = FieldKey::new(input.text()?.to_owned())
        .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
    let value_type = decode_scalar_type(input)?;
    let requirement = requirement_from_tag(input.u16()?)?;
    let absence = absence_from_tag(input.u16()?)?;
    let evolution = evolution_from_tag(input.u16()?)?;
    FieldDeclaration::new(key, value_type, requirement, absence, evolution)
        .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
}

fn write_scalar_type_dyn(
    output: &mut dyn BinaryEncodingSink,
    value: ScalarAspectType,
) -> Result<(), Denial> {
    output.u16(scalar_type_tag(value))
}

const fn scalar_type_tag(value: ScalarAspectType) -> u16 {
    match value {
        ScalarAspectType::Null => 1,
        ScalarAspectType::Bool => 2,
        ScalarAspectType::Int8 => 3,
        ScalarAspectType::Int16 => 4,
        ScalarAspectType::Int32 => 5,
        ScalarAspectType::Int64 => 6,
        ScalarAspectType::UInt8 => 7,
        ScalarAspectType::UInt16 => 8,
        ScalarAspectType::UInt32 => 9,
        ScalarAspectType::UInt64 => 10,
        ScalarAspectType::Float32 => 11,
        ScalarAspectType::Float64 => 12,
        ScalarAspectType::Decimal => 13,
        ScalarAspectType::BigInt => 14,
        ScalarAspectType::Rational => 15,
        ScalarAspectType::String => 16,
        ScalarAspectType::Bytes => 17,
        ScalarAspectType::Uuid => 18,
        ScalarAspectType::Date => 19,
        ScalarAspectType::Time => 20,
        ScalarAspectType::Timestamp => 21,
        ScalarAspectType::TimestampTz => 22,
        ScalarAspectType::EntityRef => 23,
        ScalarAspectType::ContentRef => 24,
    }
}

const fn requirement_tag(value: FieldRequirement) -> u16 {
    match value {
        FieldRequirement::Required => 1,
        FieldRequirement::Optional => 2,
        FieldRequirement::Defaulted => 3,
    }
}

fn requirement_from_tag(tag: u16) -> Result<FieldRequirement, Denial> {
    match tag {
        1 => Ok(FieldRequirement::Required),
        2 => Ok(FieldRequirement::Optional),
        3 => Ok(FieldRequirement::Defaulted),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

const fn absence_tag(value: AbsenceLaw) -> u16 {
    match value {
        AbsenceLaw::Required => 1,
        AbsenceLaw::Optional => 2,
        AbsenceLaw::Defaulted => 3,
    }
}

fn absence_from_tag(tag: u16) -> Result<AbsenceLaw, Denial> {
    match tag {
        1 => Ok(AbsenceLaw::Required),
        2 => Ok(AbsenceLaw::Optional),
        3 => Ok(AbsenceLaw::Defaulted),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

const fn evolution_tag(value: AspectEvolutionPolicy) -> u16 {
    match value {
        AspectEvolutionPolicy::Frozen => 1,
        AspectEvolutionPolicy::AdditiveFieldsAllowed => 2,
        AspectEvolutionPolicy::WideningAllowed => 3,
        AspectEvolutionPolicy::ExplicitBreakRequired => 4,
    }
}

fn evolution_from_tag(tag: u16) -> Result<AspectEvolutionPolicy, Denial> {
    match tag {
        1 => Ok(AspectEvolutionPolicy::Frozen),
        2 => Ok(AspectEvolutionPolicy::AdditiveFieldsAllowed),
        3 => Ok(AspectEvolutionPolicy::WideningAllowed),
        4 => Ok(AspectEvolutionPolicy::ExplicitBreakRequired),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
