//! Final installed exclusivity includes the independent prospective custody facts.
use super::{
    basis,
    traversal::{self, Read},
    *,
};
use crate::{
    branch::RelationalBranchRoot,
    history::data::CanonicalCommitEnvelope,
    runtime::RelationalPreparationRuntime,
    transactions::data::{CommitConflict, ConflictClass, RecordRef, TransactionCommitError},
    validation::data::{CustomInvariantRefusal, InvariantViolationFields},
};
use worth_execution::ExecutionRequest;

pub(crate) fn validate_candidate_ownership(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    root: &RelationalBranchRoot,
    candidate: &CanonicalCommitEnvelope,
) -> Result<(), TransactionCommitError> {
    if root
        .schema_authority()
        .registry()
        .current_publication_storage
        .is_none()
    {
        return Ok(());
    }
    let Some((registration, ownership)) = runtime
        .schema_contract_runtime
        .custom_invariant_registries
        .model_ownership()
    else {
        return Ok(());
    };
    execution.run_native_with_checkpoint(|context| {
            crate::execution::request_stage::checkpoint(context, 1).map_err(request)?;
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
                    for domain in &ownership.domains {
                        for entity in traversal::walk(read, subject, true, &domain.relations)? {
                            ownership.owner(read, entity)?;
                        }
                    }
                }
                Ok(())
            });
            match result {
                Ok(()) => {},
                Err(RelationalModelOwnershipDenial::Conflict(conflict)) => {
                    let _memory = execution.reserve_retained_memory(256).map_err(request)?;
                    let bytes = conflict.encode().map_err(|_| crate::mvcc::origin_denial("ownership refusal cannot encode"))?;
                    return Err(TransactionCommitError::conflict(CommitConflict::new(ConflictClass::InvariantViolation {
                        code: crate::diagnostics::data::DiagnosticCode::InvariantViolation,
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
                Err(cause) => return Err(crate::mvcc::origin_denial(format!("completed ownership observation: {cause:?}"))),
            }
        Ok(())
    }).map_err(request)?.0
}
fn request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    crate::publication::ownership::request(cause.into())
}
