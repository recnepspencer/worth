//! Principal field syntax retains its binding owner and full flattened record path.
use super::{validate_field_locus, validate_simple_identifier, ApplicationSchemaDeclarationDenial};

pub(super) fn validate(
    binding: &str,
    mapping: &str,
    relation: &str,
    principal: &str,
    fields: [(&str, &str, &str); 3],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for identifier in [binding, mapping, relation, principal] {
        validate_simple_identifier(identifier)?;
    }
    for (entity, aspect, field) in fields {
        validate_field_locus(entity, aspect, field).map_err(|_| {
            ApplicationSchemaDeclarationDenial::InvalidPrincipalBindingField {
                binding: binding.to_owned(),
                field: crate::application_schema::ApplicationFieldBindingLocus::new(
                    entity, aspect, field,
                ),
            }
        })?;
    }
    Ok(())
}
