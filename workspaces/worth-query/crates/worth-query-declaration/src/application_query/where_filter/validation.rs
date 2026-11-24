//! Definition-level admission of a where filter against its query.
use super::ApplicationQueryWhereFilterSource as Source;
use crate::application_query::{
    ApplicationQueryDefinitionDenial as Denial, WorthQueryPortableApplicationQueryParts,
};

/// The filter reads only the root entity's own fields and declared
/// parameters of the same scalar family, and narrows only the selection
/// shapes whose candidates are root rows: the exact scope root and an
/// installed root-path union.
pub(in crate::application_query) fn validate(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    let Some(filter) = definition.where_filter() else {
        return Ok(());
    };
    if definition.orbit().is_some()
        || definition.component_selection().is_some()
        || definition.managed_index().is_some()
        || definition.continuation().is_some()
    {
        return Err(Denial::WhereFilterUnsupportedShape);
    }
    for operand in filter.operands() {
        match operand.source() {
            Source::Field { entity, .. } => {
                if entity != definition.root_entity() {
                    return Err(Denial::WhereFilterFieldOutsideRoot);
                }
            }
            Source::Parameter {
                parameter,
                scalar_family,
            }
            | Source::OptionalParameter {
                parameter,
                scalar_family,
            } => {
                let Some(declared) = definition
                    .parameters()
                    .iter()
                    .find(|declared| declared.name() == parameter)
                else {
                    return Err(Denial::WhereFilterUnknownParameter);
                };
                if declared.scalar_family() != *scalar_family
                    || declared.unit().is_some()
                    || declared.is_optional()
                        != matches!(operand.source(), Source::OptionalParameter { .. })
                {
                    return Err(Denial::WhereFilterParameterMismatch);
                }
            }
            Source::Literal { .. } => {}
        }
    }
    Ok(())
}
