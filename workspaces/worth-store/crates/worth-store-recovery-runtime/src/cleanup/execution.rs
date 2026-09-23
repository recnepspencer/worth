use crate::handoff::RecoveryCleanupDeferralEvidence;
use crate::progression::ReopenedPhysicalRecovery;

use super::accounting::RecoveryCleanupAccounting;
use super::attempt::{RecoveryCleanupAttempt, RecoveryCleanupAttemptBasis};
use super::command_basis::RecoveryCleanupCommandBasis;
use super::plan::{build_plan, RecoveryCleanupPlanBasis};
use super::{
    PhysicalRecoveryCleanupCancellation, RecoveryCleanupDeferralReason,
    RecoveryCleanupDispositionKind, RecoveryCleanupTarget,
};

pub(crate) fn execute(
    mut reopened: ReopenedPhysicalRecovery,
    cancellation: Option<PhysicalRecoveryCleanupCancellation>,
) -> crate::entry::PhysicalRecoveryOutcome {
    let limits = reopened.state.authority.limits.declaration();
    let mut plan = build_plan(RecoveryCleanupPlanBasis {
        selection: &reopened.state.selection,
        base: &reopened.state.base,
        publication: &reopened.expectation,
        fates: &reopened.state.fates,
        unresolved_retirement: !reopened.state.freshness.retirements().is_empty(),
        limits,
    });
    let command_basis = RecoveryCleanupCommandBasis::from_reopened(
        &mut reopened,
        plan.identity(),
        plan.candidates(),
    );
    debug_assert_eq!(command_basis.descriptive_plan_identity(), plan.identity());
    if let Some(identity) = command_basis.plan_identity() {
        plan.bind_authority_identity(identity);
    }
    let mut accounting =
        RecoveryCleanupAccounting::begin(&plan, command_basis.terminal_binding_evaluations());
    let candidates = plan.take_eligibilities();
    let cancellation_at = match cancellation {
        Some(cancellation) => match cancellation.admit(plan.identity(), candidates.len() as u64) {
            Some(safe_point) => Some(safe_point),
            None => {
                defer_for_invalid_cancellation(&mut plan, &mut accounting, &candidates);
                return finish(reopened, plan, accounting, command_basis);
            }
        },
        None => None,
    };
    CandidateExecution {
        reopened: &reopened,
        plan: &mut plan,
        command_basis: &command_basis,
        accounting: &mut accounting,
        policy: None,
    }
    .run(candidates, cancellation_at);
    finish(reopened, plan, accounting, command_basis)
}

struct CandidateExecution<'a> {
    reopened: &'a ReopenedPhysicalRecovery,
    plan: &'a mut super::RecoveryCleanupPlan,
    command_basis: &'a RecoveryCleanupCommandBasis,
    accounting: &'a mut RecoveryCleanupAccounting,
    policy: Option<[u8; 32]>,
}

impl CandidateExecution<'_> {
    fn run(
        &mut self,
        candidates: Vec<super::RecoveryCleanupEligibility>,
        cancellation_at: Option<u64>,
    ) {
        for (index, candidate) in candidates.into_iter().enumerate() {
            if cancellation_at == Some(index as u64) {
                defer_for_cancellation(self.plan, self.accounting, candidate, index as u64);
                break;
            }
            let artifact = candidate.artifact();
            let byte_count = candidate.byte_count();
            let attempt =
                RecoveryCleanupAttemptBasis::new(self.reopened, self.plan, self.command_basis)
                    .execute(self.policy, candidate);
            match attempt {
                RecoveryCleanupAttempt::Completed {
                    freshness,
                    performed,
                    revalidation,
                } => {
                    self.policy.get_or_insert(freshness.policy_identity());
                    self.accounting.record_freshness_sample(freshness);
                    self.accounting
                        .record_completed(byte_count, performed, revalidation);
                    debug_assert!(self.plan.transition_candidate(
                        artifact,
                        RecoveryCleanupDispositionKind::SafelyRemoved,
                    ));
                }
                RecoveryCleanupAttempt::Deferred {
                    freshness,
                    evidence,
                    stop,
                } => {
                    if let Some(freshness) = freshness {
                        self.policy.get_or_insert(freshness.policy_identity());
                        self.accounting.record_freshness_sample(freshness);
                    }
                    let reason = deferral_reason(&evidence);
                    debug_assert!(self.plan.transition_candidate(
                        artifact,
                        RecoveryCleanupDispositionKind::Deferred(reason),
                    ));
                    self.accounting.record_deferral(evidence);
                    if stop {
                        self.plan.defer_remaining(reason);
                        break;
                    }
                }
            }
        }
    }
}

fn defer_for_cancellation(
    plan: &mut super::RecoveryCleanupPlan,
    accounting: &mut RecoveryCleanupAccounting,
    candidate: super::RecoveryCleanupEligibility,
    settled_actions: u64,
) {
    let target = RecoveryCleanupTarget::Wal(candidate.artifact());
    let remaining_actions = plan
        .dispositions()
        .iter()
        .filter(|disposition| disposition.kind() == RecoveryCleanupDispositionKind::Eligible)
        .count() as u64;
    let remaining_bytes = plan
        .dispositions()
        .iter()
        .filter(|disposition| disposition.kind() == RecoveryCleanupDispositionKind::Eligible)
        .map(|disposition| disposition.byte_count())
        .sum();
    plan.defer_remaining(RecoveryCleanupDeferralReason::Cancelled);
    accounting.record_cancellation(
        remaining_actions,
        remaining_bytes,
        RecoveryCleanupDeferralEvidence::Cancelled {
            target,
            settled_actions,
        },
    );
}

fn defer_for_invalid_cancellation(
    plan: &mut super::RecoveryCleanupPlan,
    accounting: &mut RecoveryCleanupAccounting,
    candidates: &[super::RecoveryCleanupEligibility],
) {
    let Some(candidate) = candidates.first() else {
        return;
    };
    let bytes = candidates
        .iter()
        .map(|candidate| candidate.byte_count())
        .sum();
    plan.defer_remaining(RecoveryCleanupDeferralReason::CancellationBindingMismatch);
    accounting.record_cancellation(
        candidates.len() as u64,
        bytes,
        RecoveryCleanupDeferralEvidence::CancellationBindingMismatch {
            target: RecoveryCleanupTarget::Wal(candidate.artifact()),
        },
    );
}

#[cfg_attr(not(feature = "certification-test-authority"), allow(unused_mut))]
fn finish(
    mut reopened: ReopenedPhysicalRecovery,
    plan: super::RecoveryCleanupPlan,
    accounting: RecoveryCleanupAccounting,
    command_basis: RecoveryCleanupCommandBasis,
) -> crate::entry::PhysicalRecoveryOutcome {
    #[cfg(feature = "certification-test-authority")]
    if reopened
        .state
        .coordination
        .owner()
        .take_certification_cleanup_media_handle_leak()
    {
        assert!(
            reopened
                .state
                .authority
                .media
                .certification_retain_cleanup_media_handle(),
            "certification cleanup handle leak must reach the admitted backend owner"
        );
    }
    let live_media_handles_after_close =
        command_basis.live_media_handle_delta(&reopened.state.authority.media);
    let closed_cleanup = command_basis.close(live_media_handles_after_close);
    let posture = accounting.finish(plan, live_media_handles_after_close);
    crate::orchestration::finish_recovery_after_cleanup(reopened, closed_cleanup, posture)
}

const fn deferral_reason(
    evidence: &RecoveryCleanupDeferralEvidence,
) -> RecoveryCleanupDeferralReason {
    match evidence {
        RecoveryCleanupDeferralEvidence::Freshness { .. } => {
            RecoveryCleanupDeferralReason::FreshnessUnavailable
        }
        RecoveryCleanupDeferralEvidence::PublishedGenerationChanged { .. } => {
            RecoveryCleanupDeferralReason::PublishedGenerationChanged
        }
        RecoveryCleanupDeferralEvidence::EligibilityChanged { .. } => {
            RecoveryCleanupDeferralReason::EligibilityChanged
        }
        RecoveryCleanupDeferralEvidence::Cancelled { .. } => {
            RecoveryCleanupDeferralReason::Cancelled
        }
        RecoveryCleanupDeferralEvidence::CancellationBindingMismatch { .. } => {
            RecoveryCleanupDeferralReason::CancellationBindingMismatch
        }
        RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { .. } => {
            RecoveryCleanupDeferralReason::DeniedBeforeEffect
        }
        RecoveryCleanupDeferralEvidence::IndeterminateEffect { .. } => {
            RecoveryCleanupDeferralReason::IndeterminateEffect
        }
    }
}
