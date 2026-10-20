//! Readmission checks portable output meaning against the declared mutation.
use crate::application_operation::{
    ApplicationMutationDescription, ApplicationMutationOutputPosture,
};
use crate::application_schema::{
    ApplicationOperationBodyOutputTarget, ApplicationOperationBodyProgram,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember,
};
use std::collections::BTreeSet;

pub(super) fn validate_outputs(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[ApplicationSchemaMember],
) -> Result<(), Denial> {
    let denied = Denial::MissingOperationProgramDependency;
    let mut roles = BTreeSet::new();
    for output in body.outputs() {
        if !roles.insert(&output.role.name)
            || body
                .outputs()
                .first()
                .is_some_and(|first| first.binding != output.binding)
        {
            return Err(Denial::DuplicateMember);
        }
        let binding = members
            .iter()
            .find_map(|member| match member {
                ApplicationSchemaMember::ApplicationMutation { description }
                    if description.binding_identity().as_str() == output.binding =>
                {
                    Some(description)
                }
                _ => None,
            })
            .ok_or(denied)?;
        if binding.operation() != operation
            || binding.scope().entity.as_str() != body.scope_entity().unwrap_or_default()
            || !binding.output_roles().contains(&output.role)
        {
            return Err(denied);
        }
        let target_valid = match &output.target {
            ApplicationOperationBodyOutputTarget::Scope => {
                output.role.posture == ApplicationMutationOutputPosture::Preserve
                    && Some(output.role.entity.as_str()) == body.scope_entity()
            }
            ApplicationOperationBodyOutputTarget::Creation { key } => {
                output.role.posture == ApplicationMutationOutputPosture::Create
                    && body.creation().is_some_and(|creation| {
                        creation.key() == key && creation.entity() == output.role.entity
                    })
            }
        };
        if !target_valid {
            return Err(denied);
        }
    }
    // Each binding of this operation must be satisfiable by its one installed body.
    for binding in members.iter().filter_map(|member| match member {
        ApplicationSchemaMember::ApplicationMutation { description }
            if description.operation() == operation =>
        {
            Some(description)
        }
        _ => None,
    }) {
        validate_coverage(body, binding)?;
    }
    Ok(())
}

fn validate_coverage(
    body: &ApplicationOperationBodyProgram,
    binding: &ApplicationMutationDescription,
) -> Result<(), Denial> {
    if binding.output_roles().iter().any(|role| {
        !role.cardinality.admits_absence()
            && !body.outputs().iter().any(|output| {
                output.binding == binding.binding_identity().as_str() && output.role == *role
            })
    }) || binding
        .output_role_families()
        .iter()
        .any(|family| family.minimum > 0)
    {
        return Err(Denial::MissingOperationProgramDependency);
    }
    Ok(())
}
