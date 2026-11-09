//! Consume one candidate phase before entering its successor's call frame.
use super::super::{artifact_execution, publication_execution, snapshot_validation};
use super::registration::CandidateRegistration;
use super::{CommitPhaseBacking, HistoryPreparedCandidate};
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;
use worth_execution::ExecutionRequest;

struct CandidateContext {
    runtime: RelationalPreparationRuntime,
    registration: CandidateRegistration,
    diagnostic: crate::runtime::RelationalDiagnosticOperationCapture,
}
struct PublicationInputs {
    publication: Option<crate::mvcc::RequiredOutputPublicationAdmission>,
    materialization:
        Option<std::sync::Arc<crate::history::data::CanonicalRequiredOutputMaterialization>>,
    caller: Option<crate::mvcc::RelationalCallerAssociation>,
}
pub(super) struct ValidatedCandidate {
    context: CandidateContext,
    inputs: PublicationInputs,
    validated: CommitPhaseBacking<snapshot_validation::SnapshotValidatedCommitExecution>,
}
pub(super) struct AssembledCandidate {
    context: CandidateContext,
    inputs: PublicationInputs,
    assembled: CommitPhaseBacking<artifact_execution::AssembledCommitExecution>,
}
pub(super) struct PublishedCandidate {
    runtime: RelationalPreparationRuntime,
    registration: CandidateRegistration,
    prepared: CommitPhaseBacking<super::super::PreparedCommitPublicationExecution>,
}
#[inline(never)]
pub(super) fn validate(
    request: &ExecutionRequest<'_, '_>,
    history: CommitPhaseBacking<HistoryPreparedCandidate<'_, '_>>,
) -> Result<CommitPhaseBacking<ValidatedCandidate>, TransactionCommitError> {
    let HistoryPreparedCandidate {
        recovered_runtime,
        scope_control,
        history_bound,
        runtime_instance_id,
        publication_binding,
        transaction_id,
        branch_id,
        control,
        retention_binding,
        decision_basis,
        coupled_proposal,
        expected_basis,
        expected_root,
        publication_cell,
        required_output_publication,
        required_output_materialization,
        caller_association,
        diagnostic_capture,
    } = history.into_inner();
    let runtime =
        recovered_runtime.with_candidate_symbols(history_bound.value().candidate_symbols());
    let validated = snapshot_validation::validate_snapshot_publication(
        request,
        &runtime,
        history_bound,
        &scope_control,
    )?;
    CommitPhaseBacking::new(
        request,
        ValidatedCandidate {
            context: CandidateContext {
                runtime,
                registration: CandidateRegistration {
                    runtime_instance_id,
                    publication_binding,
                    transaction_id,
                    branch_id,
                    control,
                    retention_binding,
                    decision_basis,
                    coupled_proposal,
                    expected_basis,
                    expected_root,
                    publication_cell,
                },
                diagnostic: diagnostic_capture,
            },
            inputs: PublicationInputs {
                publication: required_output_publication,
                materialization: required_output_materialization,
                caller: caller_association,
            },
            validated,
        },
    )
}
#[inline(never)]
pub(super) fn assemble(
    request: &ExecutionRequest<'_, '_>,
    checked: CommitPhaseBacking<ValidatedCandidate>,
) -> Result<CommitPhaseBacking<AssembledCandidate>, TransactionCommitError> {
    let ValidatedCandidate {
        context,
        inputs,
        validated,
    } = checked.into_inner();
    let lineage_deferred = (inputs.publication.is_some()
        || validated.value().has_codec_publication())
        && context
            .registration
            .expected_root
            .schema_authority()
            .registry()
            .current_publication_storage
            .is_some();
    let assembled = artifact_execution::assemble_commit_artifacts(
        request,
        &context.runtime,
        validated,
        lineage_deferred,
    )?;
    CommitPhaseBacking::new(
        request,
        AssembledCandidate {
            context,
            inputs,
            assembled,
        },
    )
}
#[inline(never)]
pub(super) fn publish(
    request: &ExecutionRequest<'_, '_>,
    source: CommitPhaseBacking<AssembledCandidate>,
) -> Result<CommitPhaseBacking<PublishedCandidate>, TransactionCommitError> {
    let AssembledCandidate {
        context,
        inputs,
        assembled,
    } = source.into_inner();
    let mut prepared = publication_execution::prepare_commit_publication_execution(
        request,
        &context.runtime,
        assembled,
        inputs.publication,
        inputs.materialization,
        inputs.caller,
    )?;
    prepared
        .value_mut()
        .append_diagnostics(context.diagnostic.finish());
    CommitPhaseBacking::new(
        request,
        PublishedCandidate {
            runtime: context.runtime,
            registration: context.registration,
            prepared,
        },
    )
}
#[inline(never)]
pub(super) fn register(
    request: &ExecutionRequest<'_, '_>,
    source: CommitPhaseBacking<PublishedCandidate>,
) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
    let PublishedCandidate {
        runtime,
        registration,
        prepared,
    } = source.into_inner();
    registration.register(request, &runtime, prepared.into_inner())
}
