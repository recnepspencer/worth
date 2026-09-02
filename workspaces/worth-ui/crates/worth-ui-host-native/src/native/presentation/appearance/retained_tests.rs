use super::command::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandFamily, UiNativeAppearanceCommandKey,
};
use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::UiNativeAppearanceScale;
use super::mounted_mechanic_fixtures::{
    allocation, backdrop_for_surface, filled_surface, pointer, text_foreground,
    FilledSurfaceFixtureInput,
};
use super::retained::{UiNativeAppearanceRetained, UiNativeAppearanceRetainedDenial};
use worth_ui_host_contract::{
    UiMountedAppearanceColor, UiMountedOverlayOrderMechanic, UiMountedPresentationAttemptIdentity,
    UiOverlayParticipantIdentity, UiPointerAffordanceFamily, UiSemanticSurfaceIdentity,
};

#[test]
fn cursor_and_text_rows_are_retained_as_distinct_non_surface_families() {
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let mut retained = UiNativeAppearanceRetained::new(scale);
    let text_key = retained
        .insert(
            UiNativeAppearanceCommand::TextForeground(text_foreground(7)),
            None,
        )
        .unwrap();
    let pointer_key = retained
        .insert(
            UiNativeAppearanceCommand::PointerAffordance(pointer(
                UiPointerAffordanceFamily::Activation,
            )),
            Some(text_key),
        )
        .unwrap();
    assert_eq!(retained.ordered_keys().as_ref(), &[text_key, pointer_key]);
    assert!(retained
        .command(pointer_key)
        .unwrap()
        .damage_rect(scale)
        .unwrap()
        .is_none());
}

#[test]
fn retained_replace_remove_and_order_edit_preserve_exact_damage_events() {
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let mechanic = filled_surface(FilledSurfaceFixtureInput {
        allocation: allocation(0, 0, 10_000, 10_000),
        color: UiMountedAppearanceColor::from_straight_srgba([10, 20, 30, 255]),
    });
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

#[test]
fn retained_commands_preserve_backdrop_and_overlay_order_without_flattening() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = backdrop_for_surface(surface, 0);
    let second = backdrop_for_surface(surface, 1);
    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        surface,
        UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
        1,
        1,
        [
            UiOverlayParticipantIdentity::Backdrop(first.identity().clone()),
            UiOverlayParticipantIdentity::Backdrop(second.identity().clone()),
        ],
    )
    .unwrap();
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let first_key = {
        let mut retained = UiNativeAppearanceRetained::new(scale);
        let first_key = retained
            .insert(UiNativeAppearanceCommand::Backdrop(first.clone()), None)
            .unwrap();
        let order_key = retained
            .insert(
                UiNativeAppearanceCommand::OverlayOrder(order),
                Some(first_key),
            )
            .unwrap();
        let second_key = retained
            .insert(
                UiNativeAppearanceCommand::Backdrop(second.clone()),
                Some(order_key),
            )
            .unwrap();
        assert_eq!(
            retained.ordered_keys().as_ref(),
            &[first_key, order_key, second_key]
        );
        assert_eq!(
            retained.command(order_key).unwrap().family(),
            UiNativeAppearanceCommandFamily::OverlayOrder
        );
        let replay = retained
            .replay_for_damage(UiNativeAppearanceDamageRect {
                left: 0,
                top: 0,
                right: 40,
                bottom: 40,
            })
            .unwrap();
        assert_eq!(replay.as_ref(), &[first_key, second_key]);
        let counters = retained.counters();
        assert_eq!(counters.full_scan_commands, 0);
        assert!(counters.damage_queries > 0);
        second_key
    };
    assert!(first_key.value() > 0);
}

#[test]
fn retained_spatial_replay_uses_the_bounded_index_not_a_draw_list_scan() {
    let scale = UiNativeAppearanceScale::qualified(1_000).unwrap();
    let mut retained = UiNativeAppearanceRetained::new(scale);
    let mut previous = None;
    for index in 0..64 {
        let key = retained
            .insert(
                UiNativeAppearanceCommand::Surface(filled_surface(FilledSurfaceFixtureInput {
                    allocation: allocation(index * 10_000, 0, 4_000, 4_000),
                    color: UiMountedAppearanceColor::from_straight_srgba([1, 2, 3, 255]),
                })),
                previous,
            )
            .unwrap();
        previous = Some(key);
    }
    let replay = retained
        .replay_for_damage(UiNativeAppearanceDamageRect {
            left: 0,
            top: 0,
            right: 4,
            bottom: 4,
        })
        .unwrap();
    assert_eq!(replay.len(), 1);
    let counters = retained.counters();
    assert_eq!(counters.full_scan_commands, 0);
    assert!(counters.damage_leaf_probes < 64);
}
