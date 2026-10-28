use super::{ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member};
use crate::application_query::ApplicationQueryAuthorizationRequirement;
use crate::application_resource::{ApplicationResourceAction, ErasedApplicationResourceDefinition};
use std::collections::BTreeSet;

pub(super) fn validate(members: &[Member]) -> Result<(), Denial> {
    let mut names = BTreeSet::new();
    for definition in members.iter().filter_map(|member| match member {
        Member::ApplicationResource { definition } => Some(definition),
        _ => None,
    }) {
        if !names.insert(&definition.parts().name) {
            return Err(Denial::DuplicateApplicationResource);
        }
        definition
            .validate()
            .map_err(|_| Denial::InvalidApplicationResource)?;
        validate_access(definition, members)?;
        validate_authorization(definition, members)?;
        let disclosure = &definition.parts().disclosure;
        if let (Some(name), Some(identity)) =
            (disclosure.capability_name(), disclosure.capability_type())
            && !members.iter().any(|member| {
                matches!(member,
                Member::ApplicationCapability { contract }
                if contract.name() == name && contract.capability_type() == identity)
            })
        {
            return Err(Denial::MissingApplicationResourceDependency);
        }
    }
    Ok(())
}

fn validate_access(
    definition: &ErasedApplicationResourceDefinition,
    members: &[Member],
) -> Result<(), Denial> {
    let access = definition.parts().access.parts();
    if members.iter().any(|member| matches!(member,
        Member::ApplicationQuery { definition: query }
        if query.name() == access.query_name && query.query_type() == access.query.as_str()
            && query.parameter_type() == access.parameters.as_str()
            && query.result_type() == access.result.as_str()
            && query.scope_entity() == access.scope_entity && query.scope_type() == access.scope.as_str())) {
        Ok(())
    } else { Err(Denial::MissingApplicationResourceDependency) }
}

fn validate_authorization(
    definition: &ErasedApplicationResourceDefinition,
    members: &[Member],
) -> Result<(), Denial> {
    for action in ApplicationResourceAction::ALL {
        let ApplicationQueryAuthorizationRequirement::Ability {
            ability,
            scope_entity,
        } = definition.parts().authorization.requirement(action)
        else {
            continue;
        };
        if !members.iter().any(|member| {
            matches!(member,
            Member::Ability { ability: declared, scope_entity: scope }
            if declared == ability && scope == scope_entity)
        }) || !members.iter().any(|member| {
            matches!(member,
                Member::AbilityPolicy { ability: declared, .. } if declared == ability)
        }) {
            return Err(Denial::MissingApplicationResourceDependency);
        }
    }
    Ok(())
}
