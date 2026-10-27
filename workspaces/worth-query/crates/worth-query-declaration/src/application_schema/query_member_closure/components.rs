use super::*;
pub(super) fn validate(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let Some(selection) = definition.component_selection() else {
        return Ok(());
    };
    if members.iter().any(|member| {
        matches!(member,
            ApplicationSchemaMember::ComponentSet { definition: set }
            if set.name() == selection.set() && set.member() == selection.member()
        )
    }) {
        Ok(())
    } else {
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery)
    }
}

#[cfg(test)]
mod tests;
