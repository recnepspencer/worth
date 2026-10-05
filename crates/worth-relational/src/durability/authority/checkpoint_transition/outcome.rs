//! Linear outcomes and repair custody for recovered checkpoint transitions.
use crate::branch::RelationalBranchBasisDescriptor;
use crate::history::data::RelationalCommitReceipt;
use crate::mvcc::RelationalPublicationOutcome;
use crate::publication::data::DeferredPublicationSettlementError;
use crate::transactions::data::TransactionCommitError;

use super::super::RecoveredRelationalRuntimeAuthority;

/// An acknowledged native commit and recovery authority for its exact successor.
/// Both are issued together by the recovery owner; callers cannot relabel a
/// descriptive receipt as a recovered successor.
///
/// The successor grant cannot be copied into multiple installation attempts:
/// ```compile_fail
/// use worth_relational::facade::durability::RecoveredCheckpointTransition;
/// fn duplicate(transition: RecoveredCheckpointTransition) {
///     let _second = transition.clone();
/// }
/// ```
#[derive(Debug)]
#[must_use = "consume the successor recovery authority for recovered installation"]
pub struct RecoveredCheckpointTransition {
    pub(super) recovered: RecoveredRelationalRuntimeAuthority,
    pub(super) commit: RelationalCommitReceipt,
}

impl RecoveredCheckpointTransition {
    pub fn commit(&self) -> &RelationalCommitReceipt {
        &self.commit
    }

    pub fn into_parts(self) -> (RelationalCommitReceipt, RecoveredRelationalRuntimeAuthority) {
        (self.commit, self.recovered)
    }
}

#[derive(Debug)]
pub enum RecoveredCheckpointTransitionDenial {
    ForeignRecoveryRuntime { expected: u64, actual: u64 },
    ForeignCandidateRuntime { expected: u64, actual: u64 },
    CandidateUnavailable,
    SourceNotRecovered(Box<RelationalBranchBasisDescriptor>),
    Publication(Box<RelationalPublicationOutcome>),
}

/// No transition performed. The original authority remains available for retry;
/// it still admits only its exact recovered images.
#[derive(Debug)]
pub struct RefusedRecoveredCheckpointTransition {
    pub(super) recovered: RecoveredRelationalRuntimeAuthority,
    pub(super) denial: RecoveredCheckpointTransitionDenial,
}

impl RefusedRecoveredCheckpointTransition {
    pub fn denial(&self) -> &RecoveredCheckpointTransitionDenial {
        &self.denial
    }

    pub fn into_authority(self) -> RecoveredRelationalRuntimeAuthority {
        self.recovered
    }
}

/// Native publication performed, but durability did not acknowledge it. This
/// linear value retains the original recovery authority and exact successor
/// alongside the existing owner-held settlement route. It grants no successor
/// authority until the recovery owner repairs that route.
///
/// Deferred performance has no acknowledged successor to extract:
/// ```compile_fail
/// use worth_relational::facade::durability::DeferredRecoveredCheckpointTransition;
/// fn install_before_acknowledgment(transition: DeferredRecoveredCheckpointTransition) {
///     let (_receipt, successor_authority) = transition.into_parts();
/// }
/// ```
#[derive(Debug)]
#[must_use = "repair the performed transition before recovered installation"]
pub struct DeferredRecoveredCheckpointTransition {
    pub(super) recovered: RecoveredRelationalRuntimeAuthority,
    pub(super) successor: RelationalBranchBasisDescriptor,
    pub(super) cause: TransactionCommitError,
}

impl DeferredRecoveredCheckpointTransition {
    pub fn cause(&self) -> &TransactionCommitError {
        &self.cause
    }
}

#[derive(Debug)]
pub enum RecoveredCheckpointTransitionError {
    Refused(Box<RefusedRecoveredCheckpointTransition>),
    DurabilityDeferred(Box<DeferredRecoveredCheckpointTransition>),
    /// Native settlement stopped without a recoverable durability route. No
    /// successor authority was issued; the native error retains its posture.
    SettlementFailed(Box<TransactionCommitError>),
}

/// Repair stopped; the same linear transition remains available for retry.
#[derive(Debug)]
pub struct RecoveredCheckpointTransitionRepairError {
    pub(super) transition: Box<DeferredRecoveredCheckpointTransition>,
    pub(super) cause: DeferredPublicationSettlementError,
}

impl RecoveredCheckpointTransitionRepairError {
    pub fn cause(&self) -> &DeferredPublicationSettlementError {
        &self.cause
    }

    pub fn into_transition(self) -> DeferredRecoveredCheckpointTransition {
        *self.transition
    }
}
