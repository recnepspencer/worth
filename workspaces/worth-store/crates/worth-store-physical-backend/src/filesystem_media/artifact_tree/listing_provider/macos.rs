//! A streaming iterator consuming exactly the independently opened, observed fd.
use super::super::{
    listing_admission::change, listing_storage::macos::provider_bounds,
    ArtifactTreeAllocatedListingFailure as Failure, ArtifactTreeFailure, ArtifactTreeFailureKind,
    ArtifactTreeListingAllocationBoundary as Boundary, ArtifactTreeListingAllocator,
    ArtifactTreeListingStorageChange as Change,
};
use cap_std::fs::Dir;
use std::{ffi::OsString, io, os::unix::ffi::OsStringExt};
use worth_store_physical_format::store_namespace::NamespaceEntryType;

pub(in crate::filesystem_media::artifact_tree) struct DirectoryEntries {
    entries: rustix::fs::Dir,
    _directory: std::fs::File,
}

pub(in crate::filesystem_media::artifact_tree) struct Entry(rustix::fs::DirEntry);

impl Entry {
    pub(in crate::filesystem_media::artifact_tree) fn entry_type(&self) -> NamespaceEntryType {
        match self.0.file_type() {
            rustix::fs::FileType::RegularFile => NamespaceEntryType::RegularFile,
            rustix::fs::FileType::Directory => NamespaceEntryType::Directory,
            rustix::fs::FileType::Symlink => NamespaceEntryType::LinkLike,
            _ => NamespaceEntryType::Other,
        }
    }
    pub(in crate::filesystem_media::artifact_tree) fn name_storage(&self) -> u64 {
        self.0.file_name().to_bytes_with_nul().len() as u64
    }
    pub(in crate::filesystem_media::artifact_tree) fn file_name(&self) -> OsString {
        OsString::from_vec(self.0.file_name().to_bytes().to_vec())
    }
}

pub(in crate::filesystem_media::artifact_tree) fn open_with_allocator<
    A: ArtifactTreeListingAllocator,
>(
    directory: Dir,
    allocator: &mut A,
) -> Result<(DirectoryEntries, u64), Failure<A::Denial>> {
    let original = directory.into_std_file();
    // A new description isolates position from the held parent's other users.
    // fstatfs observes THIS fd, which Dir::new then consumes without reopening.
    let fd = rustix::fs::openat(
        &original,
        c".",
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|error| provider_failure(error.into()))?;
    require_streaming(
        rustix::fs::fstatfs(&fd)
            .map(|observed| observed.f_flags)
            .map_err(io::Error::from),
    )
    .map_err(provider_failure)?;
    let (required_bytes, retained_bytes) = provider_bounds().ok_or_else(|| {
        ArtifactTreeFailure::structural(ArtifactTreeFailureKind::AccessLimitExceeded)
    })?;
    change(
        allocator,
        Change::Admit {
            boundary: Boundary::ProviderIterator,
            required_bytes,
        },
    )?;
    let entries = rustix::fs::Dir::new(fd).map_err(|error| provider_failure(error.into()))?;
    change(allocator, Change::Settle { retained_bytes })?;
    Ok((
        DirectoryEntries {
            entries,
            _directory: original,
        },
        retained_bytes,
    ))
}

fn provider_failure(error: io::Error) -> ArtifactTreeFailure {
    ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error)
}

fn require_streaming(observed: io::Result<u32>) -> io::Result<()> {
    if observed? & libc::MNT_UNION as u32 != 0 {
        return Err(io::ErrorKind::Unsupported.into());
    }
    Ok(())
}

impl Iterator for DirectoryEntries {
    type Item = io::Result<Entry>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = self.entries.read()?.map_err(io::Error::from);
            match entry {
                Ok(entry) if matches!(entry.file_name().to_bytes(), b"." | b"..") => continue,
                Ok(entry) => return Some(Ok(Entry(entry))),
                Err(error) => return Some(Err(error)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::require_streaming;
    #[test]
    fn eager_union_and_unavailable_observations_refuse_streaming_construction() {
        assert!(require_streaming(Ok(0)).is_ok());
        assert_eq!(
            require_streaming(Ok(libc::MNT_UNION as u32))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::Unsupported
        );
        assert_eq!(
            require_streaming(Err(std::io::ErrorKind::PermissionDenied.into()))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
    }
}
