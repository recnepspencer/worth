use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::branch::RelationalForkOutcome;
use worth_relational::facade::history::RelationalCommitIdentity;
use worth_relational::facade::mvcc::PerformedRelationalCommit;
use worth_relational::facade::publication::DeferredPublicationSettlement;
use worth_relational::facade::transactions::CommitResult;
use worth_signal::facade::branch::{SignalBranchAdvanceOutcome, SignalBranchForkOutcome};

#[path = "progress/recovery.rs"]
mod recovery;
pub(crate) use recovery::RelationalRecoveryRoute;

#[path = "progress/effect_count.rs"]
mod effect_count;
pub(crate) use effect_count::owner_effect_count_from_postures;

#[path = "progress/relational.rs"]
mod relational;

#[path = "progress/ready.rs"]
mod ready;
mod retained_image;

/// Exact Relational owner progress. A generic ordinal cannot say which owner
/// evidence or settlement obligation is alive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalAttemptProgressPosture {
    Untouched,
    Prepared,
    Performed,
    SettlementRequired,
    SettlementPending,
    Settled,
}

#[derive(Debug)]
pub(super) enum RelationalProgressEvidence {
    Performed(PerformedRelationalCommit),
    SettlementRequired {
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
    },
    SettlementPending {
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        settlement: DeferredPublicationSettlement,
    },
    SettledReceipt {
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        receipt: worth_relational::facade::history::RelationalCommitReceipt,
    },
    Settled {
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        result: std::sync::Arc<CommitResult>,
    },
}

#[derive(Debug)]
pub struct RelationalAttemptProgress {
    posture: RelationalAttemptProgressPosture,
    evidence: Option<RelationalProgressEvidence>,
    fork: Option<RelationalForkOutcome>,
    fork_successor_basis: Option<AdmittedRelationalBranchBasis>,
}

impl RelationalAttemptProgress {
    pub(crate) fn untouched() -> Self {
        Self {
            posture: RelationalAttemptProgressPosture::Untouched,
            evidence: None,
            fork: None,
            fork_successor_basis: None,
        }
    }

    /// Record a performed Relational route after its owner consumed the
    /// linear performed witness and returned a deferred settlement. The
    /// successor basis is captured before that consuming owner call; the
    /// deferred settlement is the remaining owner-issued repair authority.
    pub(crate) fn settlement_pending(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        settlement: DeferredPublicationSettlement,
    ) -> Self {
        Self {
            posture: RelationalAttemptProgressPosture::SettlementPending,
            evidence: Some(RelationalProgressEvidence::SettlementPending {
                commit_identity,
                successor_basis,
                settlement,
            }),
            fork: None,
            fork_successor_basis: None,
        }
    }

    pub(crate) fn performed(performed: PerformedRelationalCommit) -> Self {
        Self {
            posture: RelationalAttemptProgressPosture::Performed,
            evidence: Some(RelationalProgressEvidence::Performed(performed)),
            fork: None,
            fork_successor_basis: None,
        }
    }

    /// Record the exact result returned by the consuming Relational
    /// settlement call. The commit receipt is retained inside `CommitResult`;
    /// no second performed witness is manufactured.
    pub(crate) fn settled(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        result: CommitResult,
    ) -> Self {
        Self {
            posture: RelationalAttemptProgressPosture::Settled,
            evidence: Some(RelationalProgressEvidence::Settled {
                commit_identity,
                successor_basis,
                result: std::sync::Arc::new(result),
            }),
            fork: None,
            fork_successor_basis: None,
        }
    }

    pub const fn posture(&self) -> RelationalAttemptProgressPosture {
        self.posture
    }

    pub(crate) fn successor_basis(&self) -> Option<&AdmittedRelationalBranchBasis> {
        match self.evidence.as_ref() {
            Some(RelationalProgressEvidence::Performed(performed)) => Some(performed.next_basis()),
            Some(RelationalProgressEvidence::SettlementRequired {
                successor_basis, ..
            })
            | Some(RelationalProgressEvidence::SettlementPending {
                successor_basis, ..
            })
            | Some(RelationalProgressEvidence::Settled {
                successor_basis, ..
            }) => Some(successor_basis),
            Some(RelationalProgressEvidence::SettledReceipt {
                successor_basis, ..
            }) => Some(successor_basis),
            None => self.fork_successor_basis.as_ref(),
        }
    }
}

/// Exact Signal owner progress. Signal has no Relational settlement state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalAttemptProgressPosture {
    Untouched,
    PreparedForExecution,
    Performed,
}

#[derive(Debug)]
pub(super) enum SignalProgressEvidence {
    #[cfg(test)]
    Prepared,
    Advanced(std::sync::Arc<SignalBranchAdvanceOutcome>),
    Forked(SignalBranchForkOutcome),
}

#[derive(Debug)]
pub struct SignalAttemptProgress {
    posture: SignalAttemptProgressPosture,
    evidence: Option<SignalProgressEvidence>,
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
        Self {
            posture: SignalAttemptProgressPosture::Performed,
            evidence: Some(SignalProgressEvidence::Advanced(std::sync::Arc::new(
                outcome,
            ))),
        }
    }

    pub(crate) fn forked(outcome: SignalBranchForkOutcome) -> Self {
        Self {
            posture: SignalAttemptProgressPosture::Performed,
            evidence: Some(SignalProgressEvidence::Forked(outcome)),
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
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct CompositeAttemptProgress {
    relational: RelationalAttemptProgress,
    signal: SignalAttemptProgress,
}

impl CompositeAttemptProgress {
    pub(crate) fn untouched() -> Self {
        Self {
            relational: RelationalAttemptProgress::untouched(),
            signal: SignalAttemptProgress::untouched(),
        }
    }

    pub(crate) fn new(
        relational: RelationalAttemptProgress,
        signal: SignalAttemptProgress,
    ) -> Self {
        Self { relational, signal }
    }

    pub const fn relational_posture(&self) -> RelationalAttemptProgressPosture {
        self.relational.posture()
    }

    pub(crate) fn set_signal(&mut self, signal: SignalAttemptProgress) {
        self.signal = signal;
    }

    pub const fn signal_posture(&self) -> SignalAttemptProgressPosture {
        self.signal.posture()
    }

    pub fn relational(&self) -> &RelationalAttemptProgress {
        &self.relational
    }

    pub(crate) fn relational_requires_settlement(&self) -> bool {
        self.relational.requires_settlement()
    }

    pub fn signal(&self) -> &SignalAttemptProgress {
        &self.signal
    }

    pub(crate) const fn owner_effect_count(&self) -> usize {
        owner_effect_count_from_postures(self.relational.posture(), self.signal.posture())
    }

    pub(crate) fn into_parts(self) -> (RelationalAttemptProgress, SignalAttemptProgress) {
        (self.relational, self.signal)
    }
}

#[cfg(test)]
#[path = "progress/tests.rs"]
mod tests;
