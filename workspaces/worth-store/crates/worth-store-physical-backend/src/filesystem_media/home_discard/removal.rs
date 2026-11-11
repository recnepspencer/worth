//! Tombstone ordering and confined, retryable removal; never removes the lease inode.
use super::{
    FilesystemHomeDiscardDenial as Denial, FilesystemHomeDiscardStage as Stage,
    HOME_DISCARD_TOMBSTONE,
};
use crate::filesystem_media::{
    admitted_namespace::boundary_io_call, directory_synchronization::synchronize_directory_handle,
    fault_interposition::MediaFaultInterposer, MediaOperationRole as Role,
};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use std::io::Write;

pub(super) fn discard(
    root: &Dir,
    namespace: &Dir,
    parent: &Dir,
    intent: &[u8],
    boundary: &MediaFaultInterposer,
) -> Result<(), Denial> {
    publish_intent(root, intent, boundary)?;
    sync(root, boundary, Stage::Tombstone)?;
    sync(parent, boundary, Stage::Tombstone)?;
    remove_contents(namespace, Some("mutation.lock"), boundary)?;
    sync(namespace, boundary, Stage::Removal)?;
    for entry in root.entries().map_err(|e| Denial::io(Stage::Removal, e))? {
        let entry = entry.map_err(|e| Denial::io(Stage::Removal, e))?;
        let name = entry.file_name();
        if name == "namespace" || name == HOME_DISCARD_TOMBSTONE {
            continue;
        }
        remove_entry(root, &entry, boundary)?;
    }
    sync(root, boundary, Stage::Removal)?;
    remove_name(root, std::ffi::OsStr::new(HOME_DISCARD_TOMBSTONE), boundary)?;
    sync(root, boundary, Stage::Completion)
}

fn publish_intent(
    root: &Dir,
    intent: &[u8],
    boundary: &MediaFaultInterposer,
) -> Result<(), Denial> {
    match root.symlink_metadata(HOME_DISCARD_TOMBSTONE) {
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(Denial::io(Stage::Tombstone, error)),
    }
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let file = boundary_io_call(boundary, Role::CreateNew, || {
        root.open_with(HOME_DISCARD_TOMBSTONE, &options)
    });
    match file {
        Ok(mut file) => {
            boundary_io_call(boundary, Role::PositionedWrite, || file.write_all(intent))
                .map_err(|e| Denial::io(Stage::Tombstone, e))?;
            boundary_io_call(boundary, Role::SynchronizeFileState, || file.sync_all())
                .map_err(|e| Denial::io(Stage::Tombstone, e))
        }
        // A previous, possibly torn intent is sufficient to resume disposal.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(Denial::io(Stage::Tombstone, error)),
    }
}

fn remove_contents(
    directory: &Dir,
    keep: Option<&str>,
    boundary: &MediaFaultInterposer,
) -> Result<(), Denial> {
    for entry in directory
        .entries()
        .map_err(|e| Denial::io(Stage::Removal, e))?
    {
        let entry = entry.map_err(|e| Denial::io(Stage::Removal, e))?;
        if keep.is_some_and(|name| entry.file_name() == name) {
            continue;
        }
        remove_entry(directory, &entry, boundary)?;
    }
    Ok(())
}

fn remove_entry(
    directory: &Dir,
    entry: &cap_std::fs::DirEntry,
    boundary: &MediaFaultInterposer,
) -> Result<(), Denial> {
    remove_name(directory, &entry.file_name(), boundary)
}

fn remove_name(
    directory: &Dir,
    name: &std::ffi::OsStr,
    boundary: &MediaFaultInterposer,
) -> Result<(), Denial> {
    let kind = directory
        .symlink_metadata(name)
        .map(|metadata| metadata.file_type())
        .map_err(|e| Denial::io(Stage::Removal, e))?;
    if kind.is_dir() && !kind.is_symlink() {
        use cap_fs_ext::DirExt;
        let child = directory
            .open_dir_nofollow(name)
            .map_err(|e| Denial::io(Stage::Removal, e))?;
        remove_contents(&child, None, boundary)?;
        sync(&child, boundary, Stage::Removal)?;
        drop(child);
        boundary_io_call(boundary, Role::Delete, || directory.remove_dir(name))
            .map_err(|e| Denial::io(Stage::Removal, e))
    } else {
        boundary_io_call(boundary, Role::Delete, || directory.remove_file(name))
            .map_err(|e| Denial::io(Stage::Removal, e))
    }
}

fn sync(directory: &Dir, boundary: &MediaFaultInterposer, stage: Stage) -> Result<(), Denial> {
    boundary_io_call(boundary, Role::SynchronizeDirectoryPublication, || {
        synchronize_directory_handle(directory)
    })
    .map_err(|e| Denial::io(stage, e))
}
