use super::{BankMutationDenial, BankMutationOutcome, BankMutationStatus};
use crate::{
    BankCommitDenialStage, BankCommitReceipt, BankCommitRecoveryKind, BankMutationProposalDenial,
};
use bank_domain::proposals::BankProposalDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankMutationExplanationStage {
    Admission,
    Projection,
    Idempotency,
    EffectPreparation,
    ProviderCommit(BankCommitDenialStage),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankMutationExplanation<'outcome> {
    ProductStale,
    ProductUnpublished,
    NoEffect,
    Committed {
        receipt: &'outcome BankCommitReceipt,
        recovered: bool,
    },
    Stale {
        stale_fact_count: usize,
    },
    Cancelled,
    TimedOut,
    DeadlineExceeded,
    Denied {
        stage: BankMutationExplanationStage,
        reason: &'outcome BankMutationDenial,
    },
    InvariantViolated(&'outcome BankProposalDenial),
    Aborted,
    Deferred,
    SettlementDeferred,
    Indeterminate {
        recovery: BankCommitRecoveryKind,
    },
}

impl BankMutationOutcome {
    pub fn explanation(&self) -> BankMutationExplanation<'_> {
        match self.status() {
            BankMutationStatus::ProductStale(_) => BankMutationExplanation::ProductStale,
            BankMutationStatus::ProductUnpublished(_) => {
                BankMutationExplanation::ProductUnpublished
            }
            BankMutationStatus::NoEffect(_) => BankMutationExplanation::NoEffect,
            BankMutationStatus::Committed(receipt) => BankMutationExplanation::Committed {
                receipt,
                recovered: false,
            },
            BankMutationStatus::AlreadyCommitted(receipt) => BankMutationExplanation::Committed {
                receipt,
                recovered: true,
            },
            BankMutationStatus::Stale { stale_fact_count } => BankMutationExplanation::Stale {
                stale_fact_count: *stale_fact_count,
            },
            BankMutationStatus::Cancelled => BankMutationExplanation::Cancelled,
            BankMutationStatus::TimedOut => BankMutationExplanation::TimedOut,
            BankMutationStatus::DeadlineExceeded => BankMutationExplanation::DeadlineExceeded,
            BankMutationStatus::Denied(reason) => BankMutationExplanation::Denied {
                stage: denial_stage(reason),
                reason,
            },
            BankMutationStatus::InvariantViolated(reason) => {
                BankMutationExplanation::InvariantViolated(reason)
            }
            BankMutationStatus::Aborted => BankMutationExplanation::Aborted,
            BankMutationStatus::Deferred(_) => BankMutationExplanation::Deferred,
            BankMutationStatus::SettlementDeferred(_) => {
                BankMutationExplanation::SettlementDeferred
            }
            BankMutationStatus::Indeterminate(evidence) => BankMutationExplanation::Indeterminate {
                recovery: evidence.recovery_kind(),
            },
        }
    }
}

fn denial_stage(denial: &BankMutationDenial) -> BankMutationExplanationStage {
    match denial {
        BankMutationDenial::ProductSelection(_)
        | BankMutationDenial::Scope(_)
        | BankMutationDenial::Installation(_)
        | BankMutationDenial::Authorization(_) => BankMutationExplanationStage::Admission,
        BankMutationDenial::Proposal(BankMutationProposalDenial::Idempotency(_))
        | BankMutationDenial::IdempotencyIntentDrift => BankMutationExplanationStage::Idempotency,
        BankMutationDenial::Proposal(_) => BankMutationExplanationStage::Projection,
        BankMutationDenial::Preparation(_) => BankMutationExplanationStage::EffectPreparation,
        BankMutationDenial::Commit { stage, .. } => {
            BankMutationExplanationStage::ProviderCommit(*stage)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_noncommit_terminal_retains_distinct_typed_explanation() {
        assert_explanation(
            BankMutationStatus::Stale {
                stale_fact_count: 3,
            },
            |explanation| {
                matches!(
                    explanation,
                    BankMutationExplanation::Stale {
                        stale_fact_count: 3
                    }
                )
            },
        );
        assert_explanation(BankMutationStatus::Cancelled, |explanation| {
            matches!(explanation, BankMutationExplanation::Cancelled)
        });
        assert_explanation(BankMutationStatus::DeadlineExceeded, |explanation| {
            matches!(explanation, BankMutationExplanation::DeadlineExceeded)
        });
        assert_explanation(
            BankMutationStatus::Denied(BankMutationDenial::IdempotencyIntentDrift),
            |explanation| {
                matches!(
                    explanation,
                    BankMutationExplanation::Denied {
                        stage: BankMutationExplanationStage::Idempotency,
                        ..
                    }
                )
            },
        );
        assert_explanation(
            BankMutationStatus::InvariantViolated(BankProposalDenial::SelfApproval),
            |explanation| {
                matches!(
                    explanation,
                    BankMutationExplanation::InvariantViolated(BankProposalDenial::SelfApproval)
                )
            },
        );
        assert_explanation(BankMutationStatus::Aborted, |explanation| {
            matches!(explanation, BankMutationExplanation::Aborted)
        });
    }

    fn assert_explanation(
        status: BankMutationStatus,
        predicate: impl FnOnce(BankMutationExplanation<'_>) -> bool,
    ) {
        let outcome = BankMutationOutcome::new(status, None);
        assert!(predicate(outcome.explanation()));
    }
}
