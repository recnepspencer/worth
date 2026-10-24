//! Historical output fields use the existing schema-field wire representation.
use crate::record::application_schema::{
    member::schema,
    wire_vocabulary::{decode_optional, decode_type_identity, write_optional, write_type_identity},
};
use crate::record::sequence::{decode_sequence, write_sequence};
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
    record::decode_budget::RecordDecodeAttempt,
};
use worth_query_declaration::facade::{
    application_aftermath::{DeclaredCorrectionOutputField, DeclaredCorrectionOutputFieldParts},
    application_schema::ApplicationSchemaMember,
    portable_identity::WorthQueryPortableTypeIdentity,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    fields: &[DeclaredCorrectionOutputField],
) -> Result<(), Denial> {
    write_sequence(output, fields, |output, field| {
        let parts = field.parts();
        let member = field.projected_value_member();
        schema::require_supported_field_codec(&member)?;
        write_type_identity(output, &parts.source_binding)?;
        output.text(&parts.role)?;
        write_optional(output, parts.binding_unit.as_ref(), |output, unit| {
            output.text(unit)
        })?;
        output.u16(if parts.frame.is_some() { 25 } else { 3 })?;
        schema::write(output, &member)
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<DeclaredCorrectionOutputField>, Denial> {
    decode_sequence(input, budget, 14, |input, _| {
        let source_binding = decode_type_identity(input)?;
        let role = input.text()?.to_owned();
        let binding_unit = decode_optional(input, |input| Ok(input.text()?.to_owned()))?;
        let tag = input.u16()?;
        if !matches!(tag, 3 | 25) {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        let ApplicationSchemaMember::Field {
            entity,
            aspect,
            field,
            presence,
            scalar_family,
            value_type,
            canonical,
            unit,
            frame,
            writable: false,
            equality_queryable: false,
            unique: false,
        } = schema::decode(tag, input)?
        else {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        };
        Ok(DeclaredCorrectionOutputField::from_untrusted_parts(
            DeclaredCorrectionOutputFieldParts {
                source_binding,
                role,
                entity,
                aspect,
                field,
                presence,
                scalar_family,
                value_binding: WorthQueryPortableTypeIdentity::from_untrusted(value_type),
                unit,
                binding_unit,
                frame,
                canonical,
            },
        ))
    })
}
