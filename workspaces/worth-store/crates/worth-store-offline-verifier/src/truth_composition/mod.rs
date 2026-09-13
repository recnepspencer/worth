mod candidate_evaluation;
mod integrity_posture;
mod interruptible_sort;
mod operational_region_composition;
mod operational_truth_report;
#[cfg(test)]
mod tests;
mod truth_evidence;
mod truth_region;
mod truth_report_identity;

pub use candidate_evaluation::{
    discover_recovery_candidates, ObservedRecoveryFrontier, RecoveryCandidate,
    RecoveryCandidateConfidence, RecoveryCandidateDiscoveryDenial, RecoveryCandidateObservation,
    RecoveryCandidateSet,
};
pub use integrity_posture::{OfflineIntegrityObservation, OfflineIntegrityPosture};
pub(crate) use operational_truth_report::compose_operational_truth_with_owner_candidates;
pub use operational_truth_report::{
    compose_operational_truth, CanonicalPhysicalCoverageProof, OperationalTruthCompositionBudget,
    OperationalTruthCompositionDenial, OperationalTruthReport,
};
pub use truth_evidence::{
    OfflineFileTruthEvidence, OfflineRecoveryAvailability, OfflineSecurityEvidencePosture,
    OfflineTruthEvidenceAdmissionDenial, OfflineTruthEvidenceSet,
};
pub use truth_region::{
    EvidenceBoundTruthRegion, OfflineAuthorityClass, OfflineTruthEvidenceReferences,
    OperationalTruthRegion,
};
