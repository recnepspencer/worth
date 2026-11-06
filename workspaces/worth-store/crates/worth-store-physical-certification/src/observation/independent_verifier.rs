use worth_foundational::CanonicalDigestId;
use worth_store_offline_verifier::OfflineVerifierBoundarySeam;
use worth_store_offline_verifier::RecoveryObserverReport;
use worth_store_recovery_runtime::{RecoveryReportEnvelope, RecoveryReportOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndependentVerifierObservationKind {
    Agreement,
    Disagreement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndependentVerifierObservation {
    seam: OfflineVerifierBoundarySeam,
    kind: IndependentVerifierObservationKind,
    artifact_set_digest: CanonicalDigestId,
}

impl IndependentVerifierObservation {
    pub fn from_reports(
        runtime: &RecoveryReportEnvelope,
        observer: RecoveryObserverReport,
    ) -> Self {
        Self {
            seam: OfflineVerifierBoundarySeam::RuntimeVerifierComparison,
            kind: if runtime.outcome() == RecoveryReportOutcome::Recovered
                && observer.artifact_count() > 0
            {
                IndependentVerifierObservationKind::Agreement
            } else {
                IndependentVerifierObservationKind::Disagreement
            },
            artifact_set_digest: CanonicalDigestId::new(observer.artifact_set_digest()),
        }
    }

    pub const fn seam(&self) -> OfflineVerifierBoundarySeam {
        self.seam
    }

    pub const fn kind(&self) -> IndependentVerifierObservationKind {
        self.kind
    }

    pub const fn artifact_set_digest(&self) -> [u8; 32] {
        *self.artifact_set_digest.bytes()
    }
}
