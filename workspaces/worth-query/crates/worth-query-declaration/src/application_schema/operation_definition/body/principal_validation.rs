//! Portable actor sources must name real operation and principal contracts before installation.
use super::{ApplicationOperationBodyCreatedField, ApplicationOperationBodyProgram};
use crate::application_schema::{
    ApplicationFieldPresence, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};

pub(super) fn validate_principals(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[Member],
) -> Result<(), Denial> {
    let denied = Denial::MissingOperationProgramDependency;
    for step in body
        .creation()
        .iter()
        .flat_map(|creation| creation.fields())
    {
        let ApplicationOperationBodyCreatedField::PrincipalIdentity {
            mutation_binding,
            principal_binding,
            source,
            ..
        } = step
        else {
            continue;
        };
        if !members.iter().any(|member| matches!(member, Member::ApplicationMutation {description}
            if description.binding_identity().as_str() == mutation_binding && description.operation() == operation && Some(description.scope().entity.as_str()) == body.scope_entity()))
            || !members.iter().any(|member| matches!(member, Member::PrincipalBinding {
                binding, principal_entity, principal_identity_aspect, principal_identity_field, principal_identity_scalar_family, principal_identity_value_type, ..
            } if binding == principal_binding && principal_entity == source.entity() && principal_identity_aspect == source.aspect()
                && principal_identity_field == source.field() && *principal_identity_scalar_family == step.scalar_family() && principal_identity_value_type == step.binding().as_str()))
            || !members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, value_type, scalar_family, presence: ApplicationFieldPresence::Required, unit: None, .. }
                if entity == source.entity() && aspect == source.aspect() && field == source.field() && value_type == step.binding().as_str() && *scalar_family == step.scalar_family()))
            || !members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, value_type, scalar_family, unit: None, .. }
                if entity == step.field().entity() && aspect == step.field().aspect() && field == step.field().field() && value_type == step.binding().as_str() && *scalar_family == step.scalar_family()))
        { return Err(denied); }
        if !members.iter().any(|member| {
            matches!(member, Member::OperationDecisionRead { operation: candidate, target }
            if candidate == operation && Some(target.clone()) == step.source_read())
        }) {
            return Err(Denial::MissingOperationDecisionReadDependency);
        }
    }
    Ok(())
}
