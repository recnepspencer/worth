//! Bank-owned terminal classification of one Query application commit.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDeferred, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationUncommitted, WorthQueryCustomInvariantDenial,
    WorthQueryProductStaleApplication, WorthQueryProductUnpublishedApplication,
};
use worth_query_host::facade::product::WorthQueryApplicationNoEffectCause;

use super::commit_denial::{denial_kind, denial_stage};
use super::{
    commit_receipt, BankCommitDenialKind, BankCommitDenialStage, BankCommitReceipt,
    BankCommitRecoveryKind, BankUnresolvedCommitEvidence,
};

#[derive(Debug)]
pub enum BankMutationCommitOutcome {
    ProductPreparationPending(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationPending,
    ),
    ProductPreparationRefused(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationRefused,
    ),
    CleanupRequired(super::BankManagedCleanup),
    ProductStale(WorthQueryProductStaleApplication),
    ProductUnpublished(WorthQueryProductUnpublishedApplication),
    NoEffect(WorthQueryApplicationNoEffectCause),
    Committed(BankCommitReceipt),
    RecoveredCommitted(
        worth_query_host::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt,
    ),
    Stale {
        stale_fact_count: usize,
    },
    Cancelled,
    TimedOut,
    Denied {
        kind: BankCommitDenialKind,
        stage: BankCommitDenialStage,
    },
    CustomInvariantDenied {
        denial: WorthQueryCustomInvariantDenial,
        stage: BankCommitDenialStage,
    },
    Aborted,
    Deferred(WorthQueryApplicationCommitDeferred),
    /// The commit's fate is unknown. The same retained Query evidence names
    /// which recovery the operator owes.
    Indeterminate(BankUnresolvedCommitEvidence),
}

impl BankMutationCommitOutcome {
    pub const fn unresolved_evidence(&self) -> Option<&BankUnresolvedCommitEvidence> {
        match self {
            Self::Indeterminate(evidence) => Some(evidence),
            _ => None,
        }
    }

    pub const fn commit_recovery_kind(&self) -> Option<BankCommitRecoveryKind> {
        match self.unresolved_evidence() {
            Some(evidence) => Some(evidence.recovery_kind()),
            None => None,
        }
    }
}

impl From<WorthQueryApplicationCommitOutcome> for BankMutationCommitOutcome {
    fn from(outcome: WorthQueryApplicationCommitOutcome) -> Self {
        match outcome {
            WorthQueryApplicationCommitOutcome::ProductPreparationPending(pending) => {
                Self::ProductPreparationPending(pending)
            }
            WorthQueryApplicationCommitOutcome::ProductPreparationRefused(refused) => {
                Self::ProductPreparationRefused(refused)
            }
            WorthQueryApplicationCommitOutcome::CleanupRequired(owner) => {
                Self::CleanupRequired(owner.into())
            }
            WorthQueryApplicationCommitOutcome::ProductStale(stale) => Self::ProductStale(stale),
            WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished) => {
                Self::ProductUnpublished(unpublished)
            }
            WorthQueryApplicationCommitOutcome::NoEffect(no_effect) => {
                Self::NoEffect(no_effect.cause())
            }
            WorthQueryApplicationCommitOutcome::Committed(receipt) => {
                Self::Committed(commit_receipt(receipt))
            }
            WorthQueryApplicationCommitOutcome::RecoveredCommitted(receipt) => {
                Self::RecoveredCommitted(receipt)
            }
            WorthQueryApplicationCommitOutcome::Stale(stale) => Self::Stale {
                stale_fact_count: stale.stale_fact_count(),
            },
            WorthQueryApplicationCommitOutcome::Cancelled => Self::Cancelled,
            WorthQueryApplicationCommitOutcome::TimedOut => Self::TimedOut,
            WorthQueryApplicationCommitOutcome::Denied(denial) => {
                match denial.custom_invariant_denial() {
                    Some(custom) => Self::CustomInvariantDenied {
                        denial: custom.clone(),
                        stage: denial_stage(denial.stage()),
                    },
                    None => Self::Denied {
                        kind: denial_kind(denial.kind()),
                        stage: denial_stage(denial.stage()),
                    },
                }
            }
            WorthQueryApplicationCommitOutcome::Aborted => Self::Aborted,
            WorthQueryApplicationCommitOutcome::Deferred(deferred) => Self::Deferred(deferred),
            WorthQueryApplicationCommitOutcome::Indeterminate(evidence) => {
                Self::Indeterminate(BankUnresolvedCommitEvidence::from_execution(evidence))
            }
        }
    }
}

impl From<WorthQueryApplicationUncommitted> for BankMutationCommitOutcome {
    fn from(uncommitted: WorthQueryApplicationUncommitted) -> Self {
        WorthQueryApplicationCommitOutcome::from(uncommitted).into()
    }
}
