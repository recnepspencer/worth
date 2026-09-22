use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

use super::{
    UiBodyDefaultAtlasCapacities, UiNativeMechanicsCapacities, WORTH_UI_BODY_DEFAULT_FONT,
    WORTH_UI_BODY_DEFAULT_LICENSE, WORTH_UI_NATIVE_PROFILE_MANIFEST,
    WORTH_UI_TEXT_PROFILE_MANIFEST,
};

mod font_coverage;
mod native_appearance;
mod native_profile_record;
mod qualified_dependencies;
mod qualified_profile;

use crate::native_profile::QUALIFIED_PROFILES;
use native_profile_record::QUALIFIED_PROFILE_RECORDS;
use qualified_profile::assert_qualified_profile;

#[test]
fn qualified_asset_license_and_manifests_have_exact_digests() {
    assert_eq!(
        sha256(WORTH_UI_BODY_DEFAULT_FONT),
        "478c558ea716033cd60c03438f628dfa75694dcf6b5f6d505a2f05fd2b4f3823"
    );
    assert_eq!(
        sha256(WORTH_UI_BODY_DEFAULT_LICENSE.as_bytes()),
        "cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a"
    );
    assert_eq!(
        sha256(WORTH_UI_TEXT_PROFILE_MANIFEST.as_bytes()),
        "6f140249866e6815e9284fe1c8c959a8bb1b8cab252cfbe8c7c397f9a7eb9b01"
    );
    for (profile, record) in QUALIFIED_PROFILES
        .iter()
        .zip(QUALIFIED_PROFILE_RECORDS.iter())
    {
        assert_eq!(
            sha256(profile.manifest.as_bytes()),
            record.manifest_sha256,
            "manifest digest for {}",
            profile.identity.as_str()
        );
    }
}

#[test]
fn qualified_capacity_types_match_the_canonical_manifests() {
    let text = manifest(WORTH_UI_TEXT_PROFILE_MANIFEST);
    let platform = manifest(WORTH_UI_NATIVE_PROFILE_MANIFEST);
    let atlas = UiBodyDefaultAtlasCapacities::QUALIFIED;
    let native = UiNativeMechanicsCapacities::QUALIFIED;
    assert_eq!(
        (
            atlas.pages,
            atlas.page_width,
            atlas.page_height,
            atlas.entries,
            atlas.texel_bytes,
            atlas.glyph_width,
            atlas.glyph_height,
            atlas.staged_upload_bytes,
        ),
        (
            integer(&text, "atlas_pages") as u8,
            integer(&text, "atlas_page_width") as u16,
            integer(&text, "atlas_page_height") as u16,
            integer(&text, "atlas_entries") as u16,
            integer(&text, "atlas_texel_bytes") as u32,
            integer(&text, "glyph_max_width") as u16,
            integer(&text, "glyph_max_height") as u16,
            integer(&text, "staged_upload_bytes") as u32,
        )
    );
    assert_eq!(
        (
            native.retained_commands,
            native.rectangle_commands,
            native.text_commands,
            native.damage_regions,
            native.order_edits,
            native.text_bytes,
            native.readiness_owners,
            native.causes_per_owner,
            native.ready_owner_slots,
            native.presentation_slots,
            native.readback_slots,
            native.readback_bytes,
        ),
        (
            integer(&platform, "retained_commands") as u16,
            integer(&platform, "rectangle_commands") as u16,
            integer(&platform, "text_commands") as u16,
            integer(&platform, "damage_regions") as u16,
            integer(&platform, "order_edits") as u16,
            integer(&platform, "text_bytes") as u32,
            integer(&platform, "readiness_owners") as u8,
            integer(&platform, "causes_per_owner") as u8,
            integer(&platform, "ready_owner_slots") as u8,
            integer(&platform, "presentation_slots") as u8,
            integer(&platform, "readback_slots") as u8,
            integer(&platform, "readback_bytes") as u32,
        )
    );
    assert_eq!(
        native.resource_registry_entries,
        integer(&platform, "resource_registry_entries") as u8,
    );
}

/// The `worth_ui_windowing` and `worth_ui_adapter` spellings and the selected
/// profile's declared axes are independent tables; this is the second binding.
/// `native_profile.rs` binds the selection arm to the profile it indexes, but
/// an arm can misdeclare its own meaning, and both sides of that assert then
/// move together. Here the expected spellings come straight from the cfgs,
/// and the pair must pick out exactly one qualified profile: two profiles
/// share `x11`, so the windowing axis alone no longer names a profile.
#[test]
fn the_build_flag_spellings_select_the_profile_declaring_those_axes() {
    #[cfg(not(target_os = "linux"))]
    const FLAGGED_WINDOWING: &str = "win32";
    #[cfg(all(target_os = "linux", worth_ui_windowing = "wayland"))]
    const FLAGGED_WINDOWING: &str = "wayland";
    #[cfg(all(target_os = "linux", worth_ui_windowing = "x11"))]
    const FLAGGED_WINDOWING: &str = "x11";
    #[cfg(any(not(target_os = "linux"), worth_ui_adapter = "hardware"))]
    const FLAGGED_ADAPTER: &str = "deny";
    #[cfg(all(target_os = "linux", worth_ui_adapter = "software"))]
    const FLAGGED_ADAPTER: &str = "allow";
    assert_eq!(
        crate::native_profile::WORTH_UI_NATIVE_WINDOWING_SYSTEM.as_str(),
        FLAGGED_WINDOWING,
        "the selected profile's windowing_system must be the one the build flag names"
    );
    assert_eq!(
        crate::native_profile::WORTH_UI_NATIVE_SURFACE_PROFILE
            .cpu_adapter
            .as_str(),
        FLAGGED_ADAPTER,
        "the selected profile's cpu_adapter must be the one the build flag names"
    );
    let declaring = crate::native_profile::QUALIFIED_PROFILES
        .iter()
        .filter(|profile| {
            profile.windowing_system.as_str() == FLAGGED_WINDOWING
                && profile.surface.cpu_adapter.as_str() == FLAGGED_ADAPTER
        })
        .map(|profile| profile.identity.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        declaring,
        [crate::native_profile::WORTH_UI_NATIVE_PROFILE_IDENTITY.as_str()],
        "exactly the profile declaring the flagged windowing system and adapter admission must be active"
    );
}

/// Admission is a certification-only axis: exactly one qualified profile may
/// admit a software rasterizer, and it is the X11 software profile. Asserted on
/// every host and under every flag, so a product profile cannot drift to
/// `allow` behind a flag nobody builds.
#[test]
fn exactly_one_qualified_profile_admits_a_software_rasterizer() {
    let admitting = crate::native_profile::QUALIFIED_PROFILES
        .iter()
        .filter(|profile| profile.surface.cpu_adapter.as_str() == "allow")
        .map(|profile| profile.identity.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        admitting,
        ["worth-ui-linux-x11-vulkan-software-v1"],
        "exactly one qualified profile admits a software rasterizer, and it is the certification one"
    );
}

#[test]
fn every_qualified_semantic_and_dependency_pin_matches_the_closed_record() {
    #[cfg(target_os = "windows")]
    assert_eq!(
        crate::native::QUALIFIED_DX12_PRESENTATION_SYSTEM,
        wgpu::Dx12SwapchainKind::DxgiFromVisual
    );
    let text = manifest(WORTH_UI_TEXT_PROFILE_MANIFEST);
    assert_exact_manifest(
        &text,
        TEXT_STRING_FIELDS,
        TEXT_INTEGER_FIELDS,
        TEXT_BOOL_FIELDS,
    );
    assert_eq!(
        integer(&text, "asset_bytes"),
        WORTH_UI_BODY_DEFAULT_FONT.len() as i64
    );
    assert_eq!(
        text.get("subpixel").and_then(toml::Value::as_bool),
        Some(false)
    );
    for (profile, record) in QUALIFIED_PROFILES
        .iter()
        .zip(QUALIFIED_PROFILE_RECORDS.iter())
    {
        assert_qualified_profile(profile, record);
    }
    let active = manifest(WORTH_UI_NATIVE_PROFILE_MANIFEST);
    assert_eq!(
        crate::native::GPU_WAIT_DEADLINE.as_millis(),
        integer(&active, "gpu_wait_deadline_ms") as u128,
    );
    assert_eq!(
        integer(&active, "wheel_line_logical_subpixels"),
        crate::native_profile::QUALIFIED_WHEEL_LINE_LOGICAL_SUBPIXELS,
    );
    qualified_dependencies::assert_qualified_dependencies();
}

const TEXT_STRING_FIELDS: &[(&str, &str)] = &[
    ("identity", "worth-ui-body-default-v1"),
    ("asset", "NotoSans-Regular.ttf"),
    (
        "asset_sha256",
        "478c558ea716033cd60c03438f628dfa75694dcf6b5f6d505a2f05fd2b4f3823",
    ),
    ("upstream_release", "NotoSans-v2.015"),
    (
        "upstream_commit",
        "c4a321e123e4d4ff315f57f4e0adf294fe3a95be",
    ),
    (
        "upstream_asset_path",
        "NotoSans/hinted/ttf/NotoSans-Regular.ttf",
    ),
    (
        "archive_sha256",
        "0c34df072a3fa7efbb7cbf34950e1f971a4447cffe365d3a359e2d4089b958f5",
    ),
    ("license", "SIL Open Font License 1.1"),
    ("license_file", "OFL.txt"),
    (
        "license_sha256",
        "cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a",
    ),
    ("support_start", "U+0020"),
    ("support_end", "U+007E"),
    ("normalization", "none"),
    ("direction", "horizontal-ltr"),
    ("language", "none"),
    ("script", "none"),
    ("fallback", "none"),
    ("shaper", "rustybuzz-0.20.1"),
    ("rasterizer", "swash-0.2.10"),
    ("baseline", "alphabetic"),
    ("wrap", "clip"),
    ("hinting", "hinted"),
    ("coverage", "grayscale"),
    (
        "rounding",
        "origin-nearest-ties-even;bounds-floor-ceil;clip-half-open",
    ),
    (
        "dpi_basis",
        "event-time-logical-size-times-scale-generation",
    ),
    ("unsupported", "typed-denial-before-effects"),
    (
        "unsupported_preserves",
        "semantic-projection-and-predecessor-publication",
    ),
    ("live_glyph_posture", "retained-commands-pin-entries"),
    ("candidate_eviction", "unpinned-candidate-only"),
    ("saturation", "deny-before-upload-no-growth-no-fallback"),
    (
        "qualification_observation",
        "asset-license-profile-dependency-digest-v1",
    ),
];

const TEXT_INTEGER_FIELDS: &[(&str, i64)] = &[
    ("asset_bytes", 621_572),
    ("archive_bytes", 117_491_253),
    ("license_bytes", 4_396),
    ("run_count", 1),
    ("size_millipoints", 14_000),
    ("weight", 400),
    ("atlas_pages", 4),
    ("atlas_page_width", 1_024),
    ("atlas_page_height", 1_024),
    ("atlas_entries", 4_096),
    ("atlas_texel_bytes", 4_194_304),
    ("glyph_max_width", 256),
    ("glyph_max_height", 256),
    ("staged_upload_bytes", 1_048_576),
];

const TEXT_BOOL_FIELDS: &[(&str, bool)] = &[("subpixel", false)];

fn assert_exact_manifest(
    manifest: &toml::Value,
    strings: &[(&str, &str)],
    integers: &[(&str, i64)],
    booleans: &[(&str, bool)],
) {
    let expected = strings
        .iter()
        .map(|(key, _)| *key)
        .chain(integers.iter().map(|(key, _)| *key))
        .chain(booleans.iter().map(|(key, _)| *key))
        .collect::<BTreeSet<_>>();
    let observed = manifest
        .as_table()
        .expect("qualified manifest table")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed, expected,
        "canonical manifest field inventory drifted"
    );
    for (key, value) in strings {
        assert_eq!(manifest[*key].as_str(), Some(*value), "string {key}");
    }
    for (key, value) in integers {
        assert_eq!(manifest[*key].as_integer(), Some(*value), "integer {key}");
    }
    for (key, value) in booleans {
        assert_eq!(manifest[*key].as_bool(), Some(*value), "boolean {key}");
    }
}

fn manifest(text: &str) -> toml::Value {
    text.parse().expect("qualified manifest parses")
}

fn integer(manifest: &toml::Value, key: &str) -> i64 {
    manifest
        .get(key)
        .and_then(toml::Value::as_integer)
        .unwrap_or_else(|| panic!("qualified integer `{key}`"))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
