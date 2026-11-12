use super::{
    ArtifactTreeAllocatedListingFailure, ArtifactTreeDirectory, ArtifactTreeDirectoryEntry,
    ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeListingAllocator, ArtifactTreeMedia,
};

#[cfg(any(windows, target_os = "macos"))]
use super::{
    listing_admission::change, ArtifactTreeListingAllocationBoundary as Boundary,
    ArtifactTreeListingStorageChange as Change,
};

impl ArtifactTreeMedia<'_> {
    /// Admits provider/name/roster backing before construction. Confined path
    /// construction and component opening remain outside this listing closure.
    pub(crate) fn list_bounded_with_allocator<
        A: ArtifactTreeListingAllocator + super::ArtifactTreePathAllocator,
    >(
        &self,
        directory: &ArtifactTreeDirectory,
        maximum_entries: usize,
        allocator: &mut A,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, ArtifactTreeAllocatedListingFailure<A::Denial>>
    {
        if self.listing_storage_requirement(maximum_entries).is_none() {
            return Err(unqualified().into());
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        {
            let _ = (directory, allocator);
            Err(unqualified().into())
        }
        #[cfg(any(windows, target_os = "macos"))]
        {
            use crate::filesystem_media::{artifact_tree_effects::begin, MediaOperationRole};
            let directory = self
                .open_directory_with_allocator(directory, allocator)
                .map_err(|failure| match failure {
                    super::ArtifactTreeAllocatedReadFailure::Media(failure) => {
                        ArtifactTreeAllocatedListingFailure::Media(failure)
                    }
                    super::ArtifactTreeAllocatedReadFailure::Allocation { requested, cause } => {
                        ArtifactTreeAllocatedListingFailure::Allocation {
                            requested: requested as u64,
                            cause,
                        }
                    }
                    super::ArtifactTreeAllocatedReadFailure::BufferLengthMismatch {
                        requested,
                        observed,
                    } => ArtifactTreeAllocatedListingFailure::BufferLengthMismatch {
                        requested,
                        observed,
                    },
                })?;
            let attempt = begin(self.owner, MediaOperationRole::ListDirectory, 0);
            if let Some(error) = attempt.fail_before_error() {
                attempt.denied();
                return Err(ArtifactTreeFailure::io(
                    ArtifactTreeFailureKind::DeniedBeforeEffect,
                    &error,
                )
                .into());
            }
            // On failure this helper disposes provider, names, and roster before
            // the zero settlement. On success it disposes the provider first.
            match enumerate(directory, maximum_entries, allocator) {
                Err(failure) => {
                    attempt.denied();
                    change(allocator, Change::Settle { retained_bytes: 0 })?;
                    Err(failure)
                }
                Ok((observed, retained_bytes)) => {
                    if let Err(failure) = change(allocator, Change::Settle { retained_bytes }) {
                        drop(observed);
                        attempt.denied();
                        return Err(failure);
                    }
                    self.owner
                        .boundary()
                        .counters()
                        .listing_batch(observed.len());
                    // Existing PauseAfter observers now see only retained names
                    // and the listing roster, never disposed provider scratch.
                    attempt.completed(0);
                    Ok(observed)
                }
            }
        }
    }
}

fn unqualified() -> ArtifactTreeFailure {
    ArtifactTreeFailure::structural(ArtifactTreeFailureKind::AccessLimitExceeded)
}

#[cfg(any(windows, target_os = "macos"))]
fn enumerate<A: ArtifactTreeListingAllocator>(
    directory: cap_std::fs::Dir,
    maximum_entries: usize,
    allocator: &mut A,
) -> Result<(Vec<ArtifactTreeDirectoryEntry>, u64), ArtifactTreeAllocatedListingFailure<A::Denial>>
{
    #[cfg(windows)]
    use worth_store_physical_format::store_namespace::NamespaceEntryType;
    let (mut entries, provider) =
        super::listing_provider::open_with_allocator(directory, allocator)?;
    let mut observed = Vec::new();
    let mut names = 0_u64;
    loop {
        #[cfg(target_os = "macos")]
        {
            let live = listing_bytes(observed.capacity(), names)?
                .checked_add(provider)
                .ok_or_else(unqualified)?;
            change(
                allocator,
                Change::Admit {
                    boundary: Boundary::EntryName,
                    required_bytes: live
                        .checked_add(super::listing_storage::macos::CSTRING_BYTES)
                        .ok_or_else(unqualified)?,
                },
            )?;
        }
        let Some(entry) = entries.next() else { break };
        if observed.len() == maximum_entries {
            return Err(ArtifactTreeFailure::limit(
                observed.len() as u64 + 1,
                maximum_entries as u64,
            )
            .into());
        }
        let entry = entry.map_err(entry_failure)?;
        #[cfg(windows)]
        let entry_type = {
            let file_type = entry.file_type().map_err(entry_failure)?;
            if file_type.is_file() {
                NamespaceEntryType::RegularFile
            } else if file_type.is_dir() {
                NamespaceEntryType::Directory
            } else if file_type.is_symlink() {
                NamespaceEntryType::LinkLike
            } else {
                NamespaceEntryType::Other
            }
        };
        #[cfg(target_os = "macos")]
        let entry_type = entry.entry_type();
        #[cfg(target_os = "macos")]
        let entry_storage = entry.name_storage();
        #[cfg(windows)]
        let entry_storage = 0_u64;
        #[cfg(target_os = "macos")]
        if entry_storage > super::listing_storage::macos::CSTRING_BYTES {
            return Err(ArtifactTreeAllocatedListingFailure::BufferLengthMismatch {
                requested: super::listing_storage::macos::CSTRING_BYTES as usize,
                observed: entry_storage as usize,
            });
        }
        let resident = provider
            .checked_add(entry_storage)
            .ok_or_else(unqualified)?;
        reserve_slot(&mut observed, maximum_entries, resident, names, allocator)?;
        let live = listing_bytes(observed.capacity(), names)?
            .checked_add(resident)
            .ok_or_else(unqualified)?;
        #[cfg(windows)]
        let (copy_peak, maximum_name) = (3_120_u64, 1_560_usize);
        #[cfg(target_os = "macos")]
        let (copy_peak, maximum_name) = (
            super::listing_storage::macos::NAME_BYTES,
            super::listing_storage::macos::NAME_BYTES as usize,
        );
        change(
            allocator,
            Change::Admit {
                boundary: Boundary::EntryName,
                required_bytes: live.checked_add(copy_peak).ok_or_else(unqualified)?,
            },
        )?;
        let name = entry.file_name();
        if name.capacity() > maximum_name {
            return Err(ArtifactTreeAllocatedListingFailure::BufferLengthMismatch {
                requested: maximum_name,
                observed: name.capacity(),
            });
        }
        names = names
            .checked_add(u64::try_from(name.capacity()).map_err(|_| unqualified())?)
            .ok_or_else(unqualified)?;
        observed.push(ArtifactTreeDirectoryEntry::new(name, entry_type));
        drop(entry); // Native CString is gone before retained-storage settlement.
        change(
            allocator,
            Change::Settle {
                retained_bytes: listing_bytes(observed.capacity(), names)?
                    .checked_add(provider)
                    .ok_or_else(unqualified)?,
            },
        )?;
    }
    drop(entries);
    let retained = listing_bytes(observed.capacity(), names)?;
    Ok((observed, retained))
}

#[cfg(any(windows, target_os = "macos"))]
fn reserve_slot<A: ArtifactTreeListingAllocator>(
    observed: &mut Vec<ArtifactTreeDirectoryEntry>,
    maximum_entries: usize,
    provider: u64,
    names: u64,
    allocator: &mut A,
) -> Result<(), ArtifactTreeAllocatedListingFailure<A::Denial>> {
    if observed.len() < observed.capacity() {
        return Ok(());
    }
    let capacity = observed
        .capacity()
        .checked_mul(2)
        .ok_or_else(unqualified)?
        .max(4)
        .min(maximum_entries);
    let simultaneous = observed
        .capacity()
        .checked_add(capacity)
        .ok_or_else(unqualified)?;
    change(
        allocator,
        Change::Admit {
            boundary: Boundary::EntryRoster,
            required_bytes: listing_bytes(simultaneous, names)?
                .checked_add(provider)
                .ok_or_else(unqualified)?,
        },
    )?;
    let requested = slot_bytes(capacity)?;
    let mut prepared = allocator
        .allocate_listing_roster(capacity)
        .map_err(|cause| ArtifactTreeAllocatedListingFailure::Allocation { requested, cause })?;
    let mismatch = if !prepared.is_empty() {
        Some((0, prepared.len()))
    } else if prepared.capacity() != capacity {
        Some((capacity, prepared.capacity()))
    } else {
        None
    };
    if let Some((requested, actual)) = mismatch {
        return Err(ArtifactTreeAllocatedListingFailure::BufferLengthMismatch {
            requested: usize::try_from(slot_bytes(requested)?).map_err(|_| unqualified())?,
            observed: usize::try_from(slot_bytes(actual)?).map_err(|_| unqualified())?,
        });
    }
    let mut old = std::mem::take(observed);
    prepared.append(&mut old);
    *observed = prepared;
    drop(old);
    change(
        allocator,
        Change::Settle {
            retained_bytes: listing_bytes(observed.capacity(), names)?
                .checked_add(provider)
                .ok_or_else(unqualified)?,
        },
    )
}

#[cfg(any(windows, target_os = "macos"))]
fn slot_bytes(capacity: usize) -> Result<u64, ArtifactTreeFailure> {
    let bytes = capacity
        .checked_mul(std::mem::size_of::<ArtifactTreeDirectoryEntry>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or_else(unqualified)?;
    u64::try_from(bytes).map_err(|_| unqualified())
}

#[cfg(any(windows, target_os = "macos"))]
fn listing_bytes(capacity: usize, names: u64) -> Result<u64, ArtifactTreeFailure> {
    slot_bytes(capacity)?
        .checked_add(names)
        .ok_or_else(unqualified)
}

#[cfg(any(windows, target_os = "macos"))]
fn entry_failure(error: std::io::Error) -> ArtifactTreeFailure {
    ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error)
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests;
