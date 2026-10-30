//! Disposable history of actual conditional executions, separate from Native custody.
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, MemoryLimitDenial};
use worth_signal::facade::{SignalConditionalDecisionClass, SignalConditionalDecisionEvidence};
mod retention;
pub(crate) use retention::ConditionalExecutionHistory;

/// The exact history owner's outcome; missing detail is never producer absence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeConditionalHistoryAvailability {
    RetainedAvailable,
    OmittedByTier,
    CaptureDenied(MemoryLimitDenial),
    EvictedByRetention,
    OwnerClosed,
}

/// Fixed descriptive facts copied only from a real successful Signal decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeConditionalHistoryDetail {
    attempt: u64,
    class: SignalConditionalDecisionClass,
    output_version_before: u64,
    output_version: u64,
}
impl BridgeConditionalHistoryDetail {
    pub fn attempt(&self) -> u64 {
        self.attempt
    }
    pub fn class(&self) -> SignalConditionalDecisionClass {
        self.class
    }
    pub fn output_version_before(&self) -> u64 {
        self.output_version_before
    }
    pub fn output_version(&self) -> u64 {
        self.output_version
    }
    fn from_decision(decision: &SignalConditionalDecisionEvidence) -> Self {
        Self {
            attempt: decision.attempt(),
            class: decision.class(),
            output_version_before: decision.output_version_before(),
            output_version: decision.output_version(),
        }
    }
}

/// Owner-issued association to one exact execution, without a reader or authority.
/// A surviving original can observe eviction but cannot reconstruct lost history.
#[derive(Clone)]
pub(crate) enum ConditionalHistoryReference {
    Unretained(BridgeConditionalHistoryAvailability),
    Retained(Arc<HistorySlot>),
}
impl ConditionalHistoryReference {
    pub(crate) fn availability(&self) -> BridgeConditionalHistoryAvailability {
        match self {
            Self::Unretained(cause) => *cause,
            Self::Retained(slot) => slot
                .node
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .state
                .availability(),
        }
    }
    pub(crate) fn detail(
        &self,
    ) -> Result<BridgeConditionalHistoryDetail, BridgeConditionalHistoryAvailability> {
        match self {
            Self::Unretained(cause) => Err(*cause),
            Self::Retained(slot) => match slot
                .node
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .state
            {
                HistoryState::Retained(detail) => Ok(detail),
                HistoryState::Lost(cause) => Err(cause),
            },
        }
    }
}

pub(crate) struct HistorySlot {
    node: Mutex<HistoryNode>,
    // The Arc allocation remains charged until its final original/history share.
    _memory: Option<ExecutionMemoryReservation>,
}
struct HistoryNode {
    state: HistoryState,
    next: Option<Arc<HistorySlot>>,
}
#[derive(Clone, Copy)]
enum HistoryState {
    Retained(BridgeConditionalHistoryDetail),
    Lost(BridgeConditionalHistoryAvailability),
}
impl HistoryState {
    fn availability(self) -> BridgeConditionalHistoryAvailability {
        match self {
            Self::Retained(_) => BridgeConditionalHistoryAvailability::RetainedAvailable,
            Self::Lost(cause) => cause,
        }
    }
}
impl std::fmt::Debug for ConditionalExecutionHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConditionalExecutionHistory")
            .finish_non_exhaustive()
    }
}
