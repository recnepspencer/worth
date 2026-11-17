#[cfg(not(windows))]
mod capability_entries;
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    feature = "recovery-runtime-owner"
))]
mod posix;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub(super) use capability_entries::open;
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    feature = "recovery-runtime-owner"
))]
pub(super) use posix::open_with_allocator;
#[cfg(windows)]
pub(super) use windows::open;
#[cfg(all(windows, feature = "recovery-runtime-owner"))]
pub(super) use windows::open_with_allocator;
