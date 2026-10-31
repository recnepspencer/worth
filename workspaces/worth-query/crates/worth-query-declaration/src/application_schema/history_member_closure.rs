//! History associations resolve existing ordinary contracts, never install new operations.
use super::{ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member};

pub(super) fn validate(members: &[Member]) -> Result<(), Denial> {
    for member in members {
        let Member::ApplicationHistory { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        definition
            .validate()
            .map_err(|_| Denial::InvalidApplicationHistory)?;
        let access = parts.access.parts();
        if !members.iter().any(|member| {
            matches!(member,
            Member::ApplicationQuery { definition: query }
            if query.name() == access.query_name && query.query_type() == access.query.as_str()
                && query.parameter_type() == access.parameters.as_str()
                && query.result_type() == access.result.as_str()
                && query.scope_entity() == access.scope_entity
                && query.scope_type() == access.scope.as_str())
        }) {
            return Err(Denial::MissingApplicationDocumentHistoryDependency);
        }
        for operation in &parts.operations {
            if !members.iter().any(|member| {
                matches!(member,
                Member::ApplicationMutation { description }
                if description.binding_identity().as_str() == operation.binding
                    && description.scope().entity == access.scope_entity)
            }) {
                return Err(Denial::MissingApplicationDocumentHistoryDependency);
            }
        }
    }
    Ok(())
}
