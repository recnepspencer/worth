use super::super::native_profile::{
    UiNativePlatformProfileIdentity, STAGED_APPEARANCE_PROFILE,
    WORTH_UI_NATIVE_NEXT_PROFILE_MANIFEST, WORTH_UI_NATIVE_PROFILE_MANIFEST,
};

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

fn parse(manifest: &str) -> toml::Value {
    manifest.parse().expect("profile manifest parses")
}
