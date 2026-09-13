use std::collections::BTreeSet;

use super::{ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember};
use crate::application_operation::ApplicationMutationDescription;

pub(super) fn validate_identifiers(
    description: &ApplicationMutationDescription,
) -> Result<(), Denial> {
    let scope = description.scope();
    let identities = [
        description.binding_identity(),
        description.input_identity(),
        description.result_identity(),
        description.denial_identity(),
    ];
    if identities.iter().any(|identity| !identity.is_valid()) {
        return Err(Denial::InvalidApplicationMutationDescription);
    }
    for name in [
        description.operation(),
        &scope.entity,
        &scope.aspect,
        &scope.field,
    ] {
        super::identifier_validation::validate_simple_identifier(name)?;
    }
    let mut names = BTreeSet::new();
    for role in description.output_roles() {
        super::identifier_validation::validate_portable_type_identifier(&role.name)?;
        super::identifier_validation::validate_simple_identifier(&role.entity)?;
        if !names.insert(&role.name) {
            return Err(Denial::InvalidApplicationMutationDescription);
        }
    }
    Ok(())
}

pub(super) fn validate_dependencies(members: &[ApplicationSchemaMember]) -> Result<(), Denial> {
    for member in members {
        let ApplicationSchemaMember::ApplicationMutation { description } = member else {
            continue;
        };
        let scope = description.scope();
        let operation_exists = members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::Operation { operation, input_type }
                if operation == description.operation() && input_type == description.input_identity()));
        let scope_exists = members.iter().any(|member| {
            matches!(member,
            ApplicationSchemaMember::Field { entity, aspect, field, equality_queryable: true, .. }
                if entity == &scope.entity && aspect == &scope.aspect && field == &scope.field)
        });
        let outputs_exist = description.output_roles().iter().all(|role| {
            members.iter().any(|member|
            matches!(member, ApplicationSchemaMember::Entity { entity } if entity == &role.entity))
        });
        if !operation_exists || !scope_exists || !outputs_exist {
            return Err(Denial::MissingApplicationMutationDependency);
        }
    }
    Ok(())
}
