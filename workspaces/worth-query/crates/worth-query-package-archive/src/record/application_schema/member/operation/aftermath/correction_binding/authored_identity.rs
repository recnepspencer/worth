//! Authored family/key correspondence uses the existing schema-field wire owner.
use crate::record::application_schema::{
    member::schema,
    wire_vocabulary::{decode_optional, decode_type_identity, write_optional, write_type_identity},
};
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::{
    application_aftermath::{DeclaredAuthoredCorrection, DeclaredAuthoredCorrectionParts},
    application_operation::ApplicationMutationOutputPosture as Posture,
    application_schema::ApplicationSchemaMember,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    identity: &DeclaredAuthoredCorrection,
) -> Result<(), Denial> {
    let parts = identity.parts();
    schema::require_supported_field_codec(&parts.logical_key)?;
    let ApplicationSchemaMember::Field { frame, .. } = parts.logical_key.as_ref() else {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    };
    write_type_identity(output, &parts.family)?;
    output.text(&parts.source_role)?;
    output.text(&parts.inverse_role)?;
    output.u16(match parts.source_posture {
        Posture::Create => 0,
        Posture::Preserve => 1,
        Posture::Retire => 2,
    })?;
    write_optional(output, parts.binding_unit.as_ref(), |output, value| {
        output.text(value)
    })?;
    output.u16(if frame.is_some() { 25 } else { 3 })?;
    schema::write(output, &parts.logical_key)
}

pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<DeclaredAuthoredCorrection, Denial> {
    let family = decode_type_identity(input)?;
    let source_role = input.text()?.to_owned();
    let inverse_role = input.text()?.to_owned();
    let source_posture = match input.u16()? {
        0 => Posture::Create,
        1 => Posture::Preserve,
        2 => Posture::Retire,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let binding_unit = decode_optional(input, |input| Ok(input.text()?.to_owned()))?;
    let tag = input.u16()?;
    if !matches!(tag, 3 | 25) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let logical_key = schema::decode(tag, input)?;
    Ok(DeclaredAuthoredCorrection::from_untrusted_parts(
        DeclaredAuthoredCorrectionParts {
            family,
            logical_key: Box::new(logical_key),
            binding_unit,
            source_role,
            inverse_role,
            source_posture,
        },
    ))
}
