use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{BoundedRecoveryFilesystemDiscovery, RecoveryFilesystemQualificationError};

pub struct AdmittedRecoveryFilesystemMedia {
    pub(super) parts: crate::filesystem_media::recovery_qualification::AdmittedRecoveryParts,
    pub(super) discovery_incarnations_issued: u64,
    #[cfg(feature = "certification-test-authority")]
    certification_cleanup_handle:
        Option<crate::filesystem_media::CertificationRetainedMediaFileHandle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryMediaHandleObservation {
    live_file_handles: u64,
    live_directory_handles: u64,
}

impl AdmittedRecoveryFilesystemMedia {
    pub(crate) const fn from_parts(
        parts: crate::filesystem_media::recovery_qualification::AdmittedRecoveryParts,
    ) -> Self {
        Self::from_discovery(parts, 0)
    }

    pub(super) const fn from_discovery(
        parts: crate::filesystem_media::recovery_qualification::AdmittedRecoveryParts,
        discovery_incarnations_issued: u64,
    ) -> Self {
        Self {
            parts,
            discovery_incarnations_issued,
            #[cfg(feature = "certification-test-authority")]
            certification_cleanup_handle: None,
        }
    }

    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.parts.store_identity
    }

    pub const fn media_generation(&self) -> super::PhysicalRecoveryMediaGeneration {
        self.parts.media_generation
    }

    pub const fn backend_profile(&self) -> &super::QualifiedPhysicalBackendProfile {
        &self.parts.backend_profile
    }

    pub fn recovery_effect_count(&self) -> u64 {
        self.parts.recovery_effect_count()
    }

    pub fn handle_observation(&self) -> RecoveryMediaHandleObservation {
        RecoveryMediaHandleObservation::from_owner(&self.parts.owner)
    }

    /// Creates one real command-scoped backend file handle and deliberately
    /// retains it so certification can prove cleanup closeout observes media
    /// ownership rather than a runtime-supplied counter.
    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_retain_cleanup_media_handle(&mut self) -> bool {
        if self.certification_cleanup_handle.is_some() {
            return false;
        }
        let path = self.parts.owner.identity_record_path();
        match self.parts.owner.open_existing(&path).into_result() {
            crate::filesystem_media::NamespaceFileOpenResult::Opened { handle, .. } => {
                self.certification_cleanup_handle = Some(handle.retain_for_certification());
                true
            }
            crate::filesystem_media::NamespaceFileOpenResult::Failed(_) => false,
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub fn scheduler_capability_claim(
        &self,
        kind: crate::BackendCapabilityKind,
        evidence: crate::CapabilityEvidenceClass,
    ) -> Result<crate::BackendCapabilityClaimWitness, crate::BackendCapabilityAdmissionDenial> {
        self.parts.execution_capability.require(kind, evidence)
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub fn mutation_owner_observation(&self) -> crate::MutationOwnerObservation {
        self.parts.owner.mutation_owner()
    }

    pub fn bounded_discovery(
        mut self,
        maximum_entries: u64,
        maximum_bytes: u64,
    ) -> Result<BoundedRecoveryFilesystemDiscovery, RecoveryFilesystemQualificationError> {
        self.discovery_incarnations_issued = self
            .discovery_incarnations_issued
            .checked_add(1)
            .ok_or(RecoveryFilesystemQualificationError::InvalidDiscoveryLimit)?;
        BoundedRecoveryFilesystemDiscovery::new(
            self.parts,
            self.discovery_incarnations_issued,
            maximum_entries,
            maximum_bytes,
        )
    }

    /// Validates one Store scheduler binding against the admitted backend
    /// capability and returns passive completion evidence. This performs no
    /// filesystem effect and grants no media authority.
    #[cfg(feature = "recovery-runtime-owner")]
    pub fn complete_recovery_queue_binding(
        &self,
        binding: crate::BackendQueueExecutionPlanBinding,
    ) -> Option<crate::BackendQueueExecutionCompletion> {
        crate::BackendQueueExecutionAuthority::store_owned()
            .issue_ticket(
                binding,
                &self.parts.execution_capability,
                crate::BackendQueueExecutionAdaptation::None,
            )
            .ok()
            .map(|ticket| ticket.begin_completion().observe_queue_depth(1).complete())
    }
}

impl RecoveryMediaHandleObservation {
    pub(crate) fn from_owner(owner: &crate::filesystem_media::FilesystemMediaOwner) -> Self {
        let counters = owner.counters();
        Self {
            live_file_handles: counters.live_file_handles(),
            live_directory_handles: counters.live_directory_handles(),
        }
    }

    pub const fn live_file_handles(self) -> u64 {
        self.live_file_handles
    }

    pub const fn live_directory_handles(self) -> u64 {
        self.live_directory_handles
    }

    pub const fn excess_over(self, baseline: Self) -> u64 {
        self.live_file_handles
            .saturating_sub(baseline.live_file_handles)
            .saturating_add(
                self.live_directory_handles
                    .saturating_sub(baseline.live_directory_handles),
            )
    }
}
