use super::scalar_view::{CanonicalScalarView, CanonicalTextView};
use super::sink::CanonicalMaterialSink;
use super::token_writer::{
    append_bytes, append_i128, append_i32, append_i64, append_token, append_token_parts,
    append_u128, append_u64,
};
use super::writer::{CanonicalMaterialResult, CanonicalMaterialWriter};
use crate::aspects::StructAspectValue;
use crate::canonicalization::{CanonicalBasisValue, CanonicalFloatWidth, CanonicalIntegerWidth};
use crate::values::{AspectValue, InternedString};

#[cfg(test)]
pub(super) fn value_material(value: &CanonicalBasisValue) -> String {
    let mut material = CanonicalMaterialWriter::owned();
    append_value_material(&mut material, value)
        .expect("an unbounded canonical material writer accepts a typed value");
    String::from_utf8(material.finish().into_bytes())
        .expect("canonical material grammar writes UTF-8")
}

pub(crate) fn aspect_value_material(value: &AspectValue) -> String {
    let mut material = CanonicalMaterialWriter::owned();
    write_aspect_value_identity_material(value, &mut material)
        .expect("owned canonical material accepts a typed scalar");
    material.finish_string()
}

pub(crate) fn struct_value_material(value: &StructAspectValue) -> String {
    let mut material = CanonicalMaterialWriter::owned();
    append_struct_value_material(&mut material, value)
        .expect("an unbounded canonical material writer accepts a typed struct");
    String::from_utf8(material.finish().into_bytes())
        .expect("canonical material grammar writes UTF-8")
}

pub(crate) fn append_value_material(
    material: &mut CanonicalMaterialWriter,
    value: &CanonicalBasisValue,
) -> CanonicalMaterialResult {
    append_scalar_material(material, value.into())
}

/// Writes borrowed canonical scalar material into the caller's fallible sink.
/// This is representation only, not canonical readiness or identity authority.
/// Encoding uses constant stack space and allocates no intermediate text.
pub fn write_aspect_value_identity_material<S: CanonicalMaterialSink>(
    value: &AspectValue,
    material: &mut S,
) -> Result<(), S::Error> {
    append_scalar_material(material, value.into())
}

fn append_scalar_material<S: CanonicalMaterialSink>(
    material: &mut S,
    value: CanonicalScalarView<'_>,
) -> Result<(), S::Error> {
    match value {
        CanonicalScalarView::Null => append_token(material, "value.kind", "null")?,
        CanonicalScalarView::Bool(value) => {
            append_token(material, "value.kind", "bool")?;
            append_token(material, "value.bool", if value { "true" } else { "false" })?;
        }
        CanonicalScalarView::SignedInteger { width, value } => {
            append_token(material, "value.kind", "signed")?;
            append_token(material, "value.width", integer_width_token(width))?;
            append_i128(material, "value.signed", value)?;
        }
        CanonicalScalarView::UnsignedInteger { width, value } => {
            append_token(material, "value.kind", "unsigned")?;
            append_token(material, "value.width", integer_width_token(width))?;
            append_u128(material, "value.unsigned", value)?;
        }
        CanonicalScalarView::FloatBits { width, bits } => {
            append_token(material, "value.kind", "float")?;
            append_token(material, "value.width", float_width_token(width))?;
            append_u64(material, "value.float-bits", bits)?;
        }
        CanonicalScalarView::ExactText(value) => {
            append_token(material, "value.kind", "text")?;
            append_text_view(material, "value.text", value)?;
        }
        CanonicalScalarView::BytesDigest(value) => {
            append_token(material, "value.kind", "bytes-digest")?;
            append_bytes(material, "value.bytes-digest", value)?;
        }
        CanonicalScalarView::DecimalText(value) => {
            append_token(material, "value.kind", "decimal")?;
            append_text_view(material, "value.decimal", value)?;
        }
        CanonicalScalarView::BigIntText(value) => {
            append_token(material, "value.kind", "bigint")?;
            append_text_view(material, "value.bigint", value)?;
        }
        CanonicalScalarView::RationalText {
            numerator,
            denominator,
        } => {
            append_token(material, "value.kind", "rational")?;
            append_text_view(material, "value.rational.numerator", numerator)?;
            append_text_view(material, "value.rational.denominator", denominator)?;
        }
        CanonicalScalarView::BytesRefId(value) => {
            append_token(material, "value.kind", "bytes-ref")?;
            append_u64(material, "value.bytes-ref", value)?;
        }
        CanonicalScalarView::ContentRefId(value) => {
            append_token(material, "value.kind", "content-ref")?;
            append_u64(material, "value.content-ref", value)?;
        }
        CanonicalScalarView::EntityRef {
            partition_id,
            local_slot,
            generation,
        } => {
            append_token(material, "value.kind", "entity-ref")?;
            append_u64(material, "value.entity.partition", u64::from(partition_id))?;
            append_u64(material, "value.entity.slot", local_slot)?;
            append_u64(material, "value.entity.generation", u64::from(generation))?;
        }
        CanonicalScalarView::DateDays(value) => {
            append_token(material, "value.kind", "date-days")?;
            append_i64(material, "value.date-days", i64::from(value))?;
        }
        CanonicalScalarView::TimeNanos(value) => {
            append_token(material, "value.kind", "time-nanos")?;
            append_u64(material, "value.time-nanos", value)?;
        }
        CanonicalScalarView::TimestampMicros(value) => {
            append_token(material, "value.kind", "timestamp-micros")?;
            append_i64(material, "value.timestamp-micros", value)?;
        }
        CanonicalScalarView::TimestampTz {
            utc_micros_since_unix_epoch,
            offset_minutes,
        } => {
            append_token(material, "value.kind", "timestamp-tz")?;
            append_i64(
                material,
                "value.timestamp-tz.utc-micros",
                utc_micros_since_unix_epoch,
            )?;
            append_i32(
                material,
                "value.timestamp-tz.offset-minutes",
                offset_minutes,
            )?;
        }
        CanonicalScalarView::UuidBytes(bytes) => {
            append_token(material, "value.kind", "uuid")?;
            append_bytes(material, "value.uuid", bytes)?;
        }
        CanonicalScalarView::NestedSequence(value) => {
            append_token(material, "value.kind", "nested-sequence")?;
            append_u64(material, "value.nested-sequence", u64::from(value))?;
        }
    }
    Ok(())
}

pub(crate) fn append_struct_value_material(
    material: &mut CanonicalMaterialWriter,
    value: &StructAspectValue,
) -> CanonicalMaterialResult {
    write_struct_aspect_value_identity_material(value, material)
}

/// Writes fields in the StructAspectValue's canonical order without cloning them.
/// Refusal propagates the sink's exact error and preserves its accepted prefix.
pub fn write_struct_aspect_value_identity_material<S: CanonicalMaterialSink>(
    value: &StructAspectValue,
    material: &mut S,
) -> Result<(), S::Error> {
    append_token(material, "value.kind", "struct")?;
    append_u64(
        material,
        "value.struct.field-count",
        value.fields().count() as u64,
    )?;
    for (field_key, field_value) in value.fields() {
        append_token(material, "value.struct.field", field_key.as_str())?;
        write_aspect_value_identity_material(field_value, material)?;
    }
    Ok(())
}

pub(super) fn append_interned_string(
    material: &mut CanonicalMaterialWriter,
    label: &str,
    value: &InternedString,
) -> CanonicalMaterialResult {
    append_text_view(material, label, value.into())
}

fn append_text_view<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: CanonicalTextView<'_>,
) -> Result<(), S::Error> {
    match value {
        CanonicalTextView::Raw(value) => append_token_parts(material, label, ".raw", &[value]),
        CanonicalTextView::Symbol(symbol) => {
            material.append(label)?;
            append_u64(material, ".symbol", u64::from(symbol))
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
