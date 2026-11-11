use worth_signal::facade::branch::{SignalBranchAdvanceOutcome, SignalBranchForkOutcome};

/// Exact Signal owner progress. Signal has no Relational settlement state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalAttemptProgressPosture {
    Untouched,
    PreparedForExecution,
    Performed,
    DurabilityUnresolved,
}

#[derive(Debug)]
pub(in crate::publication) enum SignalProgressEvidence {
    #[cfg(test)]
    Prepared,
    Advanced(std::sync::Arc<SignalBranchAdvanceOutcome>),
    Forked(SignalBranchForkOutcome),
    Durable(std::sync::Arc<super::super::durable_signal::WorldDurableSignalResult>),
    Unresolved(std::sync::Arc<super::super::durable_signal::WorldDurableSignalAdvance>),
}

#[derive(Debug)]
pub struct SignalAttemptProgress {
    pub(super) posture: SignalAttemptProgressPosture,
    pub(super) evidence: Option<SignalProgressEvidence>,
}

impl SignalAttemptProgress {
    pub(crate) fn untouched() -> Self {
        Self {
            posture: SignalAttemptProgressPosture::Untouched,
            evidence: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn prepared_for_execution() -> Self {
        Self {
            posture: SignalAttemptProgressPosture::PreparedForExecution,
            evidence: Some(SignalProgressEvidence::Prepared),
        }
    }

    pub(crate) const fn summary(posture: SignalAttemptProgressPosture) -> Self {
        Self {
            posture,
            evidence: None,
        }
    }

    pub(crate) fn advanced(outcome: SignalBranchAdvanceOutcome) -> Self {
        Self::advanced_arc(std::sync::Arc::new(outcome))
    }

    pub(crate) fn advanced_arc(outcome: std::sync::Arc<SignalBranchAdvanceOutcome>) -> Self {
        Self {
            posture: SignalAttemptProgressPosture::Performed,
            evidence: Some(SignalProgressEvidence::Advanced(outcome)),
        }
    }

    pub(crate) fn forked(outcome: SignalBranchForkOutcome) -> Self {
        Self {
            posture: SignalAttemptProgressPosture::Performed,
            evidence: Some(SignalProgressEvidence::Forked(outcome)),
        }
    }

    pub(crate) fn durable(
        outcome: std::sync::Arc<super::super::durable_signal::WorldDurableSignalResult>,
    ) -> Self {
        Self {
            posture: SignalAttemptProgressPosture::Performed,
            evidence: Some(SignalProgressEvidence::Durable(outcome)),
        }
    }
    pub(crate) fn unresolved(
        task: std::sync::Arc<super::super::durable_signal::WorldDurableSignalAdvance>,
    ) -> Self {
        Self {
            posture: SignalAttemptProgressPosture::DurabilityUnresolved,
            evidence: Some(SignalProgressEvidence::Unresolved(task)),
        }
    }
    pub const fn posture(&self) -> SignalAttemptProgressPosture {
        self.posture
    }

    pub(crate) fn successor_basis(
        &self,
    ) -> Option<&worth_signal::facade::branch::AdmittedSignalBranchBasis> {
        match self.evidence.as_ref() {
            Some(SignalProgressEvidence::Advanced(outcome)) => Some(outcome.advanced_basis()),
            Some(SignalProgressEvidence::Forked(outcome)) => Some(outcome.created_basis()),
            Some(SignalProgressEvidence::Durable(outcome)) => Some(outcome.output.branch().basis()),
            Some(SignalProgressEvidence::Unresolved(_)) | None => None,
            #[cfg(test)]
            Some(SignalProgressEvidence::Prepared) => None,
        }
    }
}
