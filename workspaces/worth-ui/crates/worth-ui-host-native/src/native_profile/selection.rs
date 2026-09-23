//! Selects one qualified profile for the compiled target.
//!
//! Windowing selection is a compile-time declaration, so a malformed selection
//! must fail at the compiler rather than be discovered by running a suite. The
//! arms below are closed and positive, and the guard rejects every value that
//! is not one of them — including the unset case.
//!
//! A negative arm (`not(worth_ui_windowing = "x11")`) was rejected: it absorbs
//! every value the author never enumerated, so `--cfg worth_ui_windowing="X11"`
//! would silently build the other profile. Measurement also showed no predicate
//! distinguishes *unset* from *set-to-garbage*, so a default arm and a garbage
//! detector cannot coexist.
//!
//! A `cfg` name may carry more than one value at once, so "set to both" is a
//! reachable state and is rejected explicitly below. Without that guard it
//! surfaces as a duplicate-definition error naming `ACTIVE_INDEX`, which points
//! at this file rather than at the build flags that caused it.

use super::{UiNativeCpuAdapterAdmission, UiNativeWindowingSystem};

#[cfg(all(
    target_os = "linux",
    not(any(worth_ui_windowing = "wayland", worth_ui_windowing = "x11"))
))]
compile_error!(
    "worth_ui_windowing must be exactly \"wayland\" or \"x11\" when building for Linux; \
     the default is declared in .cargo/config.toml [build] rustflags"
);

#[cfg(all(
    target_os = "linux",
    not(any(worth_ui_adapter = "hardware", worth_ui_adapter = "software"))
))]
compile_error!(
    "worth_ui_adapter must be exactly \"hardware\" or \"software\" when building for Linux; \
     the default is declared in .cargo/config.toml [build] rustflags"
);

#[cfg(all(
    target_os = "linux",
    worth_ui_adapter = "hardware",
    worth_ui_adapter = "software"
))]
compile_error!(
    "worth_ui_adapter carries both \"hardware\" and \"software\"; exactly one adapter class \
     is qualified per build. Override through cargo's encoded rust-flags environment variable, \
     which replaces, and re-include -C overflow-checks=on and the windowing flag."
);

/// No qualified profile drives Wayland through a software rasterizer: the
/// software class exists for the X server without DRI3 that the certification
/// lane runs under, and a Wayland compositor always presents through the GPU.
#[cfg(all(
    target_os = "linux",
    worth_ui_windowing = "wayland",
    worth_ui_adapter = "software"
))]
compile_error!(
    "no qualified profile pairs worth_ui_windowing=\"wayland\" with worth_ui_adapter=\"software\""
);

/// Off Linux neither axis applies and the Windows profile is the compiled
/// reference, matching the single unconditional manifest this crate carried
/// before the axes existed.
#[cfg(not(target_os = "linux"))]
pub(super) const ACTIVE_INDEX: usize = 0;
#[cfg(not(target_os = "linux"))]
pub(super) const SELECTED_WINDOWING_SYSTEM: UiNativeWindowingSystem =
    UiNativeWindowingSystem::Win32;
#[cfg(not(target_os = "linux"))]
pub(super) const SELECTED_CPU_ADAPTER: UiNativeCpuAdapterAdmission =
    UiNativeCpuAdapterAdmission::Deny;

#[cfg(all(
    target_os = "linux",
    worth_ui_windowing = "wayland",
    worth_ui_windowing = "x11"
))]
compile_error!(
    "worth_ui_windowing carries both \"wayland\" and \"x11\"; exactly one windowing system \
     is qualified per build. Cargo JOINS the rust-flags arrays it reads from configuration \
     sources rather than replacing them, so `--config build.rustflags=[..]` appends to the \
     default in .cargo/config.toml instead of overriding it. Override through cargo's encoded \
     rust-flags environment variable, which replaces, and re-include -C overflow-checks=on."
);

#[cfg(all(
    target_os = "linux",
    worth_ui_windowing = "wayland",
    worth_ui_adapter = "hardware"
))]
pub(super) const ACTIVE_INDEX: usize = 1;
#[cfg(all(target_os = "linux", worth_ui_windowing = "wayland"))]
pub(super) const SELECTED_WINDOWING_SYSTEM: UiNativeWindowingSystem =
    UiNativeWindowingSystem::Wayland;

#[cfg(all(
    target_os = "linux",
    worth_ui_windowing = "x11",
    worth_ui_adapter = "hardware"
))]
pub(super) const ACTIVE_INDEX: usize = 2;
#[cfg(all(
    target_os = "linux",
    worth_ui_windowing = "x11",
    worth_ui_adapter = "software"
))]
pub(super) const ACTIVE_INDEX: usize = 3;
#[cfg(all(target_os = "linux", worth_ui_windowing = "x11"))]
pub(super) const SELECTED_WINDOWING_SYSTEM: UiNativeWindowingSystem = UiNativeWindowingSystem::X11;

#[cfg(all(target_os = "linux", worth_ui_adapter = "hardware"))]
pub(super) const SELECTED_CPU_ADAPTER: UiNativeCpuAdapterAdmission =
    UiNativeCpuAdapterAdmission::Deny;
#[cfg(all(target_os = "linux", worth_ui_adapter = "software"))]
pub(super) const SELECTED_CPU_ADAPTER: UiNativeCpuAdapterAdmission =
    UiNativeCpuAdapterAdmission::Allow;
