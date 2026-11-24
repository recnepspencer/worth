//! Named safe points of one borrowed commit request; no stage fee is charged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitStage {
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

pub fn consult_stage(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    stage: CommitStage,
) -> Result<(), worth_execution::WorkCeilingDenial> {
    observation::visit(stage);
    execution.consult()
}

#[doc(hidden)]
pub mod observation;
