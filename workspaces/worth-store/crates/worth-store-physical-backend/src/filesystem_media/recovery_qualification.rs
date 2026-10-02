use std::path::Path;
use std::sync::Arc;

use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{
    FilesystemAccessPosture, FilesystemMediaOwner, FilesystemMediaOwnerAdmissionDenial,
    MediaFaultSchedule, MutationOwnershipDenial, RootProfileQualificationReport,
};
use crate::recovery_media::{
    PhysicalRecoveryMediaGeneration, QualifiedPhysicalBackendProfile,
    RecoveryFilesystemQualificationError,
};

pub(crate) struct QualifiedRecoveryParts {
    owner: FilesystemMediaOwner,
    observed_profile: super::FilesystemBackendProfile,
    backend_profile: QualifiedPhysicalBackendProfile,
    media_generation: PhysicalRecoveryMediaGeneration,
}

pub struct AdmittedRecoveryParts {
    pub(crate) owner: FilesystemMediaOwner,
    pub(crate) execution_capability: crate::AdmittedBackendCapabilityWitness,
    pub(crate) store_identity: StableStoreIdentity,
    pub(crate) media_generation: PhysicalRecoveryMediaGeneration,
    pub(crate) backend_profile: QualifiedPhysicalBackendProfile,
}

pub(crate) fn qualify_existing_recovery(
    root: &Path,
) -> Result<QualifiedRecoveryParts, RecoveryFilesystemQualificationError> {
    qualify_existing_recovery_with_schedule(root, MediaFaultSchedule::default())
}

#[cfg(feature = "certification-test-authority")]
pub(crate) fn qualify_existing_recovery_for_certification(
    root: &Path,
    schedule: MediaFaultSchedule,
) -> Result<QualifiedRecoveryParts, RecoveryFilesystemQualificationError> {
    qualify_existing_recovery_with_schedule(root, schedule)
}

fn qualify_existing_recovery_with_schedule(
    root: &Path,
    schedule: MediaFaultSchedule,
) -> Result<QualifiedRecoveryParts, RecoveryFilesystemQualificationError> {
    let counters = Arc::new(super::operation_counters::MediaCounterCells::default());
    let boundary =
        super::fault_interposition::MediaFaultInterposer::new(schedule, Arc::clone(&counters));
    let access = FilesystemAccessPosture::CoordinatedServiceAccount
        .admitted_contract()
        .expect("coordinated access has one contract");
    let preflight = super::profile_observation::observe_admission_profile(root, &boundary)
        .map_err(|_| RecoveryFilesystemQualificationError::RootUnavailable)?;
    if super::profile_observation::deny_profile(&preflight, counters.snapshot()).is_some() {
        return Err(RecoveryFilesystemQualificationError::BackendProfileUnsupported);
    }
    let owner = FilesystemMediaOwner::admit_existing_with_boundary(root, boundary, counters)
        .map_err(map_owner_failure)?;
    finish_existing_qualification(root, owner, access)
}

fn finish_existing_qualification(
    root: &Path,
    owner: FilesystemMediaOwner,
    access: super::FilesystemAccessContract,
) -> Result<QualifiedRecoveryParts, RecoveryFilesystemQualificationError> {
    let profile = match super::profile_observation::observe_profile(
        owner.root_directory_handle(),
        owner.boundary(),
    ) {
        Ok(profile) => profile,
        Err(_) => {
            return Err(close_with(
                owner,
                RecoveryFilesystemQualificationError::RootUnavailable,
            ))
        }
    };
    if super::namespace_admission::require_opened_root_identity(
        root,
        owner.root_directory_handle().directory(),
        owner.boundary(),
    )
    .is_err()
    {
        return Err(close_with(
            owner,
            RecoveryFilesystemQualificationError::RootIdentityChanged,
        ));
    }
    if super::profile_observation::deny_profile(&profile, owner.counters()).is_some() {
        return Err(close_with(
            owner,
            RecoveryFilesystemQualificationError::BackendProfileUnsupported,
        ));
    }
    let binding = super::profile_observation::profile_binding(&profile, access);
    let report = RootProfileQualificationReport::new(binding);
    let media_generation = PhysicalRecoveryMediaGeneration::from_owner_attempt(
        owner.mutation_owner().attempt().bytes(),
    );
    Ok(QualifiedRecoveryParts {
        owner,
        observed_profile: profile,
        backend_profile: QualifiedPhysicalBackendProfile::from_report(&report),
        media_generation,
    })
}

impl QualifiedRecoveryParts {
    pub(crate) fn root_ownership_identity(&self) -> super::MediaOwnerIdentity {
        self.owner.identity()
    }

    pub(crate) fn backend_profile(&self) -> &QualifiedPhysicalBackendProfile {
        &self.backend_profile
    }

    pub(crate) const fn media_generation(&self) -> PhysicalRecoveryMediaGeneration {
        self.media_generation
    }

    pub(crate) fn admit_persisted_store(
        self,
    ) -> Result<AdmittedRecoveryParts, RecoveryFilesystemQualificationError> {
        let identity =
            super::namespace_identity_admission::admit_existing_store_identity(&self.owner)
                .map_err(|_| RecoveryFilesystemQualificationError::PersistedIdentityUnavailable)?;
        let (execution_capability, ..) = super::capability_qualification::qualify_backend_claims(
            &self.owner,
            &identity,
            &self.observed_profile,
        )
        .map_err(|_| RecoveryFilesystemQualificationError::BackendCapabilityUnavailable)?;
        self.owner.boundary().bind_store(identity.stable_identity());
        Ok(AdmittedRecoveryParts {
            owner: self.owner,
            execution_capability,
            store_identity: identity.stable_identity(),
            media_generation: self.media_generation,
            backend_profile: self.backend_profile,
        })
    }

    pub(crate) fn recovery_effect_count(&self) -> u64 {
        recovery_effect_count(self.owner.counters())
    }

    #[cfg(test)]
    pub(crate) fn close(self) -> super::OwnershipReleaseOutcome {
        self.owner.close()
    }
}

fn map_owner_failure(
    failure: super::media_owner::ObservedMediaOwnerAdmissionFailure,
) -> RecoveryFilesystemQualificationError {
    match failure.denial {
        FilesystemMediaOwnerAdmissionDenial::Ownership(MutationOwnershipDenial::Contended) => {
            RecoveryFilesystemQualificationError::OwnershipContended
        }
        FilesystemMediaOwnerAdmissionDenial::Confinement(_) => {
            RecoveryFilesystemQualificationError::ExistingStoreRequired
        }
        _ => RecoveryFilesystemQualificationError::OwnershipUnavailable,
    }
}

fn close_with(
    owner: FilesystemMediaOwner,
    error: RecoveryFilesystemQualificationError,
) -> RecoveryFilesystemQualificationError {
    let _ = owner.close();
    error
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    use super::{qualify_existing_recovery, RecoveryFilesystemQualificationError};
    use crate::filesystem_media::{
        namespace_identity_admission, FilesystemMediaAdmissionAuthority, FilesystemMediaOwner,
    };

    #[test]
    fn absent_and_incomplete_roots_are_refused_without_creation() {
        let parent = tempfile::tempdir().expect("test parent");
        let absent = parent.path().join("absent");
        let denial = qualify_existing_recovery(&absent)
            .err()
            .expect("absent root must be refused");
        assert_eq!(
            denial,
            RecoveryFilesystemQualificationError::ExistingStoreRequired
        );
        assert!(!absent.exists());

        let incomplete = parent.path().join("incomplete");
        std::fs::create_dir(&incomplete).expect("incomplete root");
        let denial = qualify_existing_recovery(&incomplete)
            .err()
            .expect("incomplete root must be refused");
        assert_eq!(
            denial,
            RecoveryFilesystemQualificationError::ExistingStoreRequired
        );
        assert_eq!(
            std::fs::read_dir(&incomplete)
                .expect("incomplete root remains")
                .count(),
            0
        );
    }

    #[test]
    fn qualification_is_exclusive_and_persisted_identity_joins_afterward() {
        let parent = tempfile::tempdir().expect("test parent");
        let root = parent.path().join("store");
        let expected_store = initialize_store(&root);
        let qualified = qualify_existing_recovery(&root).expect("qualified existing store");
        let denial = qualify_existing_recovery(&root)
            .err()
            .expect("second recovery owner must contend");
        assert_eq!(
            denial,
            RecoveryFilesystemQualificationError::OwnershipContended
        );
        let admitted = qualified
            .admit_persisted_store()
            .expect("persisted identity admission");
        assert_eq!(admitted.store_identity, expected_store);
        let _ = admitted.owner.close();
    }

    #[test]
    fn persisted_identity_is_read_after_the_exclusive_lease_is_acquired() {
        let parent = tempfile::tempdir().expect("test parent");
        let root = parent.path().join("store");
        let initial_store = initialize_store(&root);
        let qualified = qualify_existing_recovery(&root).expect("qualified existing store");
        let replacement = ProposedStoreIdentity::from_nonzero_bytes([93; 16])
            .expect("nonzero replacement identity");
        let replacement_record =
            StoreNamespaceIdentityRecord::new(StoreNamespaceVersion::CURRENT, replacement);

        std::fs::write(
            root.join("namespace").join("identity"),
            replacement_record.encode(),
        )
        .expect("adversarial identity replacement after lease acquisition");
        let admitted = qualified
            .admit_persisted_store()
            .expect("persisted identity admission");

        assert_ne!(admitted.store_identity, initial_store);
        assert_eq!(
            admitted.store_identity,
            replacement_record.published_identity()
        );
        let _ = admitted.owner.close();
    }

    #[test]
    fn profile_is_stable_while_each_lease_has_a_new_media_generation() {
        let parent = tempfile::tempdir().expect("test parent");
        let root = parent.path().join("store");
        initialize_store(&root);
        let first = qualify_existing_recovery(&root).expect("first qualification");
        let first_profile = first.backend_profile().clone();
        let first_generation = first.media_generation();
        let _ = first.close();
        let second = qualify_existing_recovery(&root).expect("second qualification");
        assert_eq!(second.backend_profile(), &first_profile);
        assert_ne!(second.media_generation(), first_generation);
        let _ = second.close();
    }

    fn initialize_store(
        root: &std::path::Path,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        let owner =
            FilesystemMediaOwner::admit(root, FilesystemMediaAdmissionAuthority::for_test())
                .expect("ordinary media owner");
        let identity = namespace_identity_admission::admit_store_identity(&owner)
            .expect("persisted identity")
            .stable_identity();
        let _ = owner.close();
        identity
    }
}

#[cfg(test)]
mod effect_tests;

impl AdmittedRecoveryParts {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(crate) fn artifact_tree(&self) -> crate::filesystem_media::ArtifactTreeMedia<'_> {
        crate::filesystem_media::ArtifactTreeMedia::for_recovery(
            &self.owner,
            self.store_identity,
            &self.execution_capability,
        )
    }

    pub(crate) fn recovery_effect_count(&self) -> u64 {
        recovery_effect_count(self.owner.counters())
    }
}

fn recovery_effect_count(counters: super::MediaCounterSnapshot) -> u64 {
    counters
        .file_creates()
        .saturating_add(counters.file_syncs())
        .saturating_add(counters.directory_syncs())
        .saturating_add(counters.replacements())
        .saturating_add(counters.deletions())
        .saturating_add(counters.cleanup_actions())
        .saturating_add(
            counters.completed_operations_for(super::MediaOperationRole::PositionedWrite),
        )
        .saturating_add(counters.completed_operations_for(super::MediaOperationRole::Append))
        .saturating_add(counters.completed_operations_for(super::MediaOperationRole::Truncate))
        .saturating_add(counters.completed_operations_for(super::MediaOperationRole::Allocate))
        .saturating_add(
            counters.completed_operations_for(super::MediaOperationRole::CreateDirectory),
        )
        .saturating_add(counters.partial_effects())
        .saturating_add(counters.indeterminate_effects())
}
