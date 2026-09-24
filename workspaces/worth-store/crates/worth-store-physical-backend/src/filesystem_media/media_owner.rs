use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use worth_store_physical_format::store_namespace::StoreNamespaceRelativeRole;

use super::{
    AdmittedStoreNamespace, ArtifactFamilyDirectory, MediaOwnerIdentity, MutationOwnerObservation,
    MutationOwnershipDenial, MutationOwnershipLease, NamespaceConfinementDenial, StagingDirectory,
};

#[path = "media_owner_admission.rs"]
mod admission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemMediaOwnerAdmissionDenial {
    Confinement(NamespaceConfinementDenial),
    Ownership(MutationOwnershipDenial),
}

/// Sealed predecessor authority. Phase 8 is responsible for minting the first
/// production value after root-specific capability qualification succeeds.
#[derive(Debug)]
pub struct FilesystemMediaAdmissionAuthority {
    _private: (),
}

impl FilesystemMediaAdmissionAuthority {
    #[cfg(test)]
    pub(super) const fn for_test() -> Self {
        Self { _private: () }
    }
}

/// Concrete owner of one confined local-filesystem namespace and its live
/// process mutation lease.
#[derive(Debug)]
pub struct FilesystemMediaOwner {
    boundary: super::fault_interposition::MediaFaultInterposer,
    namespace: AdmittedStoreNamespace,
    namespace_directory: super::NamespaceDirectoryHandle,
    families: ArtifactFamilyDirectory,
    staging: StagingDirectory,
    namespace_mutation_sequence: std::sync::Mutex<()>,
    next_file_handle_generation: AtomicU64,
    next_operation_identity: AtomicU64,
    file_mutation_sequences: super::file_mutation_sequence::FileMutationSequences,
    artifact_mutations: Arc<super::artifact_mutation_coordinator::ArtifactMutationCoordinator>,
    store_root_publication_required: AtomicBool,
    root_parent_publication_required: AtomicBool,
    mutation_lease: MutationOwnershipLease,
    #[cfg(feature = "certification-test-authority")]
    fail_next_removal_directory_sync: AtomicBool,
}

pub(super) struct ObservedMediaOwnerAdmissionFailure {
    pub(super) denial: FilesystemMediaOwnerAdmissionDenial,
    pub(super) counters: super::MediaCounterSnapshot,
    pub(super) effect_fate: super::owner_admission_effect::MediaOwnerAdmissionEffectFate,
    pub(super) release: Option<super::OwnershipReleaseOutcome>,
}

impl ObservedMediaOwnerAdmissionFailure {
    pub(super) const fn effect_possible(&self) -> bool {
        self.effect_fate.effect_possible()
    }
}

impl FilesystemMediaOwner {
    pub const fn identity(&self) -> MediaOwnerIdentity {
        self.namespace.owner_identity()
    }

    pub fn mutation_owner(&self) -> MutationOwnerObservation {
        self.mutation_lease.observation()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_next_removal_directory_sync(&self) {
        self.fail_next_removal_directory_sync
            .store(true, Ordering::Relaxed);
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn take_removal_directory_sync_failure(&self) -> bool {
        self.fail_next_removal_directory_sync
            .swap(false, Ordering::Relaxed)
    }

    pub(super) fn begin_mutation(
        &self,
    ) -> Result<super::mutation_ownership::MutationAuthority<'_>, FilesystemMediaOwnerAdmissionDenial>
    {
        if self.mutation_lease.belongs_to(self.identity()) {
            Ok(super::mutation_ownership::MutationAuthority::new(
                self.identity(),
                &self.mutation_lease,
            ))
        } else {
            Err(FilesystemMediaOwnerAdmissionDenial::Ownership(
                MutationOwnershipDenial::OwnershipLost,
            ))
        }
    }

    /// Atomically closes admission to new mutations after the backend observes
    /// that ownership can no longer be trusted. An operation that already
    /// acquired authority remains ordered before this invalidation.
    pub fn invalidate_mutation_authority(&self) {
        self.mutation_lease.invalidate();
    }

    pub fn close(self) -> super::OwnershipReleaseOutcome {
        let Self {
            boundary,
            namespace,
            namespace_directory,
            families,
            staging,
            mutation_lease,
            file_mutation_sequences,
            artifact_mutations,
            ..
        } = self;
        drop((
            artifact_mutations,
            file_mutation_sequences,
            staging,
            families,
            namespace_directory,
            namespace,
        ));
        mutation_lease.release(&boundary)
    }

    pub const fn namespace_directory(&self) -> &super::NamespaceDirectoryHandle {
        &self.namespace_directory
    }

    pub const fn families(&self) -> &ArtifactFamilyDirectory {
        &self.families
    }

    pub const fn staging(&self) -> &StagingDirectory {
        &self.staging
    }

    pub(super) fn issue_file_handle_identity(
        &self,
    ) -> Result<super::MediaHandleIdentity, FilesystemMediaOwnerAdmissionDenial> {
        let generation = self
            .next_file_handle_generation
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| {
                FilesystemMediaOwnerAdmissionDenial::Confinement(
                    NamespaceConfinementDenial::structural(
                        super::NamespaceConfinementDenialKind::AuthorityIdentityUnavailable,
                    ),
                )
            })?;
        Ok(super::MediaHandleIdentity::new(self.identity(), generation))
    }

    pub(super) fn issue_operation_identity(&self) -> Option<super::MediaOperationIdentity> {
        let current = self
            .next_operation_identity
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .ok()?;
        Some(super::MediaOperationIdentity::new(current))
    }

    pub(super) const fn boundary(&self) -> &super::fault_interposition::MediaFaultInterposer {
        &self.boundary
    }

    pub fn counters(&self) -> super::MediaCounterSnapshot {
        self.boundary.counters().snapshot()
    }

    pub fn counter_observer(&self) -> super::MediaCounterObserver {
        super::MediaCounterObserver::new(Arc::clone(self.boundary.shared_counters()))
    }

    pub(super) const fn root_directory_handle(&self) -> &super::NamespaceDirectoryHandle {
        self.namespace.root_handle()
    }

    pub(super) const fn root_parent_directory(&self) -> Option<&cap_std::fs::Dir> {
        self.namespace.publication_parent()
    }

    pub(super) fn store_root_publication_required(&self) -> bool {
        self.store_root_publication_required
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub(super) fn root_parent_publication_required(&self) -> bool {
        self.root_parent_publication_required
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub(super) fn mark_store_root_published(&self) {
        self.store_root_publication_required
            .store(false, std::sync::atomic::Ordering::Release);
    }

    pub(super) fn mark_root_parent_published(&self) {
        self.root_parent_publication_required
            .store(false, std::sync::atomic::Ordering::Release);
    }

    pub fn identity_record_path(&self) -> super::NamespaceRelativePath {
        super::NamespaceRelativePath::bind_role(
            self.identity(),
            StoreNamespaceRelativeRole::IdentityRecord,
        )
    }

    pub fn staged_identity_path(
        &self,
        name: &worth_store_physical_format::store_namespace::StagedNamespaceName,
    ) -> super::StagedNamespacePath {
        super::StagedNamespacePath::new(super::NamespaceRelativePath::bind_staged_identity(
            self.identity(),
            name,
        ))
    }

    pub fn identity_publication_target(&self) -> super::NamespacePublicationTarget {
        super::NamespacePublicationTarget::new(self.identity_record_path())
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(super) fn staged_publication_target(
        &self,
        name: &worth_store_physical_format::store_namespace::StagedNamespaceName,
    ) -> super::NamespacePublicationTarget {
        super::NamespacePublicationTarget::new(super::NamespaceRelativePath::bind_staged_identity(
            self.identity(),
            name,
        ))
    }

    pub(super) fn mutation_sequence_for(
        &self,
        file: &std::fs::File,
    ) -> std::io::Result<super::file_mutation_sequence::FileMutationSequence> {
        self.file_mutation_sequences.for_file(file)
    }

    pub(super) fn begin_namespace_mutation(
        &self,
    ) -> Result<
        super::mutation_ownership::CoordinatedNamespaceMutation<'_>,
        FilesystemMediaOwnerAdmissionDenial,
    > {
        let ownership = self.begin_mutation()?;
        let sequence = self
            .namespace_mutation_sequence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(super::mutation_ownership::CoordinatedNamespaceMutation::new(ownership, sequence))
    }

    pub(super) fn begin_artifact_mutation(
        &self,
        paths: Vec<String>,
    ) -> Result<
        super::artifact_mutation_coordinator::CoordinatedArtifactMutation<'_>,
        FilesystemMediaOwnerAdmissionDenial,
    > {
        let ownership = self.begin_mutation()?;
        Ok(self.artifact_mutations.coordinate(ownership, paths))
    }

    pub(super) fn begin_artifact_namespace_mutation(
        &self,
        paths: Vec<String>,
    ) -> Result<
        super::artifact_mutation_coordinator::CoordinatedArtifactNamespaceMutation<'_>,
        FilesystemMediaOwnerAdmissionDenial,
    > {
        let namespace = self.begin_namespace_mutation()?;
        Ok(self
            .artifact_mutations
            .coordinate_namespace(namespace, paths))
    }

    #[cfg(test)]
    pub(super) fn wait_until_artifact_mutation_is_contended(&self) {
        self.artifact_mutations.wait_until_contended();
    }

    pub(super) fn require_owned_directory(
        &self,
        directory: &super::NamespaceDirectoryHandle,
    ) -> Result<(), NamespaceConfinementDenial> {
        self.namespace.require_owned_directory(directory)
    }

    pub(super) fn directory_for_path(
        &self,
        path: &super::NamespaceRelativePath,
    ) -> Option<&super::NamespaceDirectoryHandle> {
        if path.owner_identity() != self.identity() {
            return None;
        }
        match path.parent() {
            super::namespace_confinement::NamespaceParent::Root => {
                Some(self.root_directory_handle())
            }
            super::namespace_confinement::NamespaceParent::Namespace => {
                Some(self.namespace_directory())
            }
            super::namespace_confinement::NamespaceParent::Families => {
                Some(self.families().handle())
            }
            super::namespace_confinement::NamespaceParent::Staging => Some(self.staging().handle()),
        }
    }
}
