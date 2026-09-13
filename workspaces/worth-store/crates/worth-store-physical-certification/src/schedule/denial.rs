use worth_foundational::canonicalization_api::lower_lane::basis::CanonicalBasisConstructionDenial;
use worth_foundational::canonicalization_api::lower_lane::digest::CanonicalDigestDerivationDenial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleReplayDenial {
    MissingSeed,
    EmptyPerturbationDecision,
    EmptyPerturbationTrace,
    DuplicatePerturbationDecisionFamily,
    MissingActorId,
    EmptyActorStepSchedule,
    NonCanonicalActorStepOrder,
    DuplicateActorStepActorId,
    WallClockScheduleDenied,
    UnorderedMapScheduleDenied,
    AmbientThreadScheduleDenied,
    UnboundedExplorationDenied,
    EmptyStateSpaceBudget,
    StateSpaceBudgetExceeded {
        required_steps: u32,
        max_steps: u32,
    },
    ShrinkErasedFaultLocus {
        actor_id: String,
        yieldpoint: String,
    },
    ShrinkInputDoesNotReproduceFailure,
    ScheduleCanonicalBasisDenied(CanonicalBasisConstructionDenial),
    ScheduleDigestDerivationDenied(CanonicalDigestDerivationDenial),
}
