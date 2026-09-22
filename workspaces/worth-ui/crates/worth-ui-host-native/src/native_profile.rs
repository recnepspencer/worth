mod appearance;
mod identity;
mod linux_wayland_vulkan_v1;
mod linux_x11_vulkan_software_v1;
mod linux_x11_vulkan_v1;
mod mechanics_capacities;
mod qualified_target;
mod selection;
mod surface;
mod windowing;
mod windows_dx12_v2;

pub(crate) use appearance::UiNativeAppearanceProfile;
pub use identity::UiNativePlatformProfileIdentity;
pub use mechanics_capacities::UiNativeMechanicsCapacities;
pub use qualified_target::UiNativeQualifiedTarget;
pub use surface::{
    UiNativeClientBackground, UiNativeCompositeAlpha, UiNativeCpuAdapterAdmission,
    UiNativePresentMode, UiNativeSurfaceBackends, UiNativeSurfaceFormat, UiNativeSurfaceProfile,
};
pub use windowing::UiNativeWindowingSystem;

/// One qualified (operating system, windowing system, graphics backend) triple.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeQualifiedProfile {
    pub(crate) identity: UiNativePlatformProfileIdentity,
    pub(crate) manifest: &'static str,
    pub(crate) windowing_system: UiNativeWindowingSystem,
    pub(crate) client_background: UiNativeClientBackground,
    /// The label the graphics backend stamps on a device it opens for this
    /// profile. Declared per profile rather than derived, because a device
    /// label is an observed string: `qualification_tests` proves each one
    /// against its identity so a copied profile cannot inherit another's name.
    pub(crate) device_label: &'static str,
    pub(crate) recovered_device_label: &'static str,
    pub(crate) surface: UiNativeSurfaceProfile,
    pub(crate) appearance: UiNativeAppearanceProfile,
}

/// Every qualified profile, compiled for every target.
///
/// This is deliberately not `cfg`-gated. The closed record asserts every
/// profile whichever host runs the suite, so a Linux run still proves the
/// Windows record; gating the modules would silently shrink that record to
/// whatever the host happens to be.
pub(crate) const QUALIFIED_PROFILES: [UiNativeQualifiedProfile; 4] = [
    windows_dx12_v2::PROFILE,
    linux_wayland_vulkan_v1::PROFILE,
    linux_x11_vulkan_v1::PROFILE,
    linux_x11_vulkan_software_v1::PROFILE,
];

pub(crate) const ACTIVE_PROFILE: UiNativeQualifiedProfile =
    QUALIFIED_PROFILES[selection::ACTIVE_INDEX];

/// Every qualified identity in [`QUALIFIED_PROFILES`] order, derived from that
/// array at compile time so a profile cannot be qualified without appearing
/// here. Anything outside this crate that must enumerate the qualified set
/// (the runtime's environment classifier proves each identity is admitted on
/// exactly one build) iterates this instead of keeping a second literal list
/// that a new profile can silently miss.
pub const WORTH_UI_QUALIFIED_PROFILE_IDENTITIES: [UiNativePlatformProfileIdentity;
    QUALIFIED_PROFILES.len()] = {
    let mut identities = [ACTIVE_PROFILE.identity; QUALIFIED_PROFILES.len()];
    let mut index = 0;
    while index < QUALIFIED_PROFILES.len() {
        identities[index] = QUALIFIED_PROFILES[index].identity;
        index += 1;
    }
    identities
};

pub const WORTH_UI_NATIVE_PROFILE_IDENTITY: UiNativePlatformProfileIdentity =
    ACTIVE_PROFILE.identity;
pub const WORTH_UI_NATIVE_PROFILE_MANIFEST: &str = ACTIVE_PROFILE.manifest;
pub const WORTH_UI_NATIVE_SURFACE_PROFILE: UiNativeSurfaceProfile = ACTIVE_PROFILE.surface;
pub const WORTH_UI_NATIVE_CLIENT_BACKGROUND: UiNativeClientBackground =
    ACTIVE_PROFILE.client_background;
pub const WORTH_UI_NATIVE_WINDOWING_SYSTEM: UiNativeWindowingSystem =
    ACTIVE_PROFILE.windowing_system;

/// The build flag and the selected profile name the same windowing system.
///
/// `selection.rs` turns the flag into an index; the profile at that index
/// declares its own windowing system. Both are constants, so an arm indexing
/// the wrong profile fails here rather than at the first forced event loop.
const _: () = assert!(
    ACTIVE_PROFILE
        .windowing_system
        .same_as(selection::SELECTED_WINDOWING_SYSTEM),
    "the selected qualified profile must declare the windowing system the build flag names"
);

/// The adapter flag and the selected profile agree the same way: the arm that
/// indexes the software profile must land on the one profile admitting a CPU
/// adapter, and every other arm on one that denies it.
const _: () = assert!(
    ACTIVE_PROFILE
        .surface
        .cpu_adapter
        .same_as(selection::SELECTED_CPU_ADAPTER),
    "the selected qualified profile must declare the adapter admission the build flag names"
);

pub(crate) const APPEARANCE_PROFILE: UiNativeAppearanceProfile = ACTIVE_PROFILE.appearance;
