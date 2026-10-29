use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use worth_store_physical_format::store_namespace::{
    NamespaceEntryType, StoreNamespaceRelativeRole,
};

use super::{
    MediaOwnerIdentity, MediaPathRole, NamespaceConfinementDenial, NamespaceConfinementDenialKind,
    NamespaceDirectoryHandle, NamespaceRelativePath,
};

/// A root directory admitted once from ambient configuration and thereafter
/// accessed only through relative directory capabilities.
#[derive(Debug)]
pub struct AdmittedStoreNamespace {
    owner: MediaOwnerIdentity,
    root: NamespaceDirectoryHandle,
    publication_parent: Option<Dir>,
    store_root_publication_required: bool,
    root_parent_publication_required: bool,
    admission_effect_fate: super::owner_admission_effect::MediaOwnerAdmissionEffectFate,
    next_handle_generation: u64,
}

impl AdmittedStoreNamespace {
    pub const fn owner_identity(&self) -> MediaOwnerIdentity {
        self.owner
    }

    pub const fn root_handle(&self) -> &NamespaceDirectoryHandle {
        &self.root
    }

    pub(super) fn from_opened_directory(
        directory: Dir,
        boundary: &super::fault_interposition::MediaFaultInterposer,
    ) -> Result<Self, NamespaceConfinementDenial> {
        let owner = MediaOwnerIdentity::generate().map_err(|_| {
            NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::AuthorityIdentityUnavailable,
            )
        })?;
        boundary.bind_owner(owner);
        Ok(Self {
            owner,
            root: NamespaceDirectoryHandle::new(
                owner,
                1,
                MediaPathRole::ArtifactOwned,
                directory,
                boundary.shared_counters().directory_handle_opened(),
            ),
            publication_parent: None,
            store_root_publication_required: false,
            root_parent_publication_required: false,
            admission_effect_fate:
                super::owner_admission_effect::MediaOwnerAdmissionEffectFate::DeniedBeforeEffect,
            next_handle_generation: 2,
        })
    }

    pub(super) const fn admission_effect_fate(
        &self,
    ) -> super::owner_admission_effect::MediaOwnerAdmissionEffectFate {
        self.admission_effect_fate
    }

    pub(super) fn complete_admission(
        mut self,
        publication_parent: Dir,
        effect_fate: super::owner_admission_effect::MediaOwnerAdmissionEffectFate,
    ) -> Self {
        self.publication_parent = Some(publication_parent);
        self.store_root_publication_required = true;
        self.root_parent_publication_required = true;
        self.admission_effect_fate = effect_fate;
        self
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(super) fn complete_existing_admission(mut self, publication_parent: Dir) -> Self {
        self.publication_parent = Some(publication_parent);
        self
    }
    pub(super) fn open_directory(
        &mut self,
        path: &NamespaceRelativePath,
        boundary: &super::fault_interposition::MediaFaultInterposer,
    ) -> Result<NamespaceDirectoryHandle, NamespaceConfinementDenial> {
        self.open_directory_after_check(path, boundary)
    }

    fn open_directory_after_check(
        &mut self,
        path: &NamespaceRelativePath,
        boundary: &super::fault_interposition::MediaFaultInterposer,
    ) -> Result<NamespaceDirectoryHandle, NamespaceConfinementDenial> {
        if path.owner_identity() != self.owner
            || path.parent() != super::namespace_confinement::NamespaceParent::Root
        {
            return Err(NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::AuthorityMismatch,
            ));
        }
        let relative = path.file_name();
        let metadata = boundary_io_call(
            boundary,
            super::MediaOperationRole::InspectNamespaceEntry,
            || self.root.directory().symlink_metadata(relative),
        )
        .map_err(|error| NamespaceConfinementDenial::from_io(&error))?;
        if metadata.file_type().is_symlink() {
            return Err(NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::LinkLikeEntry,
            ));
        }
        if !metadata.is_dir() {
            return Err(NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::EntryTypeMismatch,
            ));
        }

        let directory =
            boundary_io_call(boundary, super::MediaOperationRole::OpenDirectory, || {
                self.root.directory().open_dir_nofollow(relative)
            })
            .map_err(|error| NamespaceConfinementDenial::from_io(&error))?;
        let generation = self.issue_handle_generation()?;
        Ok(NamespaceDirectoryHandle::new(
            self.owner,
            generation,
            path.role(),
            directory,
            boundary.shared_counters().directory_handle_opened(),
        ))
    }

    pub(super) fn open_role_directory(
        &mut self,
        role: StoreNamespaceRelativeRole,
        boundary: &super::fault_interposition::MediaFaultInterposer,
    ) -> Result<NamespaceDirectoryHandle, NamespaceConfinementDenial> {
        if role.expected_entry_type() != NamespaceEntryType::Directory {
            return Err(NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::EntryTypeMismatch,
            ));
        }
        self.open_directory(
            &NamespaceRelativePath::bind_role(self.owner, role),
            boundary,
        )
    }

    pub(super) fn require_owned_directory(
        &self,
        handle: &NamespaceDirectoryHandle,
    ) -> Result<(), NamespaceConfinementDenial> {
        if handle.belongs_to(self.owner) {
            Ok(())
        } else {
            Err(NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::AuthorityMismatch,
            ))
        }
    }

    pub(super) const fn publication_parent(&self) -> Option<&Dir> {
        self.publication_parent.as_ref()
    }

    pub(super) const fn store_root_publication_required(&self) -> bool {
        self.store_root_publication_required
    }

    pub(super) const fn root_parent_publication_required(&self) -> bool {
        self.root_parent_publication_required
    }

    fn issue_handle_generation(&mut self) -> Result<u64, NamespaceConfinementDenial> {
        let generation = self.next_handle_generation;
        self.next_handle_generation = generation.checked_add(1).ok_or_else(|| {
            NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::AuthorityIdentityUnavailable,
            )
        })?;
        Ok(generation)
    }
}

pub(super) enum BoundaryIoStop {
    Indeterminate,
    Denied(std::io::Error),
}

pub(super) fn boundary_io_call<T>(
    boundary: &super::fault_interposition::MediaFaultInterposer,
    role: super::MediaOperationRole,
    call: impl FnOnce() -> std::io::Result<T>,
) -> std::io::Result<T> {
    let mut value = None;
    let mut call = Some(call);
    match boundary_io(boundary, role, &mut || {
        call.take().expect("boundary I/O runs once")().map(|item| value = Some(item))
    }) {
        Ok(()) => Ok(value.expect("completed boundary I/O stores its value")),
        Err(BoundaryIoStop::Denied(error)) => Err(error),
        Err(BoundaryIoStop::Indeterminate) => Err(std::io::Error::other(
            "certification interrupted admission-effect observation",
        )),
    }
}

/// One compiled fault-interposition boundary for every I/O closure.
#[inline(never)]
pub(super) fn boundary_io(
    boundary: &super::fault_interposition::MediaFaultInterposer,
    role: super::MediaOperationRole,
    call: &mut dyn FnMut() -> std::io::Result<()>,
) -> Result<(), BoundaryIoStop> {
    let attempt = boundary.begin(role, 0);
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(BoundaryIoStop::Denied(error));
    }
    match call() {
        Ok(()) if attempt.effect_observation_is_indeterminate() => {
            attempt.indeterminate(0);
            Err(BoundaryIoStop::Indeterminate)
        }
        Ok(()) => {
            attempt.completed(0);
            Ok(())
        }
        Err(error) => {
            attempt.denied();
            Err(BoundaryIoStop::Denied(error))
        }
    }
}
