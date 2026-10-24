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
            domain_reason: self.domain_reason,
            domain_value: self.domain_value.clone(),
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
            && self.domain_reason == other.domain_reason
            && self.domain_value == other.domain_value
            && self.readmission_failure == other.readmission_failure
            && self.recovery_posture == other.recovery_posture
    }
}
impl Eq for WorthQueryOutputDemandDenial {}
