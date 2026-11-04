use worth_store::physical_runtime::{
    PhysicalRecoveryCleanupRemovalOutcome, StoreRecoveryCleanupAttempt,
    StoreRecoveryCleanupFreshnessSample,
};

use crate::handoff::RecoveryCleanupDeferralEvidence;
use crate::progression::ReopenedPhysicalRecovery;

use super::{
    command_basis::{RecoveryCleanupCommandBasis, StoreCleanupCommandExecution},
    PerformedRecoveryCleanupRemoval, RecoveryCleanupEligibility, RecoveryCleanupPlan,
    RecoveryCleanupTarget,
};

pub(super) enum RecoveryCleanupAttempt {
    Completed {
        freshness: StoreRecoveryCleanupFreshnessSample,
        performed: PerformedRecoveryCleanupRemoval,
        revalidation: worth_store::physical_runtime::RecoveryCleanupArtifactRevalidationProgress,
    },
    Deferred {
        freshness: Option<StoreRecoveryCleanupFreshnessSample>,
        evidence: RecoveryCleanupDeferralEvidence,
        stop: bool,
    },
}

pub(super) struct RecoveryCleanupAttemptBasis<'recovery, 'basis> {
    reopened: &'recovery ReopenedPhysicalRecovery,
    plan: &'basis RecoveryCleanupPlan,
    command_basis: &'basis RecoveryCleanupCommandBasis,
}

impl<'recovery, 'basis> RecoveryCleanupAttemptBasis<'recovery, 'basis> {
    pub(super) const fn new(
        reopened: &'recovery ReopenedPhysicalRecovery,
        plan: &'basis RecoveryCleanupPlan,
        command_basis: &'basis RecoveryCleanupCommandBasis,
    ) -> Self {
        Self {
            reopened,
            plan,
            command_basis,
        }
    }

    pub(super) fn execute(
        &self,
        expected_policy: Option<[u8; 32]>,
        candidate: RecoveryCleanupEligibility,
    ) -> RecoveryCleanupAttempt {
        let target = RecoveryCleanupTarget::Wal(candidate.artifact());
        let attempt = match self.command_basis.execute(
            self.reopened.payload().state.coordination.owner(),
            &self.reopened.payload().state.authority.media,
            candidate.artifact(),
        ) {
            StoreCleanupCommandExecution::Attempt(attempt) => attempt,
            StoreCleanupCommandExecution::AdmissionDenied(failure) => {
                return RecoveryCleanupAttempt::Deferred {
                    freshness: None,
                    evidence: RecoveryCleanupDeferralEvidence::Freshness { target, failure },
                    stop: true,
                };
            }
            StoreCleanupCommandExecution::Unavailable => {
                return RecoveryCleanupAttempt::Deferred {
                    freshness: None,
                    evidence: RecoveryCleanupDeferralEvidence::EligibilityChanged { target },
                    stop: true,
                };
            }
        };
        match attempt {
            StoreRecoveryCleanupAttempt::FreshnessDenied(failure) => {
                RecoveryCleanupAttempt::Deferred {
                    // The failure owns any completed freshness sample. Keeping
                    // a second copy here would make accounting record one
                    // physical read and settlement twice.
                    freshness: None,
                    evidence: RecoveryCleanupDeferralEvidence::Freshness { target, failure },
                    stop: true,
                }
            }
            StoreRecoveryCleanupAttempt::PublishedGenerationChanged(freshness) => {
                let evidence = self
                    .changed_evidence(target.clone(), &freshness, expected_policy)
                    .unwrap_or(RecoveryCleanupDeferralEvidence::EligibilityChanged { target });
                RecoveryCleanupAttempt::Deferred {
                    freshness: Some(freshness),
                    evidence,
                    stop: true,
                }
            }
            StoreRecoveryCleanupAttempt::Removal { freshness, outcome } => {
                lower_removal_outcome(target, freshness, outcome)
            }
        }
    }

    fn changed_evidence(
        &self,
        target: RecoveryCleanupTarget,
        freshness: &StoreRecoveryCleanupFreshnessSample,
        expected_policy: Option<[u8; 32]>,
    ) -> Option<RecoveryCleanupDeferralEvidence> {
        if freshness.observed_published_generation() != self.plan.published_generation() {
            Some(
                RecoveryCleanupDeferralEvidence::PublishedGenerationChanged {
                    target,
                    expected: self.plan.published_generation(),
                    observed: freshness.observed_published_generation(),
                },
            )
        } else if !freshness_matches(self.reopened, self.plan, freshness, expected_policy) {
            Some(RecoveryCleanupDeferralEvidence::EligibilityChanged { target })
        } else {
            None
        }
    }
}

fn lower_removal_outcome(
    target: RecoveryCleanupTarget,
    freshness: StoreRecoveryCleanupFreshnessSample,
    outcome: PhysicalRecoveryCleanupRemovalOutcome,
) -> RecoveryCleanupAttempt {
    match outcome {
        PhysicalRecoveryCleanupRemovalOutcome::Completed(completed) => {
            let revalidation = completed.revalidation();
            RecoveryCleanupAttempt::Completed {
                freshness,
                performed: PerformedRecoveryCleanupRemoval::new(completed.into_performed()),
                revalidation,
            }
        }
        PhysicalRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(denial) => {
            RecoveryCleanupAttempt::Deferred {
                freshness: Some(freshness),
                evidence: RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { target, denial },
                stop: false,
            }
        }
        PhysicalRecoveryCleanupRemovalOutcome::Indeterminate(evidence) => {
            RecoveryCleanupAttempt::Deferred {
                freshness: Some(freshness),
                evidence: RecoveryCleanupDeferralEvidence::IndeterminateEffect { target, evidence },
                stop: true,
            }
        }
    }
}

fn freshness_matches(
    reopened: &ReopenedPhysicalRecovery,
    plan: &RecoveryCleanupPlan,
    sample: &StoreRecoveryCleanupFreshnessSample,
    expected_policy: Option<[u8; 32]>,
) -> bool {
    sample.store_identity() == reopened.store_identity()
        && Some(sample.cleanup_plan_identity()) == plan.authority_identity()
        && sample.sealed_publication_basis() == reopened.payload().expectation.plan_identity()
        && sample.policy_identity() != [0; 32]
        && expected_policy.is_none_or(|policy| policy == sample.policy_identity())
}
