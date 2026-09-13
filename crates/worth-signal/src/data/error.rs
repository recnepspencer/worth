use std::fmt;

use crate::data::graph::ScratchLeaseKind;
use crate::data::handle::NodeId;
use crate::data::node::ContextRequirement;
use crate::logic::transaction::{BranchMergeFailureEvidence, BranchMergeFailureKind};
use crate::state::SignalBranchId;

/// Library-native error type for signal graph operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalError {
    EvaluationStorageCapacityExhausted,
    EvaluationStorageUnavailable,
    SnapshotIndexUnavailable,
    ConditionalEvaluationWorkExhausted {
        maximum_visits: usize,
    },
    UpstreamDependencyWorkExhausted {
        maximum_visits: usize,
    },
    WaiterResolutionWorkExhausted {
        maximum_visits: usize,
    },
    StaleHandle {
        node: NodeId,
        expected_generation: u32,
    },
    CycleDetected {
        path: Vec<NodeId>,
    },
    ScratchReentry {
        active: ScratchLeaseKind,
        attempted: ScratchLeaseKind,
    },
    ScratchMismatch {
        expected: ScratchLeaseKind,
        restored: ScratchLeaseKind,
    },
    ContractViolation {
        node: NodeId,
        requirement: ContextRequirement,
    },
    TransactionFinished,
    TransactionPoisoned,
    EventFlushFailed {
        subscriber: String,
        source: String,
    },
    IncompatibleSnapshot {
        reason: String,
    },
    UnknownBranch {
        branch_id: Option<SignalBranchId>,
        branch_name: String,
    },
    BranchMergeFailed {
        kind: BranchMergeFailureKind,
        message: String,
        evidence: Option<Box<BranchMergeFailureEvidence>>,
    },
    ManagedQueueBranchTransferDenied {
        bound_queue_count: u32,
    },
    InvalidInput {
        message: String,
        context: Option<String>,
    },
    Internal {
        message: String,
        context: Option<String>,
    },
}

impl SignalError {
    pub fn stale_handle(node: NodeId, expected_generation: u32) -> Self {
        Self::StaleHandle {
            node,
            expected_generation,
        }
    }

    pub fn cycle_detected(path: Vec<NodeId>) -> Self {
        Self::CycleDetected { path }
    }

    pub fn scratch_reentry(active: ScratchLeaseKind, attempted: ScratchLeaseKind) -> Self {
        Self::ScratchReentry { active, attempted }
    }

    pub fn scratch_mismatch(expected: ScratchLeaseKind, restored: ScratchLeaseKind) -> Self {
        Self::ScratchMismatch { expected, restored }
    }

    pub fn contract_violation(node: NodeId, requirement: ContextRequirement) -> Self {
        Self::ContractViolation { node, requirement }
    }

    pub fn transaction_finished() -> Self {
        Self::TransactionFinished
    }

    pub fn transaction_poisoned() -> Self {
        Self::TransactionPoisoned
    }

    pub fn event_flush_failed(subscriber: impl Into<String>, source: impl Into<String>) -> Self {
        Self::EventFlushFailed {
            subscriber: subscriber.into(),
            source: source.into(),
        }
    }

    pub fn incompatible_snapshot(reason: impl Into<String>) -> Self {
        Self::IncompatibleSnapshot {
            reason: reason.into(),
        }
    }

    pub fn unknown_branch(
        branch_id: Option<SignalBranchId>,
        branch_name: impl Into<String>,
    ) -> Self {
        Self::UnknownBranch {
            branch_id,
            branch_name: branch_name.into(),
        }
    }

    pub fn branch_merge_failed(kind: BranchMergeFailureKind, message: impl Into<String>) -> Self {
        Self::BranchMergeFailed {
            kind,
            message: message.into(),
            evidence: None,
        }
    }

    pub fn branch_merge_failed_with_evidence(
        kind: BranchMergeFailureKind,
        message: impl Into<String>,
        evidence: BranchMergeFailureEvidence,
    ) -> Self {
        Self::BranchMergeFailed {
            kind,
            message: message.into(),
            evidence: Some(Box::new(evidence)),
        }
    }

    pub fn managed_queue_branch_transfer_denied(bound_queue_count: u32) -> Self {
        Self::ManagedQueueBranchTransferDenied { bound_queue_count }
    }

    /// Build an invalid-input error with no extra context.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
            context: None,
        }
    }

    /// Build an internal error with no extra context.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            context: None,
        }
    }
}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvaluationStorageCapacityExhausted => write!(f, "evaluation storage capacity exhausted"),
            Self::EvaluationStorageUnavailable => write!(f, "evaluation storage unavailable"),
            Self::SnapshotIndexUnavailable => write!(f, "snapshot indexes are unavailable for retained evaluation"),
            Self::ConditionalEvaluationWorkExhausted { maximum_visits } => write!(
                f,
                "conditional evaluation exhausted its {maximum_visits} work allowance"
            ),
            Self::UpstreamDependencyWorkExhausted { maximum_visits } => write!(
                f, "upstream dependency traversal exhausted its {maximum_visits} edge visits"
            ),
            Self::WaiterResolutionWorkExhausted { maximum_visits } => write!(
                f,
                "output waiter resolution exceeded its {maximum_visits} visit allowance"
            ),
            Self::StaleHandle {
                node,
                expected_generation,
            } => write!(
                f,
                "stale handle: {node} (expected generation {expected_generation})"
            ),
            Self::CycleDetected { path } => {
                write!(f, "Circular reference detected along path {:?}", path)
            }
            Self::ScratchReentry { active, attempted } => write!(
                f,
                "signal scratch is already leased for {active:?}; re-entrant {attempted:?} traversal is forbidden"
            ),
            Self::ScratchMismatch { expected, restored } => write!(
                f,
                "signal scratch lease mismatch: expected {expected:?}, restored {restored:?}"
            ),
            Self::ContractViolation { node, requirement } => {
                write!(f, "contract violation for {node}: missing {requirement:?}")
            }
            Self::TransactionFinished => write!(f, "transaction already finished"),
            Self::TransactionPoisoned => write!(f, "transaction is poisoned"),
            Self::EventFlushFailed { subscriber, source } => {
                write!(f, "event bus flush failed at {subscriber}: {source}")
            }
            Self::IncompatibleSnapshot { reason } => {
                write!(f, "incompatible snapshot: {reason}")
            }
            Self::UnknownBranch {
                branch_id,
                branch_name,
            } => match branch_id {
                Some(branch_id) => write!(f, "unknown branch `{}` ({})", branch_name, branch_id.0),
                None => write!(f, "unknown branch `{branch_name}`"),
            },
            Self::BranchMergeFailed {
                kind,
                message,
                evidence,
            } => {
                if let Some(evidence) = evidence {
                    match evidence.as_ref() {
                        BranchMergeFailureEvidence::Conflict(evidence) => write!(
                            f,
                            "branch merge failed ({kind:?}): {message} [{} conflict record(s), primary={:?}]",
                            evidence.records.len(),
                            evidence.summary.primary_conflict_kind
                        ),
                        BranchMergeFailureEvidence::Identity(evidence) => write!(
                            f,
                            "branch merge failed ({kind:?}): {message} [identity source={}, candidates={}]",
                            evidence.source_node,
                            evidence.candidate_target_nodes.len()
                        ),
                        BranchMergeFailureEvidence::Deletion(evidence) => write!(
                            f,
                            "branch merge failed ({kind:?}): {message} [deletion policy={}, target_only={}]",
                            evidence.deletion_policy_name.as_str(),
                            evidence.target_only_nodes.len()
                        ),
                        BranchMergeFailureEvidence::ScopedDenial(evidence) => write!(
                            f,
                            "branch merge failed ({kind:?}): {message} [scope={:?}, denial={:?}, locus={:?}]",
                            evidence.scope_family,
                            evidence.denial_kind,
                            evidence.denied_locus
                        ),
                        BranchMergeFailureEvidence::ScopedUnavailable(evidence) => write!(
                            f,
                            "branch merge failed ({kind:?}): {message} [scope={:?}, reason={:?}, outcome={:?}]",
                            evidence.scope_family,
                            evidence.reason,
                            evidence.outcome_kind
                        ),
                    }
                } else {
                    write!(f, "branch merge failed ({kind:?}): {message}")
                }
            }
            Self::ManagedQueueBranchTransferDenied { bound_queue_count } => write!(
                f,
                "branch transfer denied while {bound_queue_count} managed resource queue binding(s) remain live"
            ),
            Self::InvalidInput { message, .. } => write!(f, "invalid input: {message}"),
            Self::Internal { message, .. } => write!(f, "internal error: {message}"),
        }
    }
}

impl std::error::Error for SignalError {}
