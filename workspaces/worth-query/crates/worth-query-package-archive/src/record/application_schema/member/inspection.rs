//! Additive member45/v1. No observation, permission, or retained root is decoded.
use super::*;
use crate::record::application_schema::query_binding_reference;
use worth_query_declaration::facade::application_query::{
    ErasedApplicationQueryInspectionDefinition as Definition,
    WorthQueryPortableApplicationQueryInspectionParts as Parts,
};

pub(super) fn require_shape(member: &ApplicationSchemaMember) -> Result<(), Denial> {
    if let ApplicationSchemaMember::ApplicationQueryInspection { definition } = member {
        definition
            .validate()
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    }
    Ok(())
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &Definition,
) -> Result<(), Denial> {
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let parts = definition.parts();
    output.u16(parts.semantic_version)?;
    query_binding_reference::write(output, &parts.binding)?;
    crate::record::application_query::disclosure::write(output, &parts.metadata_disclosure)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    let semantic_version = input.u16()?;
    // Reject unknown payload grammar before allocating any field.
    if semantic_version != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    budget.require_nesting_depth(2)?;
    let binding = query_binding_reference::decode(input)?;
    let metadata_disclosure = crate::record::application_query::disclosure::decode(input, budget)?;
    let member = ApplicationSchemaMember::ApplicationQueryInspection {
        definition: Definition::from_untrusted_parts(Parts {
            semantic_version,
            binding,
            metadata_disclosure,
        }),
    };
    require_shape(&member)?;
    Ok(member)
}
