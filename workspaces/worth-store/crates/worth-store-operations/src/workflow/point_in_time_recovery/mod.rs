mod candidate_selection;
mod exact_frontier;
mod execution;
mod intent;
mod lowering;
mod owner_receipt;
mod recovery;

#[cfg(test)]
mod tests;

pub use candidate_selection::{
    PitrCandidatePosture, PitrCandidateSelectionDenial, PitrRoundingPolicy, PointInTimeCandidate,
    PointInTimeCandidateSet,
};

pub use exact_frontier::{
    ExactRecoveryFrontier, FrontierPartialOrder, RecoveryTimelineAdmission,
    RecoveryTimelineObservation, RecoveryTimelineOwner,
};
pub use execution::{
    ExecutedPointInTimeRecovery, ExecutionReadyPointInTimeRecovery, PitrExecutionDenial,
    PitrReadinessDenial, PointInTimeRecoveryOperationReceipt,
};
pub use intent::{
    AdmittedPitrSourceOperation, EvidenceBoundPointInTimeRecoveryPlan, PitrResolutionDenial,
    PitrSourceAdmissionDenial, PointInTimeRecoveryIntent, ResolvedPitrCandidate,
};
pub use lowering::{
    AuthorizedPointInTimeRecoveryPlan, LoweredPointInTimeRecoveryPlan, PitrLoweringDenial,
};
pub(crate) use owner_receipt::pitr_owner_receipt_identity;
pub use recovery::{
    PointInTimeRecoveryReceipt, PointInTimeReplayDenial, PointInTimeReplayOwner,
    PointInTimeReplayPlan, PointInTimeReplayRequest, PointInTimeReplaySourceCoordinates,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PointInTimeRecoveryOperation;
