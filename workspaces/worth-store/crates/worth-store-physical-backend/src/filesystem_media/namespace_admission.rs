use std::path::Path;

use cap_fs_ext::DirExt;
use cap_std::{ambient_authority, fs::Dir};
use same_file::Handle as FileIdentityHandle;
use worth_store_physical_format::store_namespace::{
    StoreNamespaceClassification, StoreNamespaceRelativeRole,
};

use super::{
    owner_admission_effect::{MediaOwnerAdmissionEffectFate, NamespaceAdmissionFailure},
    AdmittedStoreNamespace, NamespaceConfinementDenial, NamespaceConfinementDenialKind,
};

pub(super) fn create_or_open(
    root: &Path,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) -> Result<AdmittedStoreNamespace, NamespaceAdmissionFailure> {
    let mut effect_fate = MediaOwnerAdmissionEffectFate::DeniedBeforeEffect;
    let name = root.file_name().ok_or_else(|| {
        admission_failure(
            NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::MissingParentPublicationBoundary,
            ),
            effect_fate,
        )
    })?;
    let parent_path = root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let publication_parent = super::admitted_namespace::boundary_io_call(
        boundary,
        super::MediaOperationRole::OpenRootParent,
        || Dir::open_ambient_dir(parent_path, ambient_authority()),
    )
    .map_err(|error| io_admission_failure(error, effect_fate))?;
    let root_was_absent = admit_root_entry(&publication_parent, name, boundary, &mut effect_fate)?;
    let directory = super::admitted_namespace::boundary_io_call(
        boundary,
        super::MediaOperationRole::OpenDirectory,
        || publication_parent.open_dir_nofollow(name),
    )
    .map_err(|error| io_admission_failure(error, effect_fate))?;
    require_opened_root_identity(root, &directory, boundary)
        .map_err(|denial| admission_failure(denial, effect_fate))?;
    let classification = classify_root(root_was_absent, &directory, boundary, effect_fate)?;
    let initialize_scaffold = requires_scaffold(&classification);
    bind_initialized_store(&classification, boundary);
    require_admissible_classification(&classification, initialize_scaffold, effect_fate)?;
    let namespace = AdmittedStoreNamespace::from_opened_directory(directory, boundary)
        .map_err(|denial| admission_failure(denial, effect_fate))?;
    if initialize_scaffold {
        create_fixed_scaffold(&namespace, boundary, &mut effect_fate)?;
    }
    Ok(namespace.complete_admission(publication_parent, effect_fate))
}

#[cfg(feature = "recovery-runtime-owner")]
pub(super) fn open_existing(
    root: &Path,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) -> Result<AdmittedStoreNamespace, NamespaceAdmissionFailure> {
    let effect_fate = MediaOwnerAdmissionEffectFate::DeniedBeforeEffect;
    let name = root.file_name().ok_or_else(|| {
        admission_failure(
            NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::MissingParentPublicationBoundary,
            ),
            effect_fate,
        )
    })?;
    let parent_path = root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let publication_parent = super::admitted_namespace::boundary_io_call(
        boundary,
        super::MediaOperationRole::OpenRootParent,
        || Dir::open_ambient_dir(parent_path, ambient_authority()),
    )
    .map_err(|error| io_admission_failure(error, effect_fate))?;
    require_existing_root_entry(&publication_parent, name, boundary, effect_fate)?;
    let directory = super::admitted_namespace::boundary_io_call(
        boundary,
        super::MediaOperationRole::OpenDirectory,
        || publication_parent.open_dir_nofollow(name),
    )
    .map_err(|error| io_admission_failure(error, effect_fate))?;
    require_opened_root_identity(root, &directory, boundary)
        .map_err(|denial| admission_failure(denial, effect_fate))?;
    let classification = classify_root(false, &directory, boundary, effect_fate)?;
    if !matches!(
        classification,
        StoreNamespaceClassification::Initialized { .. }
    ) {
        return Err(admission_failure(
            NamespaceConfinementDenial::structural(classification_denial_kind(&classification)),
            effect_fate,
        ));
    }
    AdmittedStoreNamespace::from_opened_directory(directory, boundary)
        .map(|namespace| namespace.complete_existing_admission(publication_parent))
        .map_err(|denial| admission_failure(denial, effect_fate))
}

#[cfg(feature = "recovery-runtime-owner")]
fn require_existing_root_entry(
    publication_parent: &Dir,
    name: &std::ffi::OsStr,
    boundary: &super::fault_interposition::MediaFaultInterposer,
    effect_fate: MediaOwnerAdmissionEffectFate,
) -> Result<(), NamespaceAdmissionFailure> {
    let metadata = super::admitted_namespace::boundary_io_call(
        boundary,
        super::MediaOperationRole::InspectNamespaceEntry,
        || publication_parent.symlink_metadata(name),
    )
    .map_err(|error| io_admission_failure(error, effect_fate))?;
    require_directory_metadata(metadata).map_err(|denial| admission_failure(denial, effect_fate))
}

fn admit_root_entry(
    publication_parent: &Dir,
    name: &std::ffi::OsStr,
    boundary: &super::fault_interposition::MediaFaultInterposer,
    effect_fate: &mut MediaOwnerAdmissionEffectFate,
) -> Result<bool, NamespaceAdmissionFailure> {
    match super::admitted_namespace::boundary_io_call(
        boundary,
        super::MediaOperationRole::InspectNamespaceEntry,
        || publication_parent.symlink_metadata(name),
    ) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(admission_failure(
            NamespaceConfinementDenial::structural(NamespaceConfinementDenialKind::LinkLikeEntry),
            *effect_fate,
        )),
        Ok(metadata) if !metadata.is_dir() => Err(admission_failure(
            NamespaceConfinementDenial::structural(
                NamespaceConfinementDenialKind::EntryTypeMismatch,
            ),
            *effect_fate,
        )),
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match namespace_mutation_call(
                boundary,
                super::MediaOperationRole::CreateDirectory,
                || publication_parent.create_dir(name),
            ) {
                Ok(()) => {
                    *effect_fate = MediaOwnerAdmissionEffectFate::EffectPossible;
                    Ok(true)
                }
                Err(failure) if failure.error.kind() == std::io::ErrorKind::AlreadyExists => {
                    Ok(false)
                }
                Err(failure) => Err(io_admission_failure(
                    failure.error,
                    effect_fate.combine(failure.effect_fate),
                )),
            }
        }
        Err(error) => Err(io_admission_failure(error, *effect_fate)),
    }
}

fn classify_root(
    root_was_absent: bool,
    directory: &Dir,
    boundary: &super::fault_interposition::MediaFaultInterposer,
    effect_fate: MediaOwnerAdmissionEffectFate,
) -> Result<StoreNamespaceClassification, NamespaceAdmissionFailure> {
    if root_was_absent {
        Ok(StoreNamespaceClassification::AbsentEligible)
    } else {
        super::namespace_root_inventory::classify_opened_root(directory, boundary)
            .map_err(|denial| admission_failure(denial, effect_fate))
    }
}

fn requires_scaffold(classification: &StoreNamespaceClassification) -> bool {
    matches!(
        classification,
        StoreNamespaceClassification::AbsentEligible
            | StoreNamespaceClassification::EmptyEligible
            | StoreNamespaceClassification::IncompleteScaffold { .. }
    )
}

fn bind_initialized_store(
    classification: &StoreNamespaceClassification,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) {
    if let StoreNamespaceClassification::Initialized { identity, .. } = classification {
        boundary.bind_store(*identity);
    }
}

fn require_admissible_classification(
    classification: &StoreNamespaceClassification,
    initialize_scaffold: bool,
    effect_fate: MediaOwnerAdmissionEffectFate,
) -> Result<(), NamespaceAdmissionFailure> {
    if initialize_scaffold
        || matches!(
            classification,
            StoreNamespaceClassification::Initialized { .. }
        )
    {
        return Ok(());
    }
    Err(admission_failure(
        NamespaceConfinementDenial::structural(classification_denial_kind(classification)),
        effect_fate,
    ))
}

fn create_fixed_scaffold(
    namespace: &AdmittedStoreNamespace,
    boundary: &super::fault_interposition::MediaFaultInterposer,
    effect_fate: &mut MediaOwnerAdmissionEffectFate,
) -> Result<(), NamespaceAdmissionFailure> {
    for role in [
        StoreNamespaceRelativeRole::NamespaceDirectory,
        StoreNamespaceRelativeRole::FamiliesDirectory,
        StoreNamespaceRelativeRole::StagingDirectory,
    ] {
        admit_scaffold_directory(
            namespace.root_handle().directory(),
            role.components()[0],
            boundary,
            effect_fate,
        )?;
    }
    Ok(())
}

fn admit_scaffold_directory(
    root: &Dir,
    name: &str,
    boundary: &super::fault_interposition::MediaFaultInterposer,
    effect_fate: &mut MediaOwnerAdmissionEffectFate,
) -> Result<(), NamespaceAdmissionFailure> {
    match super::admitted_namespace::boundary_io_call(
        boundary,
        super::MediaOperationRole::InspectNamespaceEntry,
        || root.symlink_metadata(name),
    ) {
        Ok(metadata) => require_directory_metadata(metadata)
            .map_err(|denial| admission_failure(denial, *effect_fate)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match namespace_mutation_call(
                boundary,
                super::MediaOperationRole::CreateDirectory,
                || root.create_dir(name),
            ) {
                Ok(()) => {
                    *effect_fate = MediaOwnerAdmissionEffectFate::EffectPossible;
                    Ok(())
                }
                Err(failure) if failure.error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let metadata = super::admitted_namespace::boundary_io_call(
                        boundary,
                        super::MediaOperationRole::InspectNamespaceEntry,
                        || root.symlink_metadata(name),
                    )
                    .map_err(|error| io_admission_failure(error, *effect_fate))?;
                    require_directory_metadata(metadata)
                        .map_err(|denial| admission_failure(denial, *effect_fate))
                }
                Err(failure) => Err(io_admission_failure(
                    failure.error,
                    effect_fate.combine(failure.effect_fate),
                )),
            }
        }
        Err(error) => Err(io_admission_failure(error, *effect_fate)),
    }
}

struct NamespaceMutationFailure {
    error: std::io::Error,
    effect_fate: MediaOwnerAdmissionEffectFate,
}

fn namespace_mutation_call<T>(
    boundary: &super::fault_interposition::MediaFaultInterposer,
    role: super::MediaOperationRole,
    call: impl FnOnce() -> std::io::Result<T>,
) -> Result<T, NamespaceMutationFailure> {
    let mut value = None;
    let mut call = Some(call);
    match super::admitted_namespace::boundary_io(boundary, role, &mut || {
        call.take().expect("namespace mutation runs once")().map(|item| value = Some(item))
    }) {
        Ok(()) => Ok(value.expect("completed namespace mutation stores its value")),
        Err(super::admitted_namespace::BoundaryIoStop::Denied(error)) => {
            Err(denied_mutation(error))
        }
        Err(super::admitted_namespace::BoundaryIoStop::Indeterminate) => {
            Err(interrupted_mutation())
        }
    }
}

fn denied_mutation(error: std::io::Error) -> NamespaceMutationFailure {
    NamespaceMutationFailure {
        error,
        effect_fate: MediaOwnerAdmissionEffectFate::DeniedBeforeEffect,
    }
}

fn interrupted_mutation() -> NamespaceMutationFailure {
    NamespaceMutationFailure {
        error: std::io::Error::other("certification interrupted admission-effect observation"),
        effect_fate: MediaOwnerAdmissionEffectFate::EffectPossible,
    }
}

pub(super) fn require_opened_root_identity(
    root: &Path,
    directory: &Dir,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) -> Result<(), NamespaceConfinementDenial> {
    let (opened, current) = super::admitted_namespace::boundary_io_call(
        boundary,
        super::MediaOperationRole::ValidateRootIdentity,
        || {
            let opened = directory
                .try_clone()
                .map(Dir::into_std_file)
                .and_then(FileIdentityHandle::from_file)?;
            let current = FileIdentityHandle::from_path(root)?;
            Ok((opened, current))
        },
    )
    .map_err(|error| NamespaceConfinementDenial::from_io(&error))?;
    if opened == current {
        Ok(())
    } else {
        Err(NamespaceConfinementDenial::structural(
            NamespaceConfinementDenialKind::RootIdentityChanged,
        ))
    }
}

fn require_directory_metadata(
    metadata: cap_std::fs::Metadata,
) -> Result<(), NamespaceConfinementDenial> {
    if metadata.file_type().is_symlink() {
        Err(NamespaceConfinementDenial::structural(
            NamespaceConfinementDenialKind::LinkLikeEntry,
        ))
    } else if metadata.is_dir() {
        Ok(())
    } else {
        Err(NamespaceConfinementDenial::structural(
            NamespaceConfinementDenialKind::EntryTypeMismatch,
        ))
    }
}

fn classification_denial_kind(
    classification: &StoreNamespaceClassification,
) -> NamespaceConfinementDenialKind {
    match classification {
        StoreNamespaceClassification::IncompleteScaffold { .. } => {
            NamespaceConfinementDenialKind::NamespaceIncomplete
        }
        StoreNamespaceClassification::UnsupportedVersion(_) => {
            NamespaceConfinementDenialKind::NamespaceVersionUnsupported
        }
        StoreNamespaceClassification::Damaged(_) => {
            NamespaceConfinementDenialKind::NamespaceDamaged
        }
        StoreNamespaceClassification::Ambiguous(_) => {
            NamespaceConfinementDenialKind::NamespaceAmbiguous
        }
        _ => NamespaceConfinementDenialKind::NamespaceNotAdmissible,
    }
}

fn io_admission_failure(
    error: std::io::Error,
    effect_fate: MediaOwnerAdmissionEffectFate,
) -> NamespaceAdmissionFailure {
    admission_failure(NamespaceConfinementDenial::from_io(&error), effect_fate)
}

const fn admission_failure(
    denial: NamespaceConfinementDenial,
    effect_fate: MediaOwnerAdmissionEffectFate,
) -> NamespaceAdmissionFailure {
    NamespaceAdmissionFailure::new(denial, effect_fate)
}
