use super::super::observer_evidence_summary::RecoveryObserverEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverConclusion {
    artifact_set_digest: [u8; 32],
    evidence: RecoveryObserverEvidence,
}

impl RecoveryObserverConclusion {
    pub(crate) const fn new(
        artifact_set_digest: [u8; 32],
        evidence: RecoveryObserverEvidence,
    ) -> Self {
        Self {
            artifact_set_digest,
            evidence,
        }
    }

    pub(crate) const fn artifact_set_digest(self) -> [u8; 32] {
        self.artifact_set_digest
    }

    pub(crate) const fn evidence(self) -> RecoveryObserverEvidence {
        self.evidence
    }
}
