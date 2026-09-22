mod linux_wayland_vulkan_v1;
mod linux_x11_vulkan_software_v1;
mod linux_x11_vulkan_v1;
mod windows_dx12_v2;

/// One qualified native profile's closed record, stated independently of the
/// manifest it pins.
///
/// The record carries no identity of its own: the identity it expects is the
/// `identity` entry of [`Self::strings`], so a record paired with the wrong
/// manifest fails on that string rather than passing silently. This is the
/// mis-pairing failure a constant-versus-its-own-literal check cannot see.
pub(super) struct QualifiedProfileRecord {
    pub(super) manifest_sha256: &'static str,
    pub(super) strings: &'static [(&'static str, &'static str)],
    pub(super) integers: &'static [(&'static str, i64)],
    pub(super) booleans: &'static [(&'static str, bool)],
}

/// Index-aligned with [`crate::native_profile::QUALIFIED_PROFILES`].
///
/// Every profile is asserted on every target, not only the `cfg`-selected one,
/// so a Linux run still proves the Windows record and a Windows run still
/// proves all three Linux records.
pub(super) const QUALIFIED_PROFILE_RECORDS: [QualifiedProfileRecord; 4] = [
    windows_dx12_v2::RECORD,
    linux_wayland_vulkan_v1::RECORD,
    linux_x11_vulkan_v1::RECORD,
    linux_x11_vulkan_software_v1::RECORD,
];

/// Holds the index alignment above as a compile-time fact rather than a
/// comment.
///
/// `zip` stops at the shorter side, so a profile added without its record
/// would simply go unasserted and every test would stay green. This fails the
/// build instead.
const _: () = assert!(
    QUALIFIED_PROFILE_RECORDS.len() == crate::native_profile::QUALIFIED_PROFILES.len(),
    "every qualified profile owns a closed record"
);
