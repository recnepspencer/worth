use super::{
    assert_text_damage_transition, fixture, prepare, set_text, text_candidate, text_contract,
};
use worth_ui_host_contract::*;

#[test]
fn allocation_change_routes_foreground_without_semantic_invalidation_and_survives_retry() {
    let role = fixture::foreground_role();
    let contract = text_contract();
    let adopted = contract.scalar_spans()[0].paint_identity();
    let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(contract));
    let (surface, _) = super::super::super::mounting_fixture::mount(&mut session, 1_000);
    set_text(&mut session, 0, "AB");
    fixture::close_source(&mut session, &role, "geometry-routing-initial");
    let initial = prepare(&mut session);
    let old = text_candidate(&initial, adopted, 10);
    fixture::publish(&mut session, &host, initial, 1);

    // The text owner's completed occurrence moves. No owner observation, source
    // close, text edit, or theme change accompanies the admitted allocation
    // change. Dropping preparation must not consume it.
    let revision = session
        .mounted
        .next_occurrence_geometry_revision_for_test(surface)
        .get();
    crate::facade::entry::mounted_occurrence_geometry_test_support::install_shifted_surface_geometry(
        &mut session,
        surface,
        revision,
        &[],
        old.mounted_instance(),
        16.0,
    );
    let mut abandoned = prepare(&mut session);
    assert!(abandoned
        .appearance_invalidation_batch()
        .unwrap()
        .is_physical_input_only());
    let resized = text_candidate(&abandoned, adopted, 10);
    assert_eq!(resized.bounds().x(), old.bounds().x() + 16.0);
    assert_eq!(resized.bounds().width(), old.bounds().width());
    assert_eq!(resized.mounted_instance(), old.mounted_instance());
    assert_eq!(resized.foregrounds(), old.foregrounds());
    assert_text_damage_transition(&abandoned, UiAppearanceTextDamageTransition::Replace);
    let (_, records) = abandoned
        .lower_appearance(
            UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            None,
        )
        .into_parts();
    assert!(!records.is_empty());
    for record in records {
        let crate::runtime::appearance::UiAppearanceInspectionRecord::Projection {
            consumers_selected,
            ..
        } = record
        else {
            panic!("geometry refresh must not deny");
        };
        assert_eq!(
            consumers_selected, 0,
            "retained physical refresh does not select semantic consumers"
        );
    }
    drop(abandoned);

    let retry = prepare(&mut session);
    let retried = text_candidate(&retry, adopted, 10);
    assert_eq!(retried.bounds(), resized.bounds());
    assert_eq!(retried.clip_bounds(), resized.clip_bounds());
    assert_text_damage_transition(&retry, UiAppearanceTextDamageTransition::Replace);
    fixture::publish(&mut session, &host, retry, 2);
    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);
    let _ = session.shutdown();
}

#[test]
fn scalar_edit_changes_qualified_geometry_despite_unchanged_foreground() {
    let role = fixture::foreground_role();
    let contract = text_contract();
    let adopted = contract.scalar_spans()[0].paint_identity();
    let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(contract));
    let _ = super::super::super::mounting_fixture::mount(&mut session, 1_000);
    set_text(&mut session, 0, "AB");
    fixture::close_source(&mut session, &role, "scalar-geometry-initial");
    let initial = prepare(&mut session);
    let before = text_candidate(&initial, adopted, 10);
    fixture::publish(&mut session, &host, initial, 1);
    // Same length and adopted range; the glyph source itself changes.
    set_text(&mut session, 1, "CB");
    let changed = prepare(&mut session);
    let after = super::text_candidate_named(&changed, adopted, 10, "CB");
    assert_eq!(before.foregrounds(), after.foregrounds());
    assert_ne!(
        before.qualified_layout_identity(),
        after.qualified_layout_identity()
    );
    assert_text_damage_transition(&changed, UiAppearanceTextDamageTransition::Replace);
    fixture::publish(&mut session, &host, changed, 2);
    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    let _ = session.shutdown();
}
