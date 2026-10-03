use super::ArtifactTreeMedia;

/// Requested application-heap bounds for listing an already-confined directory.
/// This is a mechanical cost description, not allocation or filesystem authority.
/// It excludes construction/opening of the caller's confined path, allocator
/// bookkeeping, OS storage, and later payload/error-context copies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactTreeListingStorageRequirement {
    maximum_entries: usize,
    provider_construction_peak: u64,
    provider_retained: u64,
    entry_name_construction_peak: u64,
    entry_name_retained: u64,
    listing_construction_peak: u64,
    listing_retained: u64,
}

impl ArtifactTreeListingStorageRequirement {
    pub const fn maximum_entries(self) -> usize {
        self.maximum_entries
    }

    pub const fn provider_construction_peak_bytes(self) -> u64 {
        self.provider_construction_peak
    }

    pub const fn provider_retained_bytes(self) -> u64 {
        self.provider_retained
    }

    pub const fn entry_name_construction_peak_bytes(self) -> u64 {
        self.entry_name_construction_peak
    }

    pub const fn entry_name_retained_bytes(self) -> u64 {
        self.entry_name_retained
    }

    pub const fn listing_construction_peak_bytes(self) -> u64 {
        self.listing_construction_peak
    }

    pub const fn listing_retained_bytes(self) -> u64 {
        self.listing_retained
    }
}

impl ArtifactTreeMedia<'_> {
    /// Allocation-free and available before opening/enumerating the directory.
    /// `None` means no qualified bound, never zero cost or admission to proceed.
    pub fn listing_storage_requirement(
        &self,
        maximum_entries: usize,
    ) -> Option<ArtifactTreeListingStorageRequirement> {
        qualified_requirement(maximum_entries)
    }
}

#[cfg(not(windows))]
fn qualified_requirement(_: usize) -> Option<ArtifactTreeListingStorageRequirement> {
    // The Windows envelope is not evidence about libc or other providers.
    None
}

#[cfg(all(windows, feature = "recovery-runtime-owner"))]
pub(super) const fn provider_path_construction_bytes() -> u64 {
    // The fixed winx UTF-16 buffer coexists with its WTF-8 conversion growth.
    14 * 0x7fff
}

#[cfg(all(windows, feature = "recovery-runtime-owner"))]
pub(super) fn provider_iterator_storage(path: &std::path::PathBuf) -> Option<(u64, u64)> {
    use std::{
        alloc::Layout,
        mem::size_of,
        path::{Component, PathBuf},
    };
    let encoded = u64::try_from(path.as_os_str().len()).ok()?;
    let capacity = u64::try_from(path.capacity()).ok()?;
    let components = u64::try_from(path.components().count()).ok()?;
    let reconstructed = encoded.checked_add(2)?.checked_mul(2)?.max(8);
    let component_capacity = components.checked_add(1)?.checked_mul(2)?.max(4);
    let component_size = u64::try_from(size_of::<Component<'_>>()).ok()?;
    let arc = Layout::new::<[usize; 2]>()
        .extend(Layout::new::<PathBuf>())
        .ok()?
        .0
        .pad_to_align();
    let arc = u64::try_from(arc.size()).ok()?;
    let join = encoded
        .checked_mul(2)?
        .checked_add(reconstructed.checked_mul(2)?)?
        .checked_add(
            component_capacity
                .checked_mul(2)?
                .checked_mul(component_size)?,
        )?;
    // Pinned std to_u16s reserves encoded bytes + 1 units, not UTF-16 length.
    let search = encoded
        .checked_add(reconstructed)?
        .checked_add(encoded.checked_add(3)?.checked_mul(2)?)?
        .checked_add(arc)?;
    Some((
        capacity.checked_add(join.max(search))?,
        encoded.checked_add(arc)?,
    ))
}

#[cfg(windows)]
fn qualified_requirement(maximum_entries: usize) -> Option<ArtifactTreeListingStorageRequirement> {
    use super::ArtifactTreeDirectoryEntry;
    use std::{alloc::Layout, mem::size_of, path::Component, path::PathBuf};

    if maximum_entries == 0
        || env!("WORTH_STORE_MEDIA_COMPILER") != "rustc 1.94.0 (4a4ef493e 2026-03-02)"
    {
        return None;
    }
    // winx =0.36.4 get_file_path has a fixed 0x7fff-u16 buffer and rejects
    // overlong output. Rust 1.94 WTF-8 has <=3 bytes/unit and doubling growth.
    // std readdir clones root, verbatim-joins '*', converts to UTF-16, then
    // constructs Arc<PathBuf>. PathBuf::_push also builds a Component vector;
    // both that vector and reconstructed OsString include growth overlap.
    let wide = 0x7fff_u64;
    let encoded = 3 * wide;
    let returned_capacity = 6 * wide;
    let reconstructed_capacity = 2 * (encoded + 2);
    let component_capacity = 2 * (wide + 2);
    let component_size = u64::try_from(size_of::<Component<'_>>()).ok()?;
    let arc = Layout::new::<[usize; 2]>()
        .extend(Layout::new::<PathBuf>())
        .ok()?
        .0
        .pad_to_align()
        .size();
    let arc = u64::try_from(arc).ok()?;
    let path_join_peak = returned_capacity
        + 2 * encoded
        + 2 * reconstructed_capacity
        + 2 * component_capacity * component_size;
    let search_peak =
        returned_capacity + encoded + reconstructed_capacity + 2 * (encoded + 3) + arc;
    let provider_construction_peak = (2 * wide + 2 * returned_capacity)
        .max(path_join_peak)
        .max(search_peak);
    let provider_retained = encoded + arc;
    // std DirEntry stores WIN32_FIND_DATAW inline; cFileName has 260 u16 slots.
    let entry_name_retained = 6 * 260;
    let entry_name_construction_peak = 2 * entry_name_retained;
    let entries = u64::try_from(maximum_entries).ok()?;
    let slots =
        entries.checked_mul(u64::try_from(size_of::<ArtifactTreeDirectoryEntry>()).ok()?)?;
    // list_bounded starts at min(maximum_entries,4096), rejects before the
    // next push at the limit, and doubles only below it. Retained capacity is
    // <=2*limit; old-plus-new growth backing is <=3*limit.
    let names = entries.checked_mul(entry_name_retained)?;
    let listing_retained = slots.checked_mul(2)?.checked_add(names)?;
    let listing_construction_peak = slots
        .checked_mul(3)?
        .checked_add(names)?
        .checked_add(provider_construction_peak)?
        .checked_add(entry_name_construction_peak)?;
    Some(ArtifactTreeListingStorageRequirement {
        maximum_entries,
        provider_construction_peak,
        provider_retained,
        entry_name_construction_peak,
        entry_name_retained,
        listing_construction_peak,
        listing_retained,
    })
}
