#[cfg(feature = "recovery-runtime-owner")]
use super::super::{
    listing_admission::change, listing_storage::provider_iterator_storage,
    ArtifactTreeAllocatedListingFailure, ArtifactTreeFailure, ArtifactTreeFailureKind,
    ArtifactTreeListingAllocationBoundary, ArtifactTreeListingAllocator,
    ArtifactTreeListingStorageChange,
};
use cap_std::fs::Dir;
use std::{fs, io, os::windows::ffi::OsStrExt, path::Path};

pub(in crate::filesystem_media::artifact_tree) struct DirectoryEntries {
    entries: fs::ReadDir,
    // cap's no-delete-sharing directory handle must remain live while the
    // handle-derived pathname is used. ReadDir is disposed before this handle.
    _directory: fs::File,
}

pub(in crate::filesystem_media::artifact_tree) fn open(
    directory: Dir,
) -> io::Result<DirectoryEntries> {
    let directory = directory.into_std_file();
    let path = winx::file::get_file_path(&directory)?;
    require_extended_absolute_path(&path)?;
    let entries = fs::read_dir(&path)?;
    Ok(DirectoryEntries {
        entries,
        _directory: directory,
    })
}

fn require_extended_absolute_path(path: &Path) -> io::Result<()> {
    // Preserve the handle-derived prefix verbatim. Stripping it turns extended
    // UNC paths into relative names and reintroduces current-directory lookup.
    let extended = path.as_os_str().encode_wide().take(4).eq([
        b'\\' as u16,
        b'\\' as u16,
        b'?' as u16,
        b'\\' as u16,
    ]);
    if extended && path.is_absolute() {
        Ok(())
    } else {
        Err(io::ErrorKind::InvalidData.into())
    }
}

#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::filesystem_media::artifact_tree) fn open_with_allocator<
    A: ArtifactTreeListingAllocator,
>(
    directory: Dir,
    allocator: &mut A,
) -> Result<(DirectoryEntries, u64), ArtifactTreeAllocatedListingFailure<A::Denial>> {
    let directory = directory.into_std_file();
    // The transient handle-path conversion lives inside the caller's admitted
    // operation envelope; only the iterator storage it seeds is admitted here.
    let path = winx::file::get_file_path(&directory).map_err(provider_failure)?;
    require_extended_absolute_path(&path).map_err(provider_failure)?;
    let (required_bytes, retained_bytes) = provider_iterator_storage(&path).ok_or_else(|| {
        ArtifactTreeFailure::structural(ArtifactTreeFailureKind::AccessLimitExceeded)
    })?;
    change(
        allocator,
        ArtifactTreeListingStorageChange::Admit {
            boundary: ArtifactTreeListingAllocationBoundary::ProviderIterator,
            required_bytes,
        },
    )?;
    let entries = fs::read_dir(&path).map_err(provider_failure)?;
    drop(path);
    change(
        allocator,
        ArtifactTreeListingStorageChange::Settle { retained_bytes },
    )?;
    Ok((
        DirectoryEntries {
            entries,
            _directory: directory,
        },
        retained_bytes,
    ))
}

#[cfg(feature = "recovery-runtime-owner")]
fn provider_failure(error: io::Error) -> ArtifactTreeFailure {
    ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error)
}

impl Iterator for DirectoryEntries {
    type Item = io::Result<fs::DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_path_posture_preserves_extended_disk_and_unc_without_normalization() {
        for path in [r"\\?\C:\store\wal", r"\\?\UNC\server\share\store\wal"] {
            require_extended_absolute_path(Path::new(path)).unwrap();
        }
        for path in [r"C:\store\wal", r"UNC\server\share\wal", r"wal", ""] {
            assert_eq!(
                require_extended_absolute_path(Path::new(path))
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
        }
    }
}
