#[cfg(not(windows))]
mod capability_entries;
#[cfg(all(target_os = "macos", feature = "recovery-runtime-owner"))]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub(super) use capability_entries::open;
#[cfg(all(target_os = "macos", feature = "recovery-runtime-owner"))]
pub(super) use macos::open_with_allocator;
#[cfg(windows)]
pub(super) use windows::open;
#[cfg(all(windows, feature = "recovery-runtime-owner"))]
pub(super) use windows::open_with_allocator;
