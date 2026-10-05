mod cursor;
mod denial;
mod plan;
mod record;
#[cfg(test)]
pub(crate) mod terminal_head_retirement_fixture;

pub use cursor::{
    HistoricalReleasedDropTargetWitness, HistoricalRetiredTargetWitness, RecoveryPageObservation,
    RecoveryPageSource,
};
pub use denial::{PhysicalRedoPlanningDenial, PhysicalRedoProjectionLimit};
#[cfg(any(test, feature = "test-support"))]
pub use plan::head_replay_limit_for_test;
pub use plan::{
    admit_current_source_copy_publication, admit_physical_redo_members,
    physical_redo_observation_target_identities, physical_redo_observation_targets,
    physical_redo_target_identities, plan_physical_redo, AdmittedPhysicalRedoMembers,
    AdmittedRootStepMemberView, ExceededHeadReplayBound, HeadReplayBound,
    HistoricalConsumedOperationSet, HistoricalRetirements, ImmutablePhysicalRedoPlan,
    PhysicalExtentCopyAdmission, PhysicalRedoAdmissionLimits, PhysicalRedoDecision,
    PhysicalRedoDecisionKind, PhysicalRedoDecisionPrior, PhysicalRedoDecisionView,
    PhysicalRedoGroupBinding, PhysicalRedoMemberInput, PhysicalRedoPlanCounters,
    PhysicalRedoProjection, PhysicalRewriteAdmission, SelectedReleaseHeadReplayDenial,
    VerifiedOrderedReleasedHeadReplayV14, VerifiedSelectedReleaseHeadReplayV14,
    VerifiedSelectedTerminalHeadRetirementReplay,
};
pub use record::{
    decode_physical_redo_records, PhysicalRedoExtentCoordinate, PhysicalRedoRecord,
    PhysicalRedoTarget, PhysicalRedoTargetIdentity,
};
