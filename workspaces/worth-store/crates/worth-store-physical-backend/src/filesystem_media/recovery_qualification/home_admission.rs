//! Lease-first home admission: existing identity is not synchronized or rewritten.
use super::*;
use crate::recovery_media::RecoveryStorePresence;

pub(crate) fn qualify_home_recovery(
    request: super::super::FilesystemQualificationRequest,
) -> Result<QualifiedRecoveryParts, RecoveryFilesystemQualificationError> {
    if request.access.admitted_contract().is_none() {
        return Err(RecoveryFilesystemQualificationError::BackendProfileUnsupported);
    }
    qualify_recovery_with_schedule(
        &request.root,
        request.fault_schedule,
        NamespaceAdmissionOrigin::Home,
        request.expected_basis,
    )
}

impl QualifiedRecoveryParts {
    pub(crate) fn store_presence(
        &self,
    ) -> Result<RecoveryStorePresence, RecoveryFilesystemQualificationError> {
        let has_entries = |directory: &super::super::NamespaceDirectoryHandle| {
            super::super::artifact_tree_effects::directory_has_entries(
                &self.owner,
                directory.directory(),
            )
            .map_err(|_| RecoveryFilesystemQualificationError::RootUnavailable)
        };
        let lock_name =
            worth_store_physical_format::store_namespace::StoreNamespaceRelativeRole::MutationLock
                .components()
                .last()
                .expect("the mutation lock has a fixed name");
        let namespace_has_store = super::super::artifact_tree_effects::directory_has_other_entry(
            &self.owner,
            self.owner.namespace_directory().directory(),
            lock_name,
        )
        .map_err(|_| RecoveryFilesystemQualificationError::RootUnavailable)?;
        Ok(
            if namespace_has_store
                || has_entries(self.owner.families().handle())?
                || has_entries(self.owner.staging().handle())?
            {
                RecoveryStorePresence::ExistingArtifacts
            } else {
                RecoveryStorePresence::ProvenAbsent
            },
        )
    }
    pub(crate) fn qualify_serving(
        self,
        request: super::super::FilesystemQualificationRequest,
    ) -> super::super::AdmittedFilesystemMedia {
        let mut owner = self.owner;
        owner.enter_serving_boundary(&request);
        FilesystemMediaOwner::qualify_held(request, owner)
    }
}
