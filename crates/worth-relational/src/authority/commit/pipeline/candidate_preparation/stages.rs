//! Consume one candidate phase before entering its successor's call frame.
use super::super::publication_execution::PublicationPhaseOwners;
use super::super::{artifact_execution, publication_execution, snapshot_validation};
use super::registration::CandidateRegistration;
use super::registration::CandidateRegistrationInputs;
use super::{CandidateArtifactSelection, SelectedCandidateSymbols};
use super::{CommitPhaseBacking, HistoryPreparedCandidate};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

struct CandidateContext {
    registration: CandidateRegistration,
    diagnostic: crate::publication::diagnostic_store::RelationalDiagnosticOperationCapture,
}
struct PublicationInputs {
    publication: Option<
        crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
    >,
    materialization:
        Option<std::sync::Arc<crate::history::data::CanonicalRequiredOutputMaterialization>>,
    caller: Option<crate::history::data::RelationalCallerAssociation>,
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
    registration: CandidateRegistration,
    prepared: CommitPhaseBacking<super::super::PreparedCommitPublicationExecution>,
}
#[inline(never)]
pub(super) fn validate(
    request: &ExecutionRequest<'_, '_>,
    history: CommitPhaseBacking<HistoryPreparedCandidate<'_, '_>>,
) -> Result<
    (
        SelectedCandidateSymbols,
        CommitPhaseBacking<ValidatedCandidate>,
    ),
    TransactionCommitError,
> {
    let HistoryPreparedCandidate {
        scope_control: _scope_control,
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
    let pending = history_bound.value().candidate_symbols();
    let symbols = SelectedCandidateSymbols {
        dictionary: pending.binding(),
        references: pending.required_references().clone(),
    };
    let validated = snapshot_validation::validate_snapshot_publication(request, history_bound)?;
    let validated = CommitPhaseBacking::new(
        request,
        ValidatedCandidate {
            context: CandidateContext {
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
    )?;
    Ok((symbols, validated))
}
#[inline(never)]
pub(super) fn assemble(
    request: &ExecutionRequest<'_, '_>,
    inputs_for_assembly: CandidateArtifactSelection<'_>,
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
        inputs_for_assembly.owners,
        inputs_for_assembly.max_patch_records,
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
pub(super) fn publish<Attach: FnOnce(&crate::branch::root::PreparedRelationalBranchRootCapture)>(
    request: &ExecutionRequest<'_, '_>,
    owners: PublicationPhaseOwners<'_, Attach>,
    source: CommitPhaseBacking<AssembledCandidate>,
) -> Result<CommitPhaseBacking<PublishedCandidate>, TransactionCommitError> {
    let AssembledCandidate {
        context,
        inputs,
        assembled,
    } = source.into_inner();
    let mut prepared = publication_execution::prepare_commit_publication_execution(
        request,
        owners,
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
            registration: context.registration,
            prepared,
        },
    )
}
#[inline(never)]
pub(super) fn register(
    request: &ExecutionRequest<'_, '_>,
    inputs: CandidateRegistrationInputs<'_>,
    source: CommitPhaseBacking<PublishedCandidate>,
) -> Result<crate::mvcc::publication::PreparedRelationalCommitCandidate, TransactionCommitError> {
    let PublishedCandidate {
        registration,
        prepared,
    } = source.into_inner();
    registration.register(request, inputs, prepared.into_inner())
}
