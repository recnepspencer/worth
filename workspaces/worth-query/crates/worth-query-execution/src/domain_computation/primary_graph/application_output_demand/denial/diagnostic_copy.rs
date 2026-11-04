//! Diagnostic copies preserve evidence without duplicating cleanup authority.
use super::WorthQueryOutputDemandDenial;

// Cloning is diagnostic only. The original refusal remains the unique owner.
impl Clone for WorthQueryOutputDemandDenial {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            evidence: self.evidence.clone(),
            cleanup: None,
            subject: self.subject.clone(),
            domain: self.domain.clone(),
            readmission_failure: self.readmission_failure,
            recovery_posture: self.recovery_posture,
        }
    }
}
impl PartialEq for WorthQueryOutputDemandDenial {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.evidence == other.evidence
            && self.subject == other.subject
            && self.domain == other.domain
            && self.readmission_failure == other.readmission_failure
            && self.recovery_posture == other.recovery_posture
    }
}
impl Eq for WorthQueryOutputDemandDenial {}
