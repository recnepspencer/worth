//! Ordinary continuity shares accepted root DATA; only declared transitions compile.
use super::inputs::{SchemaContinuityInputs, SchemaContinuityRequest};
use super::*;
use contracts::transactions::data::outcomes::commit_preparation::CommitPreparationErrorOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
pub fn resolve_schema_continuity<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<SchemaContinuityInputs<'a>>,
    branch_id: &contracts::history::data::coordinates::BranchId,
    options: impl Into<SchemaContinuityRequest<'a>>,
) -> Result<SchemaContinuityPlan, TransactionCommitError> {
    let inputs = inputs.into();
    let options = options.into();
    let descriptor_policy = inputs.schema.descriptor_semantics_policy.clone();
    let authority_input = match (options.copied_authority, options.transition) {
        (Some(input), _) => input.clone(),
        (None, Some(_)) => {
            worth_relational_branch::internal_port::schema::continuity::authority_input::SchemaContinuityAuthorityInput::copy_registry_for_request(
                &inputs.schema.registry,
                inputs
                    .schema
                    .descriptor_semantics_policy
                    .current_write_version(),
                inputs
                    .schema
                    .descriptor_canonical_basis_policy
                    .current_write_version(),
                execution,
            )
            .map_err(|cause| request_denial(cause, options.basis))?
        }
        (None, None) => options.authority.continuity_input(
            inputs
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
    let previous_root = inputs
        .history
        .branch_cell(branch_id)
        .and_then(|cell| cell.root());
    let previous_envelope = previous_root
        .as_deref()
        .and_then(SchemaContinuityPrior::from_root);
    let Some(previous_envelope) = previous_envelope.as_ref() else {
        if let Some(proposed_transition) = options.transition {
            return materialize_declared_transition(
                inputs.diagnostics,
                proposed_transition.clone(),
                options.reconciliation.cloned(),
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
        inputs
            .diagnostics
            .instrumentation
            .count_descriptor_version_mismatch();
        return Err(schema_continuity_conflict(
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
    match options.transition {
        Some(proposed_transition) => materialize_declared_transition(
            inputs.diagnostics,
            proposed_transition.clone(),
            options.reconciliation.cloned(),
            current_descriptor_semantics_version,
            current_descriptor_canonical_basis_version,
            branch_id,
            Some(previous_envelope),
            authority_input.target_schema_basis(),
            current_schema_version,
            &authority_input,
        ),
        None if drift_detected => Err(schema_continuity_conflict(
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
    cause: contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial,
    basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
) -> TransactionCommitError {
    use crate::mvcc::publication::PublicationPreparationDenial as Denial;
    use contracts::transactions::data::outcomes::commit_preparation::CommitPreparationError;
    use contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial;
    use contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind;
    if cause == contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial::AllocationLayoutOverflow {
        return TransactionCommitError::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(
                contracts::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow,
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
            TransactionCommitError::publication(contracts::publication::data::publication_error::PublicationError::new(
                contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly,
                detail,
            ))
        }
    }
}
