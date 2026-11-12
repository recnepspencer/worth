//! Original-effect continuation never returns a fabricated live Performed value.
use crate::{history::WorldReadmittedPublication, recovery::WorldRecoveredUnpublishedEffects};
mod progress;
mod retention;
pub use progress::WorldHomeTransitionEffectProgress;

#[derive(Debug)]
pub enum WorldHomeTransitionEffectDenial {
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
    Plan(worth_foundational::facade::HomeProgramTransitionDescriptionDenial),
    Source(crate::publication::NoEffectCause),
    Recovery(crate::recovery::RuntimeWorldRecoveryDenial),
    Medium(crate::durability::WorldDurableMediumDenial),
    Readmission(crate::durability::WorldReadmissionDenial),
    History(crate::history::CompositeHistoryCatalogDenial),
    NativeEffectRequired,
    IdentityScope,
    NativeEffectMismatch,
    ReleaseInProgress,
    RetentionCapacity,
    RetentionScope,
    NativeRetention(worth_relational::facade::branch::RelationalBranchBasisDenial),
    SignalRetention(worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial),
    RetentionOwnerFailure,
    WorldHeadUnavailable,
}

#[derive(Debug)]
pub enum WorldHomeTransitionEffectOutcome {
    Continued(WorldReadmittedPublication),
    /// The same owner has more work to advance. This includes an acknowledged
    /// occurrence awaiting materialization; it is not an unknown-fate proof.
    Pending(WorldRecoveredUnpublishedEffects),
    EvidenceRequired(WorldRecoveredUnpublishedEffects),
    Indeterminate(WorldRecoveredUnpublishedEffects),
    ReopenRequired(WorldRecoveredUnpublishedEffects),
    Refused {
        cause: WorldHomeTransitionEffectDenial,
        custody: WorldRecoveredUnpublishedEffects,
    },
}
