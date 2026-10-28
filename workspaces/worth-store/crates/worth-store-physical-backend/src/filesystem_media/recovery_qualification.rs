mod home_admission;
pub(crate) use home_admission::qualify_home_recovery;
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
    binding: super::qualification_basis::RootProfileBinding,
    backend_profile: QualifiedPhysicalBackendProfile,
    media_generation: PhysicalRecoveryMediaGeneration,
}

pub struct AdmittedRecoveryParts {
    pub(crate) owner: FilesystemMediaOwner,
    pub(crate) execution_capability: crate::AdmittedBackendCapabilityWitness,
    pub(crate) store_identity: StableStoreIdentity,
    pub(crate) media_generation: PhysicalRecoveryMediaGeneration,
    pub(crate) backend_profile: QualifiedPhysicalBackendProfile,
    binding: super::qualification_basis::RootProfileBinding,
}

pub(crate) fn qualify_existing_recovery(
    root: &Path,
) -> Result<QualifiedRecoveryParts, RecoveryFilesystemQualificationError> {
    qualify_recovery_with_schedule(
        root,
        MediaFaultSchedule::default(),
        NamespaceAdmissionOrigin::Existing,
        None,
    )
}

#[cfg(feature = "certification-test-authority")]
pub(crate) fn qualify_existing_recovery_for_certification(
    root: &Path,
    schedule: MediaFaultSchedule,
) -> Result<QualifiedRecoveryParts, RecoveryFilesystemQualificationError> {
    qualify_recovery_with_schedule(root, schedule, NamespaceAdmissionOrigin::Existing, None)
}

#[derive(Clone, Copy)]
enum NamespaceAdmissionOrigin {
    Existing,
    Home,
}

fn qualify_recovery_with_schedule(
    root: &Path,
    schedule: MediaFaultSchedule,
    origin: NamespaceAdmissionOrigin,
    expected: Option<super::qualification_basis::RootProfileBinding>,
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
    if let Some(expected) = &expected {
        let observed = super::profile_observation::profile_binding(&preflight, access);
        if super::qualification_basis_drift::basis_drift(expected, &observed).is_some() {
            return Err(RecoveryFilesystemQualificationError::RootIdentityChanged);
        }
    }
    let owner = match origin {
        NamespaceAdmissionOrigin::Existing => {
            FilesystemMediaOwner::admit_existing_with_boundary(root, boundary, counters)
        }
        NamespaceAdmissionOrigin::Home => {
            FilesystemMediaOwner::admit_with_boundary(root, boundary, counters)
        }
    }
    .map_err(map_owner_failure)?;
    let parts = finish_existing_qualification(root, owner, access)?;
    if let Some(expected) = &expected {
        if super::qualification_basis_drift::basis_drift(expected, &parts.binding).is_some() {
            return Err(RecoveryFilesystemQualificationError::RootIdentityChanged);
        }
    }
    Ok(parts)
}

pub(super) fn finish_existing_qualification(
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
    let report = RootProfileQualificationReport::new(binding.clone());
    let media_generation = PhysicalRecoveryMediaGeneration::from_owner_attempt(
        owner.mutation_owner().attempt().bytes(),
    );
    Ok(QualifiedRecoveryParts {
        owner,
        observed_profile: profile,
        binding,
        backend_profile: QualifiedPhysicalBackendProfile::from_report(&report),
        media_generation,
    })
}

impl QualifiedRecoveryParts {
    pub(crate) fn verify_root(
        &self,
        root: &Path,
    ) -> Result<(), RecoveryFilesystemQualificationError> {
        super::namespace_admission::require_opened_root_identity(
            root,
            self.owner.root_directory_handle().directory(),
            self.owner.boundary(),
        )
        .map_err(|_| RecoveryFilesystemQualificationError::RootIdentityChanged)
    }

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
            binding: self.binding,
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
mod tests;

#[cfg(test)]
mod effect_tests;

impl AdmittedRecoveryParts {
    /// The durability admission basis ordinary open derives from this same
    /// media, so a policy declared over it has the identity Serving binds.
    #[cfg(feature = "recovery-runtime-owner")]
    pub(crate) fn physical_durability_admission_basis(
        &self,
    ) -> Result<crate::PhysicalDurabilityAdmissionBasis, crate::BackendCapabilityAdmissionDenial>
    {
        super::durability_admission::durability_admission_basis(
            self.store_identity,
            &self.binding,
            &self.execution_capability,
        )
    }

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
