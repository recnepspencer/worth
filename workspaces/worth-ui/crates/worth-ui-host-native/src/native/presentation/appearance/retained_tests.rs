use super::command::{UiNativeAppearanceCommand, UiNativeAppearanceCommandKey};
use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::UiNativeAppearanceScale;
use super::retained::{UiNativeAppearanceRetained, UiNativeAppearanceRetainedDenial};
use super::tests::{bounds, length, surface};
use worth_ui_host_contract::{UiMountedAppearanceColor, UiMountedSurfacePaint};

#[test]
fn retained_replace_remove_and_order_edit_preserve_exact_damage_events() {
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let mechanic = surface(
        0,
        0,
        10_000,
        10_000,
        0,
        UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
            10, 20, 30, 255,
        ])),
    );
    let mut retained = UiNativeAppearanceRetained::new(scale);
    let key = retained
        .insert(UiNativeAppearanceCommand::Surface(mechanic.clone()), None)
        .unwrap();
    assert_eq!(
        retained.take_damage().as_ref(),
        &[UiNativeAppearanceDamageRect {
            left: 0,
            top: 0,
            right: 10,
            bottom: 10,
        }]
    );
    retained
        .replace(UiNativeAppearanceCommand::Surface(mechanic))
        .unwrap();
    assert_eq!(retained.take_damage().len(), 1);
    retained.place_after(key, None).unwrap();
    retained.remove(key).unwrap();
    assert_eq!(retained.take_damage().len(), 1);
    assert_eq!(retained.counters().full_scan_commands, 0);
}

#[test]
fn staged_surface_fixture_keeps_logical_bounds_in_the_contract_unit() {
    let allocation = bounds(0, 0, 2_000, 3_000);
    assert_eq!(allocation.width(), 2_000);
    assert_eq!(length(1_000).subpixels(), 1_000);
}

#[test]
fn retained_replay_and_order_edits_reject_unusable_keys_and_empty_damage() {
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let mut retained = UiNativeAppearanceRetained::new(scale);
    let missing = UiNativeAppearanceCommandKey::new(99);
    assert_eq!(
        retained.place_after(missing, None),
        Err(UiNativeAppearanceRetainedDenial::MissingIdentity)
    );
    assert_eq!(retained.ordered_keys().len(), 0);
    assert_eq!(
        retained.replay_for_damage(UiNativeAppearanceDamageRect {
            left: 1,
            top: 1,
            right: 1,
            bottom: 2,
        }),
        Err(UiNativeAppearanceRetainedDenial::EmptyDamage)
    );
}
