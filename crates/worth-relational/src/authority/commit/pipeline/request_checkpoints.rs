//! Named safe points of one borrowed commit request; no stage fee is charged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommitStage {
    /// A detached transaction read consults the same original request court.
    TransactionRead,
    /// Explicit preview admission, before dictionary or plan preparation.
    PlanPreview,
    /// Each fresh residency candidate consults before it spends identities.
    PreparationRound,
    Validation,
    Admission,
    Draft,
    Boundary,
    Mutation,
    History,
    Snapshot,
    Artifact,
    PublicationPreparation,
    CandidateRegistration,
    Publication,
    /// The final pre-effect consult, after any companion preflight, where the
    /// caller's original request keeps its first cause before the head moves.
    PublicationCutover,
}

pub(crate) fn consult_stage(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    _stage: CommitStage,
) -> Result<(), worth_execution::WorkCeilingDenial> {
    consult_stage_control(execution, _stage)
}

fn consult_stage_control(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    _stage: CommitStage,
) -> Result<(), worth_execution::WorkCeilingDenial> {
    #[cfg(test)]
    observation::visit(_stage);
    execution.consult()
}

#[cfg(test)]
pub(crate) mod observation;
