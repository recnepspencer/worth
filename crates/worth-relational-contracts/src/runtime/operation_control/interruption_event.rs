use serde::{Deserialize, Serialize};

/// Typed reason why owner work stopped before its effect boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalOperationInterruption {
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalInterruptionBoundary {
    ObservationAdmission,
    TransactionAdmission,
    ProposalValidation,
    CandidatePreparation,
    PublicationPreflight,
    BeforeCriticalSection,
    AfterLinearization,
    Settlement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalInterruptionEvent {
    pub(super) interruption: RelationalOperationInterruption,
    pub(super) boundary: RelationalInterruptionBoundary,
}

impl RelationalInterruptionEvent {
    /// Observes the caller's cancellation or deadline at a named boundary.
    /// This consults borrowed request control only; it creates no request and
    /// performs no storage work. Readers still charge their own native work.
    /// Retains the interruption and boundary of an already-produced request stop.
    /// Work and nested stops remain the caller's typed stop rather than an event.
    pub fn from_request_stop(
        stop: worth_execution::MapKernelStop,
        boundary: RelationalInterruptionBoundary,
    ) -> Result<Self, worth_execution::MapKernelStop> {
        use worth_execution::MapKernelStop as Stop;
        let interruption = match stop {
            Stop::Cancelled => RelationalOperationInterruption::Cancelled,
            Stop::DeadlineElapsed => RelationalOperationInterruption::TimedOut,
            Stop::WorkCounterOverflow
            | Stop::AccountingUnavailable
            | Stop::WorkCeiling
            | Stop::NestedStopped => return Err(stop),
        };
        Ok(Self {
            interruption,
            boundary,
        })
    }

    pub const fn interruption(self) -> RelationalOperationInterruption {
        self.interruption
    }

    pub const fn boundary(self) -> RelationalInterruptionBoundary {
        self.boundary
    }
}

impl RelationalInterruptionEvent {
    #[cfg(test)]
    pub(crate) fn observe_request(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        boundary: RelationalInterruptionBoundary,
    ) -> Option<Self> {
        match execution.check_live() {
            Ok(()) => None,
            Err(stop) => Self::from_request_stop(stop, boundary).ok(),
        }
    }
}
