//! Portable actor sources must name real operation and principal contracts before installation.
use super::{ApplicationOperationBodyCreatedField, ApplicationOperationBodyProgram};
use crate::application_schema::{
    ApplicationFieldPresence, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};
use crate::{
    application_schema::ApplicationFieldBindingLocus,
    portable_identity::WorthQueryPortableTypeIdentity,
};
use worth_foundational::facade::ScalarAspectType;

/// Descriptive correspondence; native installation still binds the actual typed executor.
pub(super) struct PrincipalSourceContract<'a> {
    pub scope: &'a str,
    pub mutation_binding: &'a str,
    pub principal_binding: &'a str,
    pub field: &'a ApplicationFieldBindingLocus,
    pub binding: &'a WorthQueryPortableTypeIdentity,
    pub scalar_family: ScalarAspectType,
}
impl PrincipalSourceContract<'_> {
    pub fn is_declared_for(&self, operation: &str, members: &[Member]) -> bool {
        members.iter().any(|member| matches!(member, Member::ApplicationMutation {description}
            if description.binding_identity().as_str() == self.mutation_binding && description.operation() == operation && description.scope().entity == self.scope))
        && members.iter().any(|member| matches!(member, Member::PrincipalBinding {
            binding, principal_entity, principal_identity_aspect, principal_identity_field, principal_identity_scalar_family, principal_identity_value_type, ..
        } if binding == self.principal_binding && principal_entity == self.field.entity() && principal_identity_aspect == self.field.aspect()
            && principal_identity_field == self.field.field() && *principal_identity_scalar_family == self.scalar_family && principal_identity_value_type == self.binding.as_str()))
        && members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, value_type, scalar_family, presence: ApplicationFieldPresence::Required, unit: None, .. }
            if entity == self.field.entity() && aspect == self.field.aspect() && field == self.field.field() && value_type == self.binding.as_str() && *scalar_family == self.scalar_family))
    }
}

pub(super) fn validate_principals(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[Member],
) -> Result<(), Denial> {
    let denied = || Denial::program_body_dependency(operation);
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
        let contract = PrincipalSourceContract {
            scope: body.scope_entity().ok_or_else(denied)?,
            mutation_binding,
            principal_binding,
            field: source,
            binding: step.binding(),
            scalar_family: step.scalar_family(),
        };
        if !contract.is_declared_for(operation, members)
            || !members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, value_type, scalar_family, unit: None, .. }
                if entity == step.field().entity() && aspect == step.field().aspect() && field == step.field().field() && value_type == step.binding().as_str() && *scalar_family == step.scalar_family()))
        { return Err(denied()); }
        if !members.iter().any(|member| {
            matches!(member, Member::OperationDecisionRead { operation: candidate, target }
            if candidate == operation && Some(target.clone()) == step.source_read())
        }) {
            return Err(Denial::decision_read_body_dependency(operation));
        }
    }
    Ok(())
}
