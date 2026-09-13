use super::counters::OwnerSemanticVerificationCounters;
use crate::inspection::OwnerDecodedArtifactBinding;
use crate::truth_composition::RecoveryCandidateObservation;

pub(crate) struct OwnerSemanticVerificationResult {
    pub(crate) counters: OwnerSemanticVerificationCounters,
    pub(crate) recovery_candidates: Vec<RecoveryCandidateObservation>,
    pub(crate) owner_bindings: Vec<OwnerDecodedArtifactBinding>,
    pub(crate) peak_owned_allocation_bytes: u64,
}

pub(crate) struct OwnerSemanticVerificationResourceDenial {
    pub(crate) required_bytes: u64,
    pub(crate) limit_bytes: u64,
}

pub(crate) enum OwnerSemanticVerificationDenial {
    Resource(OwnerSemanticVerificationResourceDenial),
    AllocationFailed,
    Media(worth_store_physical_backend::OfflineMediaReadDenial),
    Inspection(crate::OfflineInspectionDenial),
}
