use super::super::native_profile::{
    UiNativePlatformProfileIdentity, STAGED_APPEARANCE_PROFILE, WORTH_UI_NATIVE_NEXT_PROFILE_MANIFEST,
    WORTH_UI_NATIVE_PROFILE_MANIFEST,
};

use worth_ui_host_contract::{UiMountedTextSchemaVersion, WorthUiHostMechanicsAdapter};

#[cfg(feature = "certification-support")]
use crate::native::STAGED_APPEARANCE_MECHANICS;

#[cfg(feature = "certification-support")]
use worth_ui_host_contract::{UiHostAppearanceProfilePosture, UiHostPrimaryPointerKind};

#[test]
fn staged_v2_profile_is_exactly_non_current_and_carries_the_native_long_pole() {
    let staged = parse(WORTH_UI_NATIVE_NEXT_PROFILE_MANIFEST);
    let live = parse(WORTH_UI_NATIVE_PROFILE_MANIFEST);
    assert_eq!(
        live["identity"].as_str(),
        Some(UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V1.as_str())
    );
    assert_eq!(
        staged["identity"].as_str(),
        Some(STAGED_APPEARANCE_PROFILE.identity)
    );
    assert_ne!(staged["identity"], live["identity"]);
    assert_eq!(
        staged["profile_stage"].as_str(),
        Some("qualification-only-non-current")
    );
    assert_eq!(staged["live_emission"].as_str(), Some("disabled"));
    assert_eq!(
        staged["appearance_surface_pipeline"].as_str(),
        Some("rounded-fill-inward-border")
    );
    assert_eq!(
        staged["appearance_outline_pipeline"].as_str(),
        Some("outside-ring-full-fringe")
    );
    assert_eq!(
        staged["appearance_antialiasing"].as_str(),
        Some("analytic-signed-distance-pixel-center")
    );
    assert_eq!(
        staged["appearance_anti_alias_fringe_physical_pixels"].as_integer(),
        Some(1)
    );
    assert_eq!(
        staged["appearance_qualified_scales"].as_str(),
        Some("1.0;1.25;1.5;2.0")
    );
}

#[test]
fn staged_profile_capacity_and_scale_constants_match_the_checked_manifest() {
    let staged = parse(WORTH_UI_NATIVE_NEXT_PROFILE_MANIFEST);
    assert_eq!(
        staged["retained_commands"].as_integer(),
        Some(i64::from(STAGED_APPEARANCE_PROFILE.retained_commands))
    );
    assert_eq!(
        staged["surface_commands"].as_integer(),
        Some(i64::from(STAGED_APPEARANCE_PROFILE.surface_commands))
    );
    assert_eq!(
        staged["outline_commands"].as_integer(),
        Some(i64::from(STAGED_APPEARANCE_PROFILE.outline_commands))
    );
    assert_eq!(
        staged["backdrop_commands"].as_integer(),
        Some(i64::from(STAGED_APPEARANCE_PROFILE.backdrop_commands))
    );
    assert_eq!(
        staged["overlay_order_commands"].as_integer(),
        Some(i64::from(STAGED_APPEARANCE_PROFILE.overlay_order_commands))
    );
    assert_eq!(
        staged["pointer_affordance_commands"].as_integer(),
        Some(i64::from(
            STAGED_APPEARANCE_PROFILE.pointer_affordance_commands
        ))
    );
    assert_eq!(
        staged["text_commands"].as_integer(),
        Some(i64::from(
            STAGED_APPEARANCE_PROFILE.text_foreground_commands
        ))
    );
    assert_eq!(
        staged["damage_regions"].as_integer(),
        Some(i64::from(STAGED_APPEARANCE_PROFILE.damage_regions))
    );
    assert_eq!(
        STAGED_APPEARANCE_PROFILE.scales_milli,
        &[1_000, 1_250, 1_500, 2_000]
    );
    assert_eq!(
        STAGED_APPEARANCE_PROFILE.anti_alias_fringe_physical_pixels,
        1
    );
}

#[cfg(feature = "certification-support")]
#[test]
fn certification_report_uses_the_explicit_host_owned_staged_mechanic_qualification() {
    let report = crate::staged_appearance_capability_report();
    let profile = report
        .appearance_profile()
        .expect("certification report must carry the staged appearance profile");

    assert_eq!(
        profile.posture(),
        UiHostAppearanceProfilePosture::StagedNonCurrent
    );
    assert_eq!(profile.identity(), STAGED_APPEARANCE_PROFILE.identity);
    assert_eq!(profile.version(), STAGED_APPEARANCE_PROFILE.version);
    assert_eq!(
        profile.mechanics(),
        &STAGED_APPEARANCE_MECHANICS,
        "native qualification must enumerate each supported mechanic explicitly"
    );
    assert_eq!(
        profile.primary_pointer(),
        Some(UiHostPrimaryPointerKind::Mouse)
    );
}

#[test]
fn live_native_preparation_remains_v1_without_staged_appearance_or_cutover_protocol() {
    let (mechanics, _event_loop) = crate::WorthUiPreparedNativeHost::prepare_qualified().into_parts(
        crate::UiNativeWindowConfiguration::qualified("host-admission", [800, 600]),
    );
    let report = WorthUiHostMechanicsAdapter::mechanical_capability_report(&mechanics);
    let protocol = WorthUiHostMechanicsAdapter::mechanical_protocol_contract(&mechanics);

    assert!(report.appearance_profile().is_none());
    assert_eq!(
        WORTH_UI_NATIVE_PROFILE_MANIFEST
            .parse::<toml::Value>()
            .expect("live native profile manifest parses")["identity"]
            .as_str(),
        Some(UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V1.as_str())
    );
    assert_eq!(protocol.protocol().revision(), 6);
    assert_eq!(protocol.mounted_frame().revision(), 5);
    assert_eq!(protocol.mounted_presentation().revision(), 5);
    assert_eq!(protocol.observation().revision(), 7);
    assert_eq!(protocol.measurement().revision(), 5);
    assert_eq!(protocol.solicited_effect().revision(), 1);
    assert_eq!(UiMountedTextSchemaVersion::current().revision(), 3);
}

fn parse(manifest: &str) -> toml::Value {
    manifest.parse().expect("profile manifest parses")
}
