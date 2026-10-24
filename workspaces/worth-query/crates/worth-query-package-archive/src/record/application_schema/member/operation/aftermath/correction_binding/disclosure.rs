//! Disclosure metadata belonging to the corrective action association.
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::application_schema::wire_vocabulary::decode_type_identity;
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};
use worth_query_declaration::facade::{
    application_aftermath::{DeclaredCorrectionDisclosure, DeclaredCorrectionDisclosureParts},
    application_query::ApplicationQueryScopeResolutionMode,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    disclosure: &DeclaredCorrectionDisclosure,
) -> Result<(), Denial> {
    for (_, value) in disclosure.canonical_fields() {
        output.text(value)?;
    }
    Ok(())
}

pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<DeclaredCorrectionDisclosure, Denial> {
    Ok(DeclaredCorrectionDisclosure::from_untrusted_parts(
        DeclaredCorrectionDisclosureParts {
            binding: decode_type_identity(input)?,
            input: decode_type_identity(input)?,
            query_name: input.text()?.to_owned(),
            query: decode_type_identity(input)?,
            parameters: decode_type_identity(input)?,
            result: decode_type_identity(input)?,
            scope: decode_type_identity(input)?,
            scope_entity: input.text()?.to_owned(),
            scope_aspect: input.text()?.to_owned(),
            scope_field: input.text()?.to_owned(),
            scope_value: decode_type_identity(input)?,
            scope_resolution: match input.text()? {
                "input-field" => ApplicationQueryScopeResolutionMode::InputField,
                "principal-identity" => ApplicationQueryScopeResolutionMode::PrincipalIdentity,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            },
            scope_writable: match input.text()? {
                "true" => true,
                "false" => false,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            },
            principal_binding: input.text()?.to_owned(),
        },
    ))
}
