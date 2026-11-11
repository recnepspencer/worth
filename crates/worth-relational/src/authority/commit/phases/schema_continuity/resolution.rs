//! Ordinary continuity shares accepted root DATA; only declared transitions compile.
use super::*;
pub(crate) fn resolve_schema_continuity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    branch_id: &crate::history::data::BranchId,
    options: &crate::mvcc::RelationalTransactionValidationInput,
) -> Result<SchemaContinuityPlan, TransactionCommitError> {
    let descriptor_policy = runtime.config.schema.descriptor_semantics_policy.clone();
    let authority_input = match (
        options.schema_authority_input(),
        options.proposed_schema_transition(),
    ) {
        (Some(input), _) => input.clone(),
        (None, Some(_)) => {
            crate::schema::SchemaContinuityAuthorityInput::copy_registry_for_request(
                &runtime.config.schema.registry,
                runtime
                    .config
                    .schema
                    .descriptor_semantics_policy
                    .current_write_version(),
                runtime
                    .config
                    .schema
                    .descriptor_canonical_basis_policy
                    .current_write_version(),
                execution,
            )
            .map_err(|cause| request_denial(cause, options.basis()))?
        }
        (None, None) => options.schema_authority().continuity_input(
            runtime
                .config
                .schema
                .descriptor_canonical_basis_policy
                .current_write_version(),
        ),
    };
    let current_descriptor_semantics_version = authority_input.descriptor_semantics_version();
    let current_descriptor_canonical_basis_version =
        authority_input.descriptor_canonical_basis_version();
    let current_schema_version = authority_input.target_schema_version();
    let current_schema_authority = authority_input.target_schema_authority().clone();
    let previous_root = runtime
        .history
        .branch_cell(branch_id)
        .and_then(|cell| cell.root());
    let previous_envelope = previous_root
        .as_deref()
        .and_then(SchemaContinuityPrior::from_root);
    let Some(previous_envelope) = previous_envelope.as_ref() else {
        if let Some(proposed_transition) = options.proposed_schema_transition() {
            return materialize_declared_transition(
                runtime,
                proposed_transition.clone(),
                options.schema_reconciliation_policy().cloned(),
                current_descriptor_semantics_version,
                current_descriptor_canonical_basis_version,
                branch_id,
                None,
                authority_input.target_schema_basis(),
                current_schema_version,
                &authority_input,
            );
        }
        return Ok(SchemaContinuityPlan::current(&authority_input));
    };
    if !descriptor_policy.supports(previous_envelope.descriptor_semantics_version) {
        runtime
            .performance_access()
            .count_descriptor_version_mismatch();
        return Err(schema_continuity_conflict(
            runtime,
            branch_id,
            options.proposed_schema_transition(),
            Some(previous_envelope),
            ConflictClass::DescriptorSemanticsVersionUnsupported {
                previous_descriptor_semantics_version: previous_envelope
                    .descriptor_semantics_version,
                current_descriptor_semantics_version,
            },
        ));
    }

    let drift_detected = previous_envelope.schema_version != current_schema_version
        || !previous_envelope
            .authority
            .matches(&current_schema_authority);
    match options.proposed_schema_transition() {
        Some(proposed_transition) => materialize_declared_transition(
            runtime,
            proposed_transition.clone(),
            options.schema_reconciliation_policy().cloned(),
            current_descriptor_semantics_version,
            current_descriptor_canonical_basis_version,
            branch_id,
            Some(previous_envelope),
            authority_input.target_schema_basis(),
            current_schema_version,
            &authority_input,
        ),
        None if drift_detected => Err(schema_continuity_conflict(
            runtime,
            branch_id,
            None,
            Some(previous_envelope),
            ConflictClass::UndeclaredSchemaTransition {
                previous_schema_version: previous_envelope.schema_version,
                current_schema_version,
                previous_descriptor_semantics_version: previous_envelope
                    .descriptor_semantics_version,
                current_descriptor_semantics_version,
            },
        )),
        None => Ok(SchemaContinuityPlan::current(&authority_input)),
    }
}

fn request_denial(
    cause: crate::branch::RelationalBranchRootCaptureDenial,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
) -> TransactionCommitError {
    use crate::mvcc::PublicationPreparationDenial as Denial;
    use crate::transactions::data::{
        CommitExecutionDenial, CommitExecutionDenialKind, CommitPreparationError,
    };
    if cause == crate::branch::RelationalBranchRootCaptureDenial::AllocationLayoutOverflow {
        return TransactionCommitError::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(
                crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
            ),
            partition_identity: None,
        });
    }
    match Denial::from(cause) {
        Denial::Execution(cause) => TransactionCommitError::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        }),
        Denial::StorageUnavailable(failure) => {
            TransactionCommitError::preparation(CommitPreparationError::storage_unavailable(
                basis.identity().branch_id().clone(),
                basis.observation().commit_id().map(|id| id.0),
                basis.observation().version_id(),
                failure,
            ))
        }
        Denial::Invalid(detail) => {
            TransactionCommitError::publication(crate::publication::data::PublicationError::new(
                crate::publication::bundle::PublicationStage::BundleAssembly,
                detail,
            ))
        }
    }
}
