use worth_store::physical_runtime::{
    PhysicalRecoveryCleanupRemovalDenial, PhysicalRecoveryCleanupRemovalIndeterminate,
    StoreRecoveryCleanupFreshnessFailure, StoreRecoveryCleanupFreshnessSample,
};

use crate::cleanup::{
    PerformedRecoveryCleanupRemoval, RecoveryCleanupDisposition, RecoveryCleanupTarget,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecoveryCleanupCounters {
    pub dispositions: u64,
    pub current: u64,
    pub retained: u64,
    pub quarantined_or_unsupported: u64,
    pub safely_removed: u64,
    pub eligible_after_cleanup: u64,
    pub actions_planned: u64,
    pub bytes_planned: u64,
    pub actions_attempted: u64,
    pub freshness_evaluations: u64,
    pub terminal_binding_evaluations: u64,
    pub freshness_reads_completed: u64,
    pub freshness_reads_denied: u64,
    pub freshness_bytes_read: u64,
    pub artifact_revalidation_reads_attempted: u64,
    pub artifact_revalidation_reads_completed: u64,
    pub artifact_revalidation_read_failures: u64,
    pub artifact_revalidation_mismatches: u64,
    pub artifact_revalidation_bytes_read: u64,
    pub actions_completed: u64,
    pub bytes_completed: u64,
    pub actions_deferred: u64,
    pub bytes_deferred: u64,
    pub denied_before_effect: u64,
    pub indeterminate_effects: u64,
    pub performed_effects: u64,
    pub cancellation_requests: u64,
    pub actions_cancelled: u64,
    pub bytes_cancelled: u64,
    pub scheduler_commands_submitted: u64,
    pub scheduler_commands_deferred: u64,
    pub scheduler_commands_cancelled: u64,
    pub scheduler_commands_settled: u64,
    pub freshness_scheduler_submitted: u64,
    pub freshness_scheduler_deferred: u64,
    pub freshness_scheduler_cancelled: u64,
    pub freshness_scheduler_settled: u64,
    pub removal_scheduler_submitted: u64,
    pub removal_scheduler_deferred: u64,
    pub removal_scheduler_cancelled: u64,
    pub removal_scheduler_settled: u64,
    pub live_media_handles_after_close: u64,
}

pub enum RecoveryCleanupDeferralEvidence {
    Freshness {
        target: RecoveryCleanupTarget,
        failure: StoreRecoveryCleanupFreshnessFailure,
    },
    PublishedGenerationChanged {
        target: RecoveryCleanupTarget,
        expected: u64,
        observed: u64,
    },
    EligibilityChanged {
        target: RecoveryCleanupTarget,
    },
    Cancelled {
        target: RecoveryCleanupTarget,
        settled_actions: u64,
    },
    CancellationBindingMismatch {
        target: RecoveryCleanupTarget,
    },
    DeniedBeforeEffect {
        target: RecoveryCleanupTarget,
        denial: PhysicalRecoveryCleanupRemovalDenial,
    },
    IndeterminateEffect {
        target: RecoveryCleanupTarget,
        evidence: PhysicalRecoveryCleanupRemovalIndeterminate,
    },
}

pub struct RecoveryCleanupEvidence {
    plan_identity: [u8; 32],
    published_generation: u64,
    dispositions: Box<[RecoveryCleanupDisposition]>,
    freshness: Box<[StoreRecoveryCleanupFreshnessSample]>,
    performed: Box<[PerformedRecoveryCleanupRemoval]>,
    deferrals: Box<[RecoveryCleanupDeferralEvidence]>,
    counters: RecoveryCleanupCounters,
}

pub(crate) struct RecoveryCleanupEvidenceParts {
    pub(crate) plan_identity: [u8; 32],
    pub(crate) published_generation: u64,
    pub(crate) dispositions: Box<[RecoveryCleanupDisposition]>,
    pub(crate) freshness: Box<[StoreRecoveryCleanupFreshnessSample]>,
    pub(crate) performed: Box<[PerformedRecoveryCleanupRemoval]>,
    pub(crate) deferrals: Box<[RecoveryCleanupDeferralEvidence]>,
    pub(crate) counters: RecoveryCleanupCounters,
}

pub enum RecoveryCleanupPosture {
    Complete(RecoveryCleanupEvidence),
    Deferred(RecoveryCleanupEvidence),
}

impl RecoveryCleanupEvidence {
    pub(crate) fn new(parts: RecoveryCleanupEvidenceParts) -> Self {
        let RecoveryCleanupEvidenceParts {
            plan_identity,
            published_generation,
            dispositions,
            freshness,
            performed,
            deferrals,
            counters,
        } = parts;
        Self {
            plan_identity,
            published_generation,
            dispositions,
            freshness,
            performed,
            deferrals,
            counters,
        }
    }

    pub const fn plan_identity(&self) -> [u8; 32] {
        self.plan_identity
    }

    pub const fn published_generation(&self) -> u64 {
        self.published_generation
    }

    pub fn dispositions(&self) -> &[RecoveryCleanupDisposition] {
        &self.dispositions
    }

    pub fn freshness_samples(&self) -> &[StoreRecoveryCleanupFreshnessSample] {
        &self.freshness
    }

    pub fn performed_removals(&self) -> &[PerformedRecoveryCleanupRemoval] {
        &self.performed
    }

    pub fn deferrals(&self) -> &[RecoveryCleanupDeferralEvidence] {
        &self.deferrals
    }

    pub const fn counters(&self) -> RecoveryCleanupCounters {
        self.counters
    }
}

impl RecoveryCleanupPosture {
    /// Exact owned slice backing, nested residue names and framed freshness
    /// reads. The posture/evidence value itself remains inline in the caller.
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let evidence = self.evidence();
        let mut bytes = [
            std::mem::size_of_val(&*evidence.dispositions),
            std::mem::size_of_val(&*evidence.freshness),
            std::mem::size_of_val(&*evidence.performed),
            std::mem::size_of_val(&*evidence.deferrals),
        ]
        .into_iter()
        .try_fold(0_u64, |sum, size| {
            sum.checked_add(u64::try_from(size).ok()?)
        })?;
        for disposition in &evidence.dispositions {
            bytes = bytes.checked_add(target_heap_bytes(disposition.target())?)?;
        }
        for sample in &evidence.freshness {
            bytes = bytes.checked_add(u64::try_from(sample.selector_read().bytes().len()).ok()?)?;
        }
        for deferral in &evidence.deferrals {
            bytes = bytes.checked_add(target_heap_bytes(deferral.target())?)?;
            if let RecoveryCleanupDeferralEvidence::Freshness { failure, .. } = deferral {
                if let Some(sample) = failure.sample() {
                    bytes = bytes
                        .checked_add(u64::try_from(sample.selector_read().bytes().len()).ok()?)?;
                }
                if let Some(read) = failure.read().and_then(|read| read.completed()) {
                    bytes = bytes.checked_add(u64::try_from(read.bytes().len()).ok()?)?;
                }
            }
        }
        Some(bytes)
    }

    pub(crate) fn from_evidence(evidence: RecoveryCleanupEvidence) -> Self {
        if evidence.counters.actions_deferred == 0 {
            Self::Complete(evidence)
        } else {
            Self::Deferred(evidence)
        }
    }

    pub const fn evidence(&self) -> &RecoveryCleanupEvidence {
        match self {
            Self::Complete(evidence) | Self::Deferred(evidence) => evidence,
        }
    }

    pub const fn is_deferred(&self) -> bool {
        matches!(self, Self::Deferred(_))
    }
}

fn target_heap_bytes(target: &RecoveryCleanupTarget) -> Option<u64> {
    match target {
        RecoveryCleanupTarget::Residue { name, .. } => u64::try_from(name.len()).ok(),
        RecoveryCleanupTarget::Record(_)
        | RecoveryCleanupTarget::Checkpoint(_)
        | RecoveryCleanupTarget::Wal(_) => Some(0),
    }
}

impl RecoveryCleanupDeferralEvidence {
    pub const fn target(&self) -> &RecoveryCleanupTarget {
        match self {
            Self::Freshness { target, .. }
            | Self::PublishedGenerationChanged { target, .. }
            | Self::EligibilityChanged { target }
            | Self::Cancelled { target, .. }
            | Self::CancellationBindingMismatch { target }
            | Self::DeniedBeforeEffect { target, .. }
            | Self::IndeterminateEffect { target, .. } => target,
        }
    }
}
