//! Consumed draft, boundary and mutation phases do not occupy the next frame.
use super::authority_context::AuthoritativeCommitContext;
use super::draft_execution::PreparedCommitExecution;
use super::execution_admission::{AdmittedCommitState, CommitExecutionAdmission};
use super::history_binding::HistoryBoundCommitExecution;
use super::merge_draft::AcceptedMergeDraft;
use super::phase_backing::CommitPhaseBacking;
use super::publication_validation::DraftValidationInput;
use crate::authority::commit::phases::schema_continuity::inputs::SchemaContinuityInputs;
use crate::schema::data::DescriptorSemanticsVersion;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;
mod stages;

pub(crate) struct HistoryPreparationInputs<'a, Draft> {
    pub(crate) admission: CommitExecutionAdmission<'a>,
    pub(crate) continuity: SchemaContinuityInputs<'a>,
    pub(crate) descriptor_semantics: DescriptorSemanticsVersion,
    pub(crate) merge_draft: Draft,
}

#[inline(never)]
pub(super) fn prepare_bound_history<V: DraftValidationInput, Draft>(
    execution: &ExecutionRequest<'_, '_>,
    inputs: HistoryPreparationInputs<'_, Draft>,
    context: AuthoritativeCommitContext<V>,
) -> Result<CommitPhaseBacking<HistoryBoundCommitExecution>, TransactionCommitError>
where
    Draft: FnOnce(
        &ExecutionRequest<'_, '_>,
        &AdmittedCommitState<V>,
    ) -> Result<AcceptedMergeDraft, TransactionCommitError>,
{
    let instrumentation = inputs.admission.instrumentation;
    let HistoryPreparationInputs {
        admission,
        continuity,
        descriptor_semantics,
        merge_draft,
    } = inputs;
    let prepared = prepare_draft(
        execution,
        admission,
        descriptor_semantics,
        merge_draft,
        context,
    )?;
    let boundary = stages::validate_boundary(execution, prepared)?;
    let mutated = stages::mutate(execution, instrumentation, boundary)?;
    stages::bind(execution, continuity, mutated)
}

#[inline(never)]
fn prepare_draft<V: DraftValidationInput, Draft>(
    execution: &ExecutionRequest<'_, '_>,
    admission: CommitExecutionAdmission<'_>,
    descriptor_semantics: DescriptorSemanticsVersion,
    merge_draft: Draft,
    context: AuthoritativeCommitContext<V>,
) -> Result<CommitPhaseBacking<PreparedCommitExecution>, TransactionCommitError>
where
    Draft: FnOnce(
        &ExecutionRequest<'_, '_>,
        &AdmittedCommitState<V>,
    ) -> Result<AcceptedMergeDraft, TransactionCommitError>,
{
    let instrumentation = admission.instrumentation;
    let admitted = super::execution_admission::admit_commit_execution(admission, context)?;
    super::draft_execution::prepare_commit_execution(
        execution,
        instrumentation,
        descriptor_semantics,
        merge_draft,
        admitted,
    )
}
