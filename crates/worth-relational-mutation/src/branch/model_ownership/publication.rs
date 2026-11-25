//! Final installed exclusivity includes the independent prospective custody facts.
use super::*;
use crate::contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::contracts::validation::data::custom_rule::refusal::CustomInvariantRefusal;
use crate::contracts::validation::data::results::InvariantViolationFields;
use crate::validation::custom_registry::FrozenCustomInvariantRegistry;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::model_ownership::traversal::Read;
use worth_relational_branch::internal_port::branch::model_ownership::{basis, traversal};
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyOwnerPort as _,
};

pub fn validate_candidate_ownership(
    execution: &ExecutionRequest<'_, '_>,
    invariants: &FrozenCustomInvariantRegistry,
    root: &RelationalBranchRoot,
    candidate: &CanonicalCommitEnvelope,
) -> Result<(), TransactionCommitError> {
    if (*root
        .schema_authority()
        .registry()
        .owner_current_publication_storage())
    .is_none()
    {
        return Ok(());
    }
    let Some((registration, ownership)) = invariants.model_ownership() else {
        return Ok(());
    };
    execution.run_native_with_checkpoint(|context| {
            crate::contracts::execution::request_stage::checkpoint(context, 1).map_err(request)?;
            let result = basis::read_candidate(execution, root, context, |read| {
                // Explicit patch targets include both published members and their
                // source. Relation moves reach the affected descendants as well.
                for target in candidate.authored_intent_targets().flatten().chain(
                    candidate.patch.authoritative_record_patches.iter().map(|patch| patch.target.clone())) {
                    read.charge(1)?;
                    let subject = match target {
                        RecordRef::Entity(id) => Some(id),
                        RecordRef::Relation(id) => read.relation(id)?.map(|edge| edge.target),
                    };
                    let Some(subject) = subject else { continue; };
                    if read.kind(subject)?.is_none() { continue; }
                    for domain in &ownership.plan.domains {
                        for entity in traversal::walk(read, subject, true, &domain.relations)? {
                            ownership.plan.owner(read, entity)?;
                        }
                    }
                }
                Ok(())
            });
            match result {
                Ok(()) => {},
                Err(RelationalModelOwnershipDenial::Conflict(conflict)) => {
                    let _memory = execution.reserve_retained_memory(256).map_err(request)?;
                    let bytes = conflict.encode().map_err(|_| crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("ownership refusal cannot encode"))?;
                    return Err(TransactionCommitError::conflict(CommitConflict::new(ConflictClass::InvariantViolation {
                        code: crate::contracts::diagnostics::data::artifacts::DiagnosticCode::InvariantViolation,
                        detail: "model ownership has two document roots in the completed publication candidate".into(),
                        fields: InvariantViolationFields::CustomInvariantViolation {
                            identity: registration.descriptor().identity.clone(),
                            refusal: CustomInvariantRefusal::DeclaredValue {
                                value_identity: RelationalModelOwnershipConflict::VALUE_IDENTITY.into(), bytes,
                            },
                        },
                    })));
                },
                Err(RelationalModelOwnershipDenial::Custody(cause)) => return Err(cause),
                Err(RelationalModelOwnershipDenial::Request(cause)) => return Err(request(cause)),
                Err(RelationalModelOwnershipDenial::Native(cause)) => return Err(request(cause)),
                Err(cause) => return Err(crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(format!("completed ownership observation: {cause:?}"))),
            }
        Ok(())
    }).map_err(request)?.0
}
fn request(
    cause: impl Into<crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    crate::contracts::publication::ownership::read_request::request(cause.into())
}
