//! Every filter field operand is a declared, unitless root field whose
//! scalar family and presence match the operand's admitted type.
use crate::application_query::{
    ApplicationQueryWhereFilterSource as Source, ErasedApplicationQueryDefinition,
};
use crate::application_schema::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

pub(super) fn validate(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let Some(filter) = definition.where_filter() else {
        return Ok(());
    };
    for operand in filter.operands() {
        let Source::Field {
            entity,
            aspect,
            field,
            scalar_family,
            presence,
        } = operand.source()
        else {
            continue;
        };
        let declared = members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Field {
                    entity: declared_entity,
                    aspect: declared_aspect,
                    field: declared_field,
                    scalar_family: declared_scalar,
                    presence: declared_presence,
                    unit: None,
                    ..
                } if declared_entity == entity
                    && declared_aspect == aspect
                    && declared_field == field
                    && declared_scalar == scalar_family
                    && declared_presence == presence
            )
        });
        if entity != definition.root_entity() || !declared {
            return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
        }
    }
    Ok(())
}
