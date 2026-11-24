use serde::{Deserialize, Serialize};

use super::operation_control::interruption_event::{
    RelationalInterruptionBoundary, RelationalInterruptionEvent, RelationalOperationInterruption,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalInterruptionCostCounters {
    cancelled: RelationalInterruptionBoundaryCounters,
    timed_out: RelationalInterruptionBoundaryCounters,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
struct RelationalInterruptionBoundaryCounters {
    observation_admission: u64,
    transaction_admission: u64,
    proposal_validation: u64,
    candidate_preparation: u64,
    publication_preflight: u64,
    before_critical_section: u64,
    after_linearization: u64,
    settlement: u64,
}

impl RelationalInterruptionCostCounters {
    pub const fn count(
        self,
        boundary: RelationalInterruptionBoundary,
        interruption: RelationalOperationInterruption,
    ) -> u64 {
        match interruption {
            RelationalOperationInterruption::Cancelled => self.cancelled.count(boundary),
            RelationalOperationInterruption::TimedOut => self.timed_out.count(boundary),
        }
    }
}

impl RelationalInterruptionBoundaryCounters {
    const fn count(self, boundary: RelationalInterruptionBoundary) -> u64 {
        match boundary {
            RelationalInterruptionBoundary::ObservationAdmission => self.observation_admission,
            RelationalInterruptionBoundary::TransactionAdmission => self.transaction_admission,
            RelationalInterruptionBoundary::ProposalValidation => self.proposal_validation,
            RelationalInterruptionBoundary::CandidatePreparation => self.candidate_preparation,
            RelationalInterruptionBoundary::PublicationPreflight => self.publication_preflight,
            RelationalInterruptionBoundary::BeforeCriticalSection => self.before_critical_section,
            RelationalInterruptionBoundary::AfterLinearization => self.after_linearization,
            RelationalInterruptionBoundary::Settlement => self.settlement,
        }
    }

    fn record(&mut self, boundary: RelationalInterruptionBoundary) {
        let counter = match boundary {
            RelationalInterruptionBoundary::ObservationAdmission => &mut self.observation_admission,
            RelationalInterruptionBoundary::TransactionAdmission => &mut self.transaction_admission,
            RelationalInterruptionBoundary::ProposalValidation => &mut self.proposal_validation,
            RelationalInterruptionBoundary::CandidatePreparation => &mut self.candidate_preparation,
            RelationalInterruptionBoundary::PublicationPreflight => &mut self.publication_preflight,
            RelationalInterruptionBoundary::BeforeCriticalSection => {
                &mut self.before_critical_section
            }
            RelationalInterruptionBoundary::AfterLinearization => &mut self.after_linearization,
            RelationalInterruptionBoundary::Settlement => &mut self.settlement,
        };
        *counter = counter.saturating_add(1);
    }

    fn saturating_add(self, other: Self) -> Self {
        Self {
            observation_admission: self
                .observation_admission
                .saturating_add(other.observation_admission),
            transaction_admission: self
                .transaction_admission
                .saturating_add(other.transaction_admission),
            proposal_validation: self
                .proposal_validation
                .saturating_add(other.proposal_validation),
            candidate_preparation: self
                .candidate_preparation
                .saturating_add(other.candidate_preparation),
            publication_preflight: self
                .publication_preflight
                .saturating_add(other.publication_preflight),
            before_critical_section: self
                .before_critical_section
                .saturating_add(other.before_critical_section),
            after_linearization: self
                .after_linearization
                .saturating_add(other.after_linearization),
            settlement: self.settlement.saturating_add(other.settlement),
        }
    }

    fn saturating_delta_since(self, baseline: Self) -> Self {
        Self {
            observation_admission: self
                .observation_admission
                .saturating_sub(baseline.observation_admission),
            transaction_admission: self
                .transaction_admission
                .saturating_sub(baseline.transaction_admission),
            proposal_validation: self
                .proposal_validation
                .saturating_sub(baseline.proposal_validation),
            candidate_preparation: self
                .candidate_preparation
                .saturating_sub(baseline.candidate_preparation),
            publication_preflight: self
                .publication_preflight
                .saturating_sub(baseline.publication_preflight),
            before_critical_section: self
                .before_critical_section
                .saturating_sub(baseline.before_critical_section),
            after_linearization: self
                .after_linearization
                .saturating_sub(baseline.after_linearization),
            settlement: self.settlement.saturating_sub(baseline.settlement),
        }
    }
}

/// Operations used by the Relational owners.
pub trait RelationalInterruptionCostCountersOwnerPort: Sized {
    fn record(&mut self, event: RelationalInterruptionEvent);
    fn saturating_add(self, other: Self) -> Self;
    fn saturating_delta_since(self, baseline: Self) -> Self;
}

impl RelationalInterruptionCostCountersOwnerPort for RelationalInterruptionCostCounters {
    fn record(&mut self, event: RelationalInterruptionEvent) {
        let counters = match event.interruption() {
            RelationalOperationInterruption::Cancelled => &mut self.cancelled,
            RelationalOperationInterruption::TimedOut => &mut self.timed_out,
        };
        counters.record(event.boundary());
    }

    fn saturating_add(self, other: Self) -> Self {
        Self {
            cancelled: self.cancelled.saturating_add(other.cancelled),
            timed_out: self.timed_out.saturating_add(other.timed_out),
        }
    }

    fn saturating_delta_since(self, baseline: Self) -> Self {
        Self {
            cancelled: self.cancelled.saturating_delta_since(baseline.cancelled),
            timed_out: self.timed_out.saturating_delta_since(baseline.timed_out),
        }
    }
}
