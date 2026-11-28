//! Consumed draft, boundary and mutation phases do not occupy the next frame.
use super::authority_context::AuthoritativeCommitContext;
use super::draft_execution::PreparedCommitExecution;
use super::execution_admission::CommitExecutionAdmission;
use super::history_binding::HistoryBoundCommitExecution;
use super::phase_backing::CommitPhaseBacking;
use super::publication_validation::DraftValidationInput;
use crate::authority::commit::phases::schema_continuity::inputs::SchemaContinuityInputs;
use worth_relational_contracts::internal_port::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;
pub mod stages;

pub struct HistoryPreparationInputs<'a> {
    pub admission: CommitExecutionAdmission<'a>,
    pub continuity: SchemaContinuityInputs<'a>,
    pub descriptor_semantics: DescriptorSemanticsVersion,
}

#[inline(never)]
pub(super) fn prepare_bound_history<V: DraftValidationInput>(
    execution: &ExecutionRequest<'_, '_>,
    inputs: HistoryPreparationInputs<'_>,
    context: AuthoritativeCommitContext<V>,
) -> Result<CommitPhaseBacking<HistoryBoundCommitExecution>, TransactionCommitError> {
    let instrumentation = inputs.admission.instrumentation;
    let HistoryPreparationInputs {
        admission,
        continuity,
        descriptor_semantics,
    } = inputs;
    let prepared = prepare_draft(execution, admission, descriptor_semantics, context)?;
    let boundary = stages::validate_boundary(execution, prepared)?;
    let mutated = stages::mutate(execution, instrumentation, boundary)?;
    stages::bind(execution, continuity, mutated)
}

#[inline(never)]
fn prepare_draft<V: DraftValidationInput>(
    execution: &ExecutionRequest<'_, '_>,
    admission: CommitExecutionAdmission<'_>,
    descriptor_semantics: DescriptorSemanticsVersion,
    context: AuthoritativeCommitContext<V>,
) -> Result<CommitPhaseBacking<PreparedCommitExecution>, TransactionCommitError> {
    let instrumentation = admission.instrumentation;
    let admitted = super::execution_admission::admit_commit_execution(admission, context)?;
    super::draft_execution::prepare_commit_execution(
        execution,
        instrumentation,
        descriptor_semantics,
        admitted,
    )
}
