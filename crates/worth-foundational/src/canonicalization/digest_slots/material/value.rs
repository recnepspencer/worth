use super::token_writer::{
    append_bytes, append_i128, append_i32, append_i64, append_token, append_token_parts,
    append_u128, append_u64,
};
use super::writer::{CanonicalMaterialResult, CanonicalMaterialWriter};
use crate::aspects::StructAspectValue;
use crate::canonicalization::{CanonicalBasisValue, CanonicalFloatWidth, CanonicalIntegerWidth};
use crate::values::InternedString;

pub(crate) fn value_material(value: &CanonicalBasisValue) -> String {
    let mut material = CanonicalMaterialWriter::bounded(usize::MAX);
    append_value_material(&mut material, value)
        .expect("an unbounded canonical material writer accepts a typed value");
    String::from_utf8(material.finish().into_bytes())
        .expect("canonical material grammar writes UTF-8")
}

pub(crate) fn struct_value_material(value: &StructAspectValue) -> String {
    let mut material = CanonicalMaterialWriter::bounded(usize::MAX);
    append_struct_value_material(&mut material, value)
        .expect("an unbounded canonical material writer accepts a typed struct");
    String::from_utf8(material.finish().into_bytes())
        .expect("canonical material grammar writes UTF-8")
}

pub(crate) fn append_value_material(
    material: &mut CanonicalMaterialWriter,
    value: &CanonicalBasisValue,
) -> CanonicalMaterialResult {
    match value {
        CanonicalBasisValue::Null => append_token(material, "value.kind", "null")?,
        CanonicalBasisValue::Bool(value) => {
            append_token(material, "value.kind", "bool")?;
            append_token(
                material,
                "value.bool",
                if *value { "true" } else { "false" },
            )?;
        }
        CanonicalBasisValue::SignedInteger { width, value } => {
            append_token(material, "value.kind", "signed")?;
            append_token(material, "value.width", integer_width_token(*width))?;
            append_i128(material, "value.signed", *value)?;
        }
        CanonicalBasisValue::UnsignedInteger { width, value } => {
            append_token(material, "value.kind", "unsigned")?;
            append_token(material, "value.width", integer_width_token(*width))?;
            append_u128(material, "value.unsigned", *value)?;
        }
        CanonicalBasisValue::FloatBits { width, bits } => {
            append_token(material, "value.kind", "float")?;
            append_token(material, "value.width", float_width_token(*width))?;
            append_u64(material, "value.float-bits", *bits)?;
        }
        CanonicalBasisValue::ExactText(value) => {
            append_token(material, "value.kind", "text")?;
            append_interned_string(material, "value.text", value)?;
        }
        CanonicalBasisValue::BytesDigest(value) => {
            append_token(material, "value.kind", "bytes-digest")?;
            append_bytes(material, "value.bytes-digest", value.bytes())?;
        }
        CanonicalBasisValue::DecimalText(value) => {
            append_token(material, "value.kind", "decimal")?;
            append_interned_string(material, "value.decimal", value)?;
        }
        CanonicalBasisValue::BigIntText(value) => {
            append_token(material, "value.kind", "bigint")?;
            append_interned_string(material, "value.bigint", value)?;
        }
        CanonicalBasisValue::RationalText {
            numerator,
            denominator,
        } => {
            append_token(material, "value.kind", "rational")?;
            append_interned_string(material, "value.rational.numerator", numerator)?;
            append_interned_string(material, "value.rational.denominator", denominator)?;
        }
        CanonicalBasisValue::BytesRefId(value) => {
            append_token(material, "value.kind", "bytes-ref")?;
            append_u64(material, "value.bytes-ref", *value)?;
        }
        CanonicalBasisValue::ContentRefId(value) => {
            append_token(material, "value.kind", "content-ref")?;
            append_u64(material, "value.content-ref", *value)?;
        }
        CanonicalBasisValue::EntityRef {
            partition_id,
            local_slot,
            generation,
        } => {
            append_token(material, "value.kind", "entity-ref")?;
            append_u64(material, "value.entity.partition", u64::from(*partition_id))?;
            append_u64(material, "value.entity.slot", *local_slot)?;
            append_u64(material, "value.entity.generation", u64::from(*generation))?;
        }
        CanonicalBasisValue::DateDays(value) => {
            append_token(material, "value.kind", "date-days")?;
            append_i64(material, "value.date-days", i64::from(*value))?;
        }
        CanonicalBasisValue::TimeNanos(value) => {
            append_token(material, "value.kind", "time-nanos")?;
            append_u64(material, "value.time-nanos", *value)?;
        }
        CanonicalBasisValue::TimestampMicros(value) => {
            append_token(material, "value.kind", "timestamp-micros")?;
            append_i64(material, "value.timestamp-micros", *value)?;
        }
        CanonicalBasisValue::TimestampTz {
            utc_micros_since_unix_epoch,
            offset_minutes,
        } => {
            append_token(material, "value.kind", "timestamp-tz")?;
            append_i64(
                material,
                "value.timestamp-tz.utc-micros",
                *utc_micros_since_unix_epoch,
            )?;
            append_i32(
                material,
                "value.timestamp-tz.offset-minutes",
                *offset_minutes,
            )?;
        }
        CanonicalBasisValue::UuidBytes(bytes) => {
            append_token(material, "value.kind", "uuid")?;
            append_bytes(material, "value.uuid", bytes)?;
        }
        CanonicalBasisValue::NestedSequence(value) => {
            append_token(material, "value.kind", "nested-sequence")?;
            append_u64(material, "value.nested-sequence", u64::from(*value))?;
        }
    }
    Ok(())
}

pub(crate) fn append_struct_value_material(
    material: &mut CanonicalMaterialWriter,
    value: &StructAspectValue,
) -> CanonicalMaterialResult {
    append_token(material, "value.kind", "struct")?;
    append_u64(
        material,
        "value.struct.field-count",
        value.fields().count() as u64,
    )?;
    for (field_key, field_value) in value.fields() {
        append_token(material, "value.struct.field", field_key.as_str())?;
        append_value_material(
            material,
            &crate::canonicalization::value_lowering::canonical_basis_value_for_aspect_value(
                field_value,
            ),
        )?;
    }
    Ok(())
}

pub(super) fn append_interned_string(
    material: &mut CanonicalMaterialWriter,
    label: &str,
    value: &InternedString,
) -> CanonicalMaterialResult {
    match value {
        InternedString::Raw(value) => append_token_parts(material, label, ".raw", &[value]),
        InternedString::Symbol(symbol) => {
            material.append(label)?;
            append_u64(material, ".symbol", u64::from(symbol.0))
        }
    }
}

fn integer_width_token(width: CanonicalIntegerWidth) -> &'static str {
    match width {
        CanonicalIntegerWidth::Bits8 => "i8",
        CanonicalIntegerWidth::Bits16 => "i16",
        CanonicalIntegerWidth::Bits32 => "i32",
        CanonicalIntegerWidth::Bits64 => "i64",
        CanonicalIntegerWidth::Bits128 => "i128",
    }
}

fn float_width_token(width: CanonicalFloatWidth) -> &'static str {
    match width {
        CanonicalFloatWidth::Bits32 => "f32",
        CanonicalFloatWidth::Bits64 => "f64",
    }
}
