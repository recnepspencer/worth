use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use std::io::Read;
use worth_store_physical_format::store_namespace::{
    classify_store_namespace, NamespaceDamage, NamespaceEntryObservation, NamespaceEntryType,
    NamespaceRootObservation, StagedNamespaceName, StoreNamespaceClassification,
    StoreNamespaceRelativeRole, STORE_NAMESPACE_IDENTITY_RECORD_LENGTH,
};

use super::fault_interposition::MediaFaultInterposer;
use super::{MediaOperationRole, NamespaceConfinementDenial};

const MAX_NAMESPACE_ENTRIES: usize = 1_024;

pub(super) fn classify_opened_root(
    root: &Dir,
    boundary: &MediaFaultInterposer,
) -> Result<StoreNamespaceClassification, NamespaceConfinementDenial> {
    complete_scaffold_snapshot(classify_once(root, boundary)?, root, boundary)
}

fn complete_scaffold_snapshot(
    first: StoreNamespaceClassification,
    root: &Dir,
    boundary: &MediaFaultInterposer,
) -> Result<StoreNamespaceClassification, NamespaceConfinementDenial> {
    // Identity publication follows all three fixed directories. A root walk
    // can start before that scaffold exists and read identity after publication.
    // Reobserve that one mixed-time shape after its identity witness; persistent
    // missing directories still classify as damage, never as a new namespace.
    if matches!(
        first,
        StoreNamespaceClassification::Damaged(
            NamespaceDamage::PublishedIdentityWithoutCompleteScaffold
        )
    ) {
        classify_once(root, boundary)
    } else {
        Ok(first)
    }
}

fn classify_once(
    root: &Dir,
    boundary: &MediaFaultInterposer,
) -> Result<StoreNamespaceClassification, NamespaceConfinementDenial> {
    let mut observations = Vec::new();
    let entries = entries(root, boundary)?;
    for entry in entries {
        let entry = entry.map_err(|error| NamespaceConfinementDenial::from_io(&error))?;
        require_inventory_bound(observations.len())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let entry_type = classify_entry_type(
            entry
                .file_type()
                .map_err(|error| NamespaceConfinementDenial::from_io(&error))?,
        );
        match name.as_ref() {
            "namespace" => {
                observations.push(NamespaceEntryObservation::canonical(
                    StoreNamespaceRelativeRole::NamespaceDirectory,
                    entry_type,
                ));
                if entry_type == NamespaceEntryType::Directory {
                    observe_namespace_directory(root, boundary, &mut observations)?;
                }
            }
            "families" => observations.push(NamespaceEntryObservation::canonical(
                StoreNamespaceRelativeRole::FamiliesDirectory,
                entry_type,
            )),
            "staging" => observations.push(NamespaceEntryObservation::canonical(
                StoreNamespaceRelativeRole::StagingDirectory,
                entry_type,
            )),
            _ => observations.push(NamespaceEntryObservation::unknown(name, entry_type)),
        }
    }
    boundary.shared_counters().listing_batch(observations.len());
    Ok(classify_store_namespace(
        &NamespaceRootObservation::directory(observations),
    ))
}

#[cfg(test)]
mod snapshot_tests;

fn observe_namespace_directory(
    root: &Dir,
    boundary: &MediaFaultInterposer,
    observations: &mut Vec<NamespaceEntryObservation>,
) -> Result<(), NamespaceConfinementDenial> {
    let namespace = super::admitted_namespace::boundary_io_call(
        boundary,
        MediaOperationRole::OpenDirectory,
        || root.open_dir_nofollow("namespace"),
    )
    .map_err(|error| NamespaceConfinementDenial::from_io(&error))?;
    let entries = entries(&namespace, boundary)?;
    for entry in entries {
        let entry = entry.map_err(|error| NamespaceConfinementDenial::from_io(&error))?;
        require_inventory_bound(observations.len())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let entry_type = classify_entry_type(
            entry
                .file_type()
                .map_err(|error| NamespaceConfinementDenial::from_io(&error))?,
        );
        match name.as_ref() {
            "identity" if entry_type == NamespaceEntryType::RegularFile => {
                observations.push(NamespaceEntryObservation::published_identity(
                    read_identity(&namespace, boundary)?,
                ));
            }
            "identity" => observations.push(NamespaceEntryObservation::CanonicalRole {
                role: StoreNamespaceRelativeRole::IdentityRecord,
                entry_type,
            }),
            "mutation.lock" => observations.push(NamespaceEntryObservation::canonical(
                StoreNamespaceRelativeRole::MutationLock,
                entry_type,
            )),
            _ => match StagedNamespaceName::parse(&name) {
                Some(staged) => observations.push(NamespaceEntryObservation::StagedIdentity {
                    name: staged,
                    entry_type,
                }),
                None => observations.push(NamespaceEntryObservation::unknown(
                    format!("namespace/{name}"),
                    entry_type,
                )),
            },
        }
    }
    Ok(())
}

fn entries(
    directory: &Dir,
    boundary: &MediaFaultInterposer,
) -> Result<cap_std::fs::ReadDir, NamespaceConfinementDenial> {
    super::admitted_namespace::boundary_io_call(boundary, MediaOperationRole::ListDirectory, || {
        directory.entries()
    })
    .map_err(|error| NamespaceConfinementDenial::from_io(&error))
}

fn read_identity(
    namespace: &Dir,
    boundary: &MediaFaultInterposer,
) -> Result<Vec<u8>, NamespaceConfinementDenial> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = super::admitted_namespace::boundary_io_call(
        boundary,
        MediaOperationRole::OpenExisting,
        || namespace.open_with("identity", &options),
    )
    .map_err(|error| NamespaceConfinementDenial::from_io(&error))?;
    let mut bytes = Vec::with_capacity(STORE_NAMESPACE_IDENTITY_RECORD_LENGTH + 1);
    boundary
        .shared_counters()
        .explicit_heap_allocation(bytes.capacity());
    let requested = (STORE_NAMESPACE_IDENTITY_RECORD_LENGTH + 1) as u64;
    let attempt = boundary.begin(MediaOperationRole::PositionedRead, requested);
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(NamespaceConfinementDenial::from_io(&error));
    }
    let limit = attempt.transfer_limit(requested);
    let result = file.take(limit).read_to_end(&mut bytes);
    match result {
        Ok(_) if limit != requested => {
            attempt.partial(bytes.len() as u64);
            return Err(NamespaceConfinementDenial::structural(
                super::NamespaceConfinementDenialKind::NamespaceNotAdmissible,
            ));
        }
        Ok(_) => attempt.completed(bytes.len() as u64),
        Err(error) if bytes.is_empty() => {
            attempt.denied();
            return Err(NamespaceConfinementDenial::from_io(&error));
        }
        Err(error) => {
            attempt.partial(bytes.len() as u64);
            return Err(NamespaceConfinementDenial::from_io(&error));
        }
    }
    Ok(bytes)
}

fn require_inventory_bound(observed: usize) -> Result<(), NamespaceConfinementDenial> {
    if observed < MAX_NAMESPACE_ENTRIES {
        Ok(())
    } else {
        Err(NamespaceConfinementDenial::structural(
            super::NamespaceConfinementDenialKind::NamespaceNotAdmissible,
        ))
    }
}

fn classify_entry_type(file_type: cap_std::fs::FileType) -> NamespaceEntryType {
    if file_type.is_file() {
        NamespaceEntryType::RegularFile
    } else if file_type.is_dir() {
        NamespaceEntryType::Directory
    } else if file_type.is_symlink() {
        NamespaceEntryType::LinkLike
    } else {
        NamespaceEntryType::Other
    }
}
