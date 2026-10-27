use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::ApplicationSchemaMember;

mod aftermath;
mod authorization;
mod capability;
mod field;
mod invariant;
mod mutation_description;
mod operation;
mod principal_binding;
mod relation_aspect;
pub(super) use relation_aspect::append_relation_field;
mod schema;
mod vocabulary;

pub(super) fn append_member(
    basis: &mut ApplicationSchemaCanonicalBasis,
    index: usize,
    member: &ApplicationSchemaMember,
) {
    let prefix = format!("member[{index}]");
    match member {
        ApplicationSchemaMember::RelationAspect { definition } => {
            relation_aspect::append(basis, &prefix, definition)
        }
        ApplicationSchemaMember::ApplicationMutation { description } => {
            mutation_description::append(basis, &prefix, description)
        }
        ApplicationSchemaMember::ApplicationInvariant { .. } => {
            invariant::append(basis, &prefix, member)
        }
        ApplicationSchemaMember::Entity { .. }
        | ApplicationSchemaMember::Aspect { .. }
        | ApplicationSchemaMember::Field { .. }
        | ApplicationSchemaMember::Relation { .. }
        | ApplicationSchemaMember::PrincipalBinding { .. } => {
            schema::append_schema_member(basis, &prefix, member)
        }
        ApplicationSchemaMember::ApplicationQuery { .. }
        | ApplicationSchemaMember::ApplicationHistoryQuery { .. }
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
        | ApplicationSchemaMember::OperationProjectionWorkBudget { .. }
        | ApplicationSchemaMember::OperationExternalEffect { .. }
        | ApplicationSchemaMember::OperationInboundOccurrence { .. }
        | ApplicationSchemaMember::OperationAftermath { .. } => {
            operation::append_operation_member(basis, &prefix, member)
        }
        ApplicationSchemaMember::OperationBody { operation, body } => {
            operation::append_body(basis, &prefix, operation, body);
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
