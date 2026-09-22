//! Asserts each qualified profile's Rust constants against the manifest it
//! includes.
//!
//! Split from the parent test module because the parent reached its line cap:
//! these assertions are one cohesive responsibility — proving manifest-to-code
//! correspondence for a single profile — and travel together.

use super::native_profile_record::QualifiedProfileRecord;
use super::{assert_exact_manifest, manifest};
use crate::native_profile::{UiNativeAppearanceProfile, UiNativeQualifiedProfile};

/// Asserts one qualified profile's Rust constants against the manifest it
/// includes, field by field.
///
/// Every comparison crosses the manifest boundary — a Rust constant against a
/// parsed TOML value. A constant compared to its own literal proves nothing,
/// and a module that pairs one profile's `include_str!` with another profile's
/// constants fails here on `identity` rather than compiling clean.
pub(super) fn assert_qualified_profile(
    profile: &UiNativeQualifiedProfile,
    record: &QualifiedProfileRecord,
) {
    let declared = manifest(profile.manifest);
    assert_exact_manifest(&declared, record.strings, record.integers, record.booleans);
    let identity = profile.identity.as_str();
    assert_eq!(declared["identity"].as_str(), Some(identity));
    assert_eq!(
        declared["client_background"].as_str(),
        Some(profile.client_background.as_str()),
        "client_background for {identity}"
    );
    assert_eq!(
        declared["windowing_system"].as_str(),
        Some(profile.windowing_system.as_str()),
        "windowing_system for {identity}"
    );
    assert_qualified_surface_strings(&declared, profile, identity);
    assert_qualified_appearance_capacities(&declared, profile.appearance, identity);
    assert_qualified_device_labels(profile, identity);
}

/// Asserts the string-valued surface and appearance axes against the manifest.
///
/// These are the axes this change exists to bind: a profile declaring `Opaque`
/// against a manifest declaring `PreMultiplied` fails here rather than at its
/// first reader.
fn assert_qualified_surface_strings(
    declared: &toml::Value,
    profile: &UiNativeQualifiedProfile,
    identity: &str,
) {
    let surface = profile.surface;
    let appearance = profile.appearance;
    for (key, qualified) in [
        ("runtime_backend_selection", surface.backends.as_str()),
        ("surface_format", surface.surface_format.as_str()),
        ("target_format", surface.target_format.as_str()),
        ("present_mode", surface.present_mode.as_str()),
        ("composite_alpha", surface.composite_alpha.as_str()),
        ("cpu_adapter", surface.cpu_adapter.as_str()),
        ("appearance_antialiasing", geometry_basis(appearance)),
        ("appearance_qualified_scales", scales(appearance)),
        ("appearance_primary_pointer", primary_pointer(appearance)),
    ] {
        assert_eq!(
            declared[key].as_str(),
            Some(qualified),
            "surface {key} for {identity}"
        );
    }
    assert_eq!(
        declared["identity"].as_str(),
        Some(appearance.identity),
        "appearance identity for {identity}"
    );
}

/// Asserts the integer appearance capacities against the manifest.
///
/// These size real arenas, so a capacity drifting from its qualified record
/// would hand the product an arena the manifest never certified.
fn assert_qualified_appearance_capacities(
    declared: &toml::Value,
    appearance: UiNativeAppearanceProfile,
    identity: &str,
) {
    for (key, qualified) in [
        ("appearance_version", i64::from(appearance.version)),
        (
            "appearance_anti_alias_fringe_physical_pixels",
            i64::from(appearance.anti_alias_fringe_physical_pixels),
        ),
        ("retained_commands", i64::from(appearance.retained_commands)),
        ("surface_commands", i64::from(appearance.surface_commands)),
        ("outline_commands", i64::from(appearance.outline_commands)),
        ("backdrop_commands", i64::from(appearance.backdrop_commands)),
        (
            "overlay_order_commands",
            i64::from(appearance.overlay_order_commands),
        ),
        (
            "pointer_affordance_commands",
            i64::from(appearance.pointer_affordance_commands),
        ),
        (
            "text_commands",
            i64::from(appearance.text_foreground_commands),
        ),
        ("damage_regions", i64::from(appearance.damage_regions)),
    ] {
        assert_eq!(
            declared[key].as_integer(),
            Some(qualified),
            "appearance {key} for {identity}"
        );
    }
}

/// Asserts each profile's device labels are derived from its own identity.
///
/// A device label is an observed string, so a profile copied from another must
/// not inherit the source profile's name for a device it opens.
fn assert_qualified_device_labels(profile: &UiNativeQualifiedProfile, identity: &str) {
    assert_eq!(
        (profile.device_label, profile.recovered_device_label),
        (
            format!("{identity}-device").as_str(),
            format!("{identity}-recovered-device").as_str()
        ),
        "device labels for {identity}"
    );
}

/// The manifest spelling of a qualified geometry basis.
///
/// Written as a `match` rather than a stored string so a new basis variant
/// fails to compile here instead of silently losing its manifest counterpart.
fn geometry_basis(appearance: UiNativeAppearanceProfile) -> &'static str {
    match appearance.geometry_basis {
        worth_ui_host_contract::UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter => {
            "analytic-signed-distance-pixel-center"
        }
    }
}

/// The manifest spelling of a qualified scale ladder, rendered from the closed
/// four-row table rather than restated beside it.
fn scales(appearance: UiNativeAppearanceProfile) -> &'static str {
    const LADDER: [(&[u16; 4], &str); 1] = [(&[1_000, 1_250, 1_500, 2_000], "1.0;1.25;1.5;2.0")];
    LADDER
        .iter()
        .find(|(rows, _)| *rows == appearance.scales_milli)
        .map(|(_, spelled)| *spelled)
        .expect("qualified scale ladder has a manifest spelling")
}

/// The manifest spelling of a qualified primary pointer.
///
/// Exhaustive over the kind *and* over its absence, so adding a pointer kind
/// fails to compile here rather than silently losing its manifest counterpart.
fn primary_pointer(appearance: UiNativeAppearanceProfile) -> &'static str {
    match appearance.primary_pointer {
        Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse) => "mouse",
        Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Pen) => "pen",
        None => "none",
    }
}
