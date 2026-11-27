//! Principal-field constraints of an extended authorization path.
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationFieldConstraint,
    WorthQueryPortableApplicationAuthorizationFieldConstraintParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    constraint: &ApplicationAuthorizationFieldConstraint,
) -> Result<(), Denial> {
    let parts = constraint.parts();
    super::super::wire_vocabulary::write_usize(output, parts.traversal_ordinal)?;
    output.text(&parts.entity)?;
    output.text(&parts.aspect)?;
    output.text(&parts.field)?;
    output.u16(super::comparison_tag(parts.comparison))?;
    output.text(&parts.principal_aspect)?;
    output.text(&parts.principal_field)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationAuthorizationFieldConstraint, Denial> {
    Ok(
        ApplicationAuthorizationFieldConstraint::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableApplicationAuthorizationFieldConstraintParts {
                traversal_ordinal: super::super::wire_vocabulary::decode_usize(input)?,
                entity: input.text()?.to_owned(),
                aspect: input.text()?.to_owned(),
                field: input.text()?.to_owned(),
                comparison: super::comparison_from_tag(input.u16()?)?,
                principal_aspect: input.text()?.to_owned(),
                principal_field: input.text()?.to_owned(),
            },
        ),
    )
}
