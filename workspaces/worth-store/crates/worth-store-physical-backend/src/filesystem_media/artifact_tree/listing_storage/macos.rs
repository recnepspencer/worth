//! Audited Darwin LP64 streaming provider storage, not an OS capability claim.
use super::ArtifactTreeListingStorageRequirement;
use std::{alloc::Layout, mem::size_of};

// Darwin readdir may return a name beyond the inline d_name array. Its actual
// d_namlen ABI is u16; do not infer a bound from sizeof(d_name) or NAME_MAX.
const _: fn(&libc::dirent) -> u16 = |entry| entry.d_namlen;
pub(in crate::filesystem_media::artifact_tree) const CSTRING_BYTES: u64 = u16::MAX as u64 + 1;
pub(in crate::filesystem_media::artifact_tree) const NAME_BYTES: u64 = u16::MAX as u64;
const INITIAL_BUFFER: u64 = 2 * 1024;
const LARGE_BUFFER: u64 = 8 * 1024;

fn struct_bytes(fields: &[Layout]) -> Option<u64> {
    let mut layout = Layout::from_size_align(0, 1).ok()?;
    for field in fields {
        layout = layout.extend(*field).ok()?.0;
    }
    u64::try_from(layout.pad_to_align().size()).ok()
}

pub(in crate::filesystem_media::artifact_tree) fn provider_bounds() -> Option<(u64, u64)> {
    if !super::super::path_storage::qualified() {
        return None;
    }
    // Public Darwin SDK DIR layout. _telldir is from the pinned Apple source;
    // it is NOT a public libc type. No tell/seek/rewind creates ddloc records.
    let pointer = Layout::new::<*mut ()>();
    let header = struct_bytes(&[
        Layout::new::<libc::c_int>(),
        Layout::new::<libc::size_t>(),
        Layout::new::<libc::size_t>(),
        pointer,
        Layout::new::<libc::c_int>(),
        Layout::new::<libc::c_long>(),
        Layout::new::<libc::c_long>(),
        Layout::new::<libc::c_int>(),
        Layout::new::<libc::pthread_mutex_t>(),
        pointer,
    ])?;
    let position = struct_bytes(&[
        pointer,
        Layout::new::<libc::c_long>(),
        Layout::new::<libc::off_t>(),
    ])?;
    let retained = header.checked_add(position)?.checked_add(LARGE_BUFFER)?;
    // malloc(new) precedes free(old) during the single 2 KiB -> 8 KiB growth.
    Some((retained.checked_add(INITIAL_BUFFER)?, retained))
}

pub(super) fn requirement(maximum_entries: usize) -> Option<ArtifactTreeListingStorageRequirement> {
    if maximum_entries == 0 {
        return None;
    }
    let (provider_construction_peak, provider_retained) = provider_bounds()?;
    let entries = u64::try_from(maximum_entries).ok()?;
    let slots = entries
        .checked_mul(u64::try_from(size_of::<super::super::ArtifactTreeDirectoryEntry>()).ok()?)?;
    let names = entries.checked_mul(NAME_BYTES)?;
    let listing_retained = slots.checked_mul(2)?.checked_add(names)?;
    let entry_name_construction_peak = CSTRING_BYTES.checked_add(NAME_BYTES)?;
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
        entry_name_retained: NAME_BYTES,
        listing_construction_peak,
        listing_retained,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supported_darwin_layout_includes_header_position_and_buffer_overlap() {
        // Independent native SDK probe: sizeof(DIR)=136, pthread_mutex=64,
        // long/off_t/pointer=8. Pinned Apple _telldir is three pointer-width fields.
        assert_eq!(
            provider_bounds(),
            Some((136 + 24 + 2048 + 8192, 136 + 24 + 8192))
        );
        assert_eq!((CSTRING_BYTES, NAME_BYTES), (65536, 65535));
        assert!(requirement(0).is_none());
        assert!(requirement(usize::MAX).is_none());
    }
}
