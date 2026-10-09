#[cfg(not(windows))]
mod capability_entries;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub(super) use capability_entries::open;
#[cfg(windows)]
pub(super) use windows::open;
#[cfg(all(windows, feature = "recovery-runtime-owner"))]
pub(super) use windows::open_with_allocator;
