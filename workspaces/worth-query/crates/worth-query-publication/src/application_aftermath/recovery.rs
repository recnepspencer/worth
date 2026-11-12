use super::{WorthQueryPublishedAftermathPosture, WorthQueryPublishedCanonicalWork};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedRecoverySupportTruth {
    EvidenceBundle,
    CertificationSummary,
    ParityArtifact,
    DegradedRecoveryReport,
    StaleBasisDisclosure,
    TransientLifecycleEvidence,
    ResidualDebtStatement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedRecoveryDurability {
    ProcessLocal,
    DurableHome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedRecoverySupport {
    support_truth: WorthQueryPublishedRecoverySupportTruth,
    posture: WorthQueryPublishedAftermathPosture,
    durability: WorthQueryPublishedRecoveryDurability,
    inspection_work: WorthQueryPublishedCanonicalWork,
}

impl WorthQueryPublishedRecoverySupport {
    pub(super) const fn new(
        support_truth: WorthQueryPublishedRecoverySupportTruth,
        posture: WorthQueryPublishedAftermathPosture,
        durability: WorthQueryPublishedRecoveryDurability,
        inspection_work: WorthQueryPublishedCanonicalWork,
    ) -> Self {
        Self {
            support_truth,
            posture,
            durability,
            inspection_work,
        }
    }

    pub const fn support_truth(&self) -> WorthQueryPublishedRecoverySupportTruth {
        self.support_truth
    }

    pub const fn posture(&self) -> WorthQueryPublishedAftermathPosture {
        self.posture
    }

    pub const fn durability(&self) -> WorthQueryPublishedRecoveryDurability {
        self.durability
    }

    pub const fn inspection_work(&self) -> WorthQueryPublishedCanonicalWork {
        self.inspection_work
    }
}
