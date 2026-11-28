/// Truthful denial from the owning recovery catalog or its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldRecoveryDenial {
    HomeProgramTransitionRequired,
    RecoveredCustodyRequired,
    ContinuationAlreadyCommitted,
    ContinuationAlreadyStarted,
    ReleaseAlreadyStarted,
    ReopenRequired,
    OwnerUnavailable(crate::lifecycle::RuntimeWorldOwnerUnavailable),
    ForeignHandle,
    StaleIncarnation,
    MissingRecord,
    Busy,
    TooYoung,
    ClockRegressed,
    CallerCapabilityLive,
    SettlementRequired,
    SettlementEvidenceUnavailable,
    OriginalSourceUnavailable,
    OriginalSourceMismatch,
    OutputCapacityExhausted,
}
impl From<crate::lifecycle::RuntimeWorldOwnerUnavailable> for RuntimeWorldRecoveryDenial {
    fn from(value: crate::lifecycle::RuntimeWorldOwnerUnavailable) -> Self {
        Self::OwnerUnavailable(value)
    }
}

use worth_execution::report::Disposition::*;
use worth_execution::report::RetryHint::{AfterBackoff, WithFreshAuthority};

crate::denial_report::world_denial_report!(RuntimeWorldRecoveryDenial, "recovery", {
    Self::HomeProgramTransitionRequired => ("home_program_transition_required", Authorization, Reauthorize, "Recovery requires an admitted Home program transition."),
    Self::RecoveredCustodyRequired => ("recovered_custody_required", Authorization, Reauthorize, "Recovery requires the original recovered custody."),
    Self::ContinuationAlreadyCommitted => ("continuation_already_committed", Conflict, Final, "The recovery continuation is already committed."),
    Self::ContinuationAlreadyStarted => ("continuation_already_started", Conflict, Final, "The recovery continuation is already in progress."),
    Self::ReleaseAlreadyStarted => ("release_already_started", Conflict, Final, "The recovery release is already in progress."),
    Self::ReopenRequired => ("reopen_required", Unavailable, Retry(WithFreshAuthority), "Reopen the Home before continuing recovery."),
    Self::OwnerUnavailable(_) => ("owner_unavailable", Unavailable, Retry(WithFreshAuthority), "The World owner is unavailable."),
    Self::ForeignHandle => ("foreign_handle", Authorization, Reauthorize, "The recovery handle belongs to another owner."),
    Self::StaleIncarnation => ("stale_incarnation", Conflict, Retry(WithFreshAuthority), "The recovery handle belongs to a stale incarnation."),
    Self::MissingRecord => ("missing_record", NotFound, Final, "The recovery record is unavailable."),
    Self::Busy => ("busy", Unavailable, Retry(AfterBackoff), "The recovery catalog is busy."),
    Self::TooYoung => ("too_young", Conflict, Retry(AfterBackoff), "The minimum recovery cleanup age has not elapsed."),
    Self::ClockRegressed => ("clock_regressed", Indeterminate, Escalate, "The World clock cannot establish the recovery record's age."),
    Self::CallerCapabilityLive => ("caller_capability_live", Conflict, Final, "A caller still holds the recovery capability."),
    Self::SettlementRequired => ("settlement_required", Conflict, Final, "The recovery record requires settlement."),
    Self::SettlementEvidenceUnavailable => ("settlement_evidence_unavailable", Unavailable, Final, "Evidence for recovery settlement is unavailable."),
    Self::OriginalSourceUnavailable => ("original_source_unavailable", Unavailable, Final, "The original recovery source is unavailable."),
    Self::OriginalSourceMismatch => ("original_source_mismatch", RelationLaw, Escalate, "The recovery source does not match the original."),
    Self::OutputCapacityExhausted => ("output_capacity_exhausted", Exhausted, Final, "The World recovery output capacity is exhausted."),
});

impl std::error::Error for RuntimeWorldRecoveryDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OwnerUnavailable(cause) => Some(cause),
            _ => None,
        }
    }
}
