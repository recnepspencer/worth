//! Linear outcomes for recovered checkpoint transitions.
use crate::branch::RelationalBranchBasisDescriptor;
use crate::history::data::RelationalCommitReceipt;
use crate::mvcc::RelationalPublicationOutcome;
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
    CodecTransition(String),
    CodecRequest(crate::execution::RelationalExecutionDenialCause),
    CodecIndexPreparation(crate::indexes::data::DerivedIndexMaintenanceDenial),
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

#[derive(Debug)]
pub enum RecoveredCheckpointTransitionError {
    Refused(Box<RefusedRecoveredCheckpointTransition>),
    /// Native settlement refused the performed witness. No successor authority
    /// was issued; the native error retains its posture.
    SettlementFailed(Box<TransactionCommitError>),
}
