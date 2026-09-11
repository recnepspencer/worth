use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::ApplicationSchemaMember;

mod aftermath;
mod authorization;
mod capability;
mod field;
mod operation;
mod principal_binding;
mod schema;
mod vocabulary;

pub(super) fn append_member(
    basis: &mut ApplicationSchemaCanonicalBasis,
    index: usize,
    member: &ApplicationSchemaMember,
    revised: bool,
) {
    let prefix = format!("member[{index}]");
    match member {
        ApplicationSchemaMember::Entity { .. }
        | ApplicationSchemaMember::Aspect { .. }
        | ApplicationSchemaMember::Field { .. }
        | ApplicationSchemaMember::Relation { .. }
        | ApplicationSchemaMember::PrincipalBinding { .. } => {
            schema::append_schema_member(basis, &prefix, member, revised)
        }
        ApplicationSchemaMember::ApplicationQuery { .. }
        | ApplicationSchemaMember::ApplicationCapability { .. }
        | ApplicationSchemaMember::ApplicationCapabilityContext { .. }
        | ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot { .. }
        | ApplicationSchemaMember::ApplicationCapabilityProvenance { .. } => {
            capability::append_capability_member(basis, &prefix, member)
        }
        ApplicationSchemaMember::Operation { .. }
        | ApplicationSchemaMember::OperationProgram { .. }
        | ApplicationSchemaMember::OperationDecisionRead { .. }
        | ApplicationSchemaMember::OperationMutationPrecondition { .. }
        | ApplicationSchemaMember::OperationDecisionFactBudget { .. }
        | ApplicationSchemaMember::OperationProjectionWorkBudget { .. }
        | ApplicationSchemaMember::OperationExternalEffect { .. }
        | ApplicationSchemaMember::OperationAftermath { .. } => {
            operation::append_operation_member(basis, &prefix, member)
        }
        ApplicationSchemaMember::Policy { .. }
        | ApplicationSchemaMember::Ability { .. }
        | ApplicationSchemaMember::OperationAbility { .. }
        | ApplicationSchemaMember::AbilityPolicy { .. } => {
            authorization::append_authorization_member(basis, &prefix, member)
        }
        ApplicationSchemaMember::Unit { .. } | ApplicationSchemaMember::Effect { .. } => {
            vocabulary::append_vocabulary_member(basis, &prefix, member)
        }
    }
}
