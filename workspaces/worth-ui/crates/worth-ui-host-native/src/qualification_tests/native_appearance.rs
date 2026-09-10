use super::super::native_profile::{
    UiNativePlatformProfileIdentity, APPEARANCE_PROFILE, WORTH_UI_NATIVE_PROFILE_MANIFEST,
};

use worth_ui_host_contract::{
    UiHostAppearanceProfilePosture, UiMountedTextSchemaVersion, WorthUiHostMechanicsAdapter,
};

#[cfg(feature = "certification-support")]
use crate::native::APPEARANCE_MECHANICS;

#[cfg(feature = "certification-support")]
use worth_ui_host_contract::{
    UiHostAppearanceGeometryQualificationBasis, UiHostAppearanceScaleDenial,
    UiHostPrimaryPointerKind,
};

#[test]
fn current_v2_profile_carries_the_native_appearance_long_pole() {
    let live = parse(WORTH_UI_NATIVE_PROFILE_MANIFEST);
    assert_eq!(
        live["identity"].as_str(),
        Some(UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V2.as_str())
    );
    assert_eq!(live["identity"].as_str(), Some(APPEARANCE_PROFILE.identity));
    assert_eq!(live["profile_stage"].as_str(), Some("current"));
    assert_eq!(live["live_emission"].as_str(), Some("enabled"));
    assert_eq!(
        live["appearance_surface_pipeline"].as_str(),
        Some("rounded-fill-inward-border")
    );
    assert_eq!(
        live["appearance_outline_pipeline"].as_str(),
        Some("outside-ring-full-fringe")
    );
    assert_eq!(
        live["appearance_antialiasing"].as_str(),
        Some("analytic-signed-distance-pixel-center")
    );
    assert_eq!(
        live["appearance_anti_alias_fringe_physical_pixels"].as_integer(),
        Some(1)
    );
    assert_eq!(
        live["appearance_qualified_scales"].as_str(),
        Some("1.0;1.25;1.5;2.0")
    );
}

#[test]
fn current_profile_capacity_and_scale_constants_match_the_checked_manifest() {
    let live = parse(WORTH_UI_NATIVE_PROFILE_MANIFEST);
    assert_eq!(
        live["retained_commands"].as_integer(),
        Some(i64::from(APPEARANCE_PROFILE.retained_commands))
    );
    assert_eq!(
        live["surface_commands"].as_integer(),
        Some(i64::from(APPEARANCE_PROFILE.surface_commands))
    );
    assert_eq!(
        live["outline_commands"].as_integer(),
        Some(i64::from(APPEARANCE_PROFILE.outline_commands))
    );
    assert_eq!(
        live["backdrop_commands"].as_integer(),
        Some(i64::from(APPEARANCE_PROFILE.backdrop_commands))
    );
    assert_eq!(
        live["overlay_order_commands"].as_integer(),
        Some(i64::from(APPEARANCE_PROFILE.overlay_order_commands))
    );
    assert_eq!(
        live["pointer_affordance_commands"].as_integer(),
        Some(i64::from(APPEARANCE_PROFILE.pointer_affordance_commands))
    );
    assert_eq!(
        live["text_commands"].as_integer(),
        Some(i64::from(APPEARANCE_PROFILE.text_foreground_commands))
    );
    assert_eq!(
        live["damage_regions"].as_integer(),
        Some(i64::from(APPEARANCE_PROFILE.damage_regions))
    );
    assert_eq!(
        APPEARANCE_PROFILE.scales_milli,
        &[1_000, 1_250, 1_500, 2_000]
    );
    assert_eq!(APPEARANCE_PROFILE.anti_alias_fringe_physical_pixels, 1);
}

#[cfg(feature = "certification-support")]
#[test]
fn certification_report_uses_the_current_host_owned_mechanic_qualification() {
    let report = crate::appearance_capability_report();
    let profile = report
        .appearance_profile()
        .expect("certification report must carry the current appearance profile");

    assert_eq!(profile.posture(), UiHostAppearanceProfilePosture::Current);
    assert_eq!(profile.identity(), APPEARANCE_PROFILE.identity);
    assert_eq!(profile.version(), APPEARANCE_PROFILE.version);
    assert_eq!(
        profile.mechanics(),
        &APPEARANCE_MECHANICS,
        "native qualification must enumerate each supported mechanic explicitly"
    );
    assert_eq!(
        profile.primary_pointer(),
        Some(UiHostPrimaryPointerKind::Mouse)
    );
}

#[cfg(feature = "certification-support")]
#[test]
fn certification_report_carries_each_native_scale_fringe_enclosure_without_fallback() {
    let report = crate::appearance_capability_report();
    let profile = report
        .appearance_profile()
        .expect("certification report must carry the current appearance profile");
    let rows = profile.geometry_qualification().rows();
    assert_eq!(
        rows.iter()
            .map(|row| row.device_scale_milli())
            .collect::<Vec<_>>(),
        vec![1_000, 1_250, 1_500, 2_000]
    );
    assert_eq!(
        rows.iter()
            .map(|row| row.anti_alias_fringe_physical_pixels())
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 1]
    );
    assert_eq!(
        rows.iter()
            .map(|row| row.anti_alias_fringe_logical_subpixels().subpixels())
            .collect::<Vec<_>>(),
        vec![1_000, 800, 667, 500]
    );
    assert!(rows.iter().all(|row| row.basis()
        == UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter));
    assert_eq!(
        profile.geometry_qualification().row_for_scale(1_333),
        Err(UiHostAppearanceScaleDenial::UnsupportedScale(1_333))
    );
}

#[test]
fn live_native_preparation_uses_v2_appearance_and_cutover_protocol() {
    let (mechanics, _event_loop) =
        crate::WorthUiPreparedNativeHost::prepare_qualified().into_parts(
            crate::UiNativeWindowConfiguration::qualified("host-admission", [800, 600]),
        );
    let report = WorthUiHostMechanicsAdapter::mechanical_capability_report(&mechanics);
    let protocol = WorthUiHostMechanicsAdapter::mechanical_protocol_contract(&mechanics);

    assert_eq!(
        report.appearance_profile().map(|profile| profile.posture()),
        Some(UiHostAppearanceProfilePosture::Current)
    );
    assert_eq!(
        WORTH_UI_NATIVE_PROFILE_MANIFEST
            .parse::<toml::Value>()
            .expect("live native profile manifest parses")["identity"]
            .as_str(),
        Some(UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V2.as_str())
    );
    assert_eq!(protocol.protocol().revision(), 7);
    assert_eq!(protocol.mounted_frame().revision(), 6);
    assert_eq!(protocol.mounted_presentation().revision(), 6);
    assert_eq!(protocol.observation().revision(), 7);
    assert_eq!(protocol.measurement().revision(), 5);
    assert_eq!(protocol.solicited_effect().revision(), 1);
    assert_eq!(UiMountedTextSchemaVersion::current().revision(), 4);
}

fn parse(manifest: &str) -> toml::Value {
    manifest.parse().expect("profile manifest parses")
}
