use super::ArtifactTreeMedia;
#[cfg(feature = "recovery-runtime-owner")]
use super::{path::validate_component, ArtifactTreeDirectory};

pub(super) fn qualified() -> bool {
    cfg!(windows) && env!("WORTH_STORE_MEDIA_PATH_PROFILE") == "qualified"
}

#[cfg(feature = "recovery-runtime-owner")]
pub(super) fn child_bytes(directory: &ArtifactTreeDirectory, component: &str) -> Option<u64> {
    validate_component(component).ok()?;
    let count = directory.components.len();
    if count >= 8 {
        return None;
    }
    // Vec::clone reserves len slots. The subsequent push may retain those
    // slots while requesting its doubled/minimum-four replacement.
    let new = count.checked_mul(2)?.max(4);
    directory
        .components
        .iter()
        .try_fold(
            count
                .checked_add(new)?
                .checked_mul(std::mem::size_of::<String>())?
                .checked_add(component.len())?,
            |bytes, value| bytes.checked_add(value.len()),
        )
        .and_then(|bytes| u64::try_from(bytes).ok())
}

#[cfg(feature = "recovery-runtime-owner")]
pub(super) fn file_bytes(directory: &ArtifactTreeDirectory, component: &str) -> Option<u64> {
    validate_component(component).ok()?;
    directory
        .components
        .iter()
        .try_fold(
            directory
                .components
                .len()
                .checked_mul(std::mem::size_of::<String>())?
                .checked_add(component.len())?,
            |bytes, value| bytes.checked_add(value.len()),
        )
        .and_then(|bytes| u64::try_from(bytes).ok())
}

#[cfg(feature = "recovery-runtime-owner")]
pub(super) fn open_bytes(component: &str) -> Option<u64> {
    if !qualified() {
        return None;
    }
    validate_component(component).ok()?;
    #[cfg(not(windows))]
    {
        None
    }
    #[cfg(windows)]
    {
        use std::{alloc::Layout, borrow::Cow, ffi::OsStr, fs::File};
        // cap-primitives 4.0.2 private CowComponent has one Cow payload and
        // four variants. Pinned rustc's tagged upper bound is payload+word;
        // a niche layout can only reduce it. MaybeOwned is the actual public
        // dependency type; non-racy MaybeOwnedFile has only this field.
        if Layout::new::<Cow<'_, OsStr>>().align() > Layout::new::<usize>().align() {
            return None;
        }
        let cow = Layout::new::<Cow<'_, OsStr>>()
            .extend(Layout::new::<usize>())
            .ok()?
            .0
            .pad_to_align()
            .size();
        let slots = cow
            .checked_mul(4)?
            .checked_add(std::mem::size_of::<maybe_owned::MaybeOwned<'_, File>>())?;
        let encoded = component.len();
        let wide = component.encode_utf16().count();
        // Single Normal component: empty rebuilt PathBuf grows to max(8,B).
        // Vec<u16> collection/growth has capacity <=2*max(4,U); charge its
        // old+replacement peak, not merely final length. No dot/symlink walk.
        let normal = slots
            .checked_add(encoded.max(8))?
            .checked_add(wide.max(4).checked_mul(6)?)?;
        // cap's device check temporarily uppercases the stem before Context
        // exists. Count the exact Unicode output without constructing it.
        // Whole-component census conservatively covers cap's file_prefix,
        // including leading-dot names (whose first dot is not a separator).
        let upper = component
            .chars()
            .flat_map(char::to_uppercase)
            .try_fold(0usize, |bytes, ch| bytes.checked_add(ch.len_utf8()))?;
        let uppercase = component.len().max(upper).max(8).checked_mul(3)?;
        u64::try_from(normal.max(uppercase)).ok()
    }
}

impl ArtifactTreeMedia<'_> {
    /// No allocation, opening, or ambient path authority is involved.
    /// This build-profile qualification assumes unmodified audited pinned
    /// dependency sources. Its Cargo.lock metadata checks do not attest source
    /// bytes or detect registry replacement/vendor substitution.
    pub fn path_storage_is_qualified(&self) -> bool {
        qualified()
    }
}
