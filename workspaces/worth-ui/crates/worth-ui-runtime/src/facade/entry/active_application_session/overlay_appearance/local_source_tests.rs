use super::*;
use crate::runtime::motion::*;
use crate::runtime::portal::*;
use worth_ui_dsl::*;
use worth_ui_host_contract::*;

#[path = "local_source_test_world.rs"]
mod world;
use world::SourceWorld;

// Exact service-owner and source-adapter evidence; authored publication has
// separate production proofs in declared_overlay_binding_tests.
#[test]
fn immutable_sources_select_local_changes_and_preserve_observed_roots_at_scale() {
    let mut world = SourceWorld::new();
    let mut initial_work = UiActiveBackdropAppearanceWork::default();
    let (accepted, _) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        None,
        &mut initial_work,
    );
    let (foreign, _) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.foreign,
        None,
        &mut Default::default(),
    );
    assert_eq!(
        initial_work.motion_bindings_visited, 1,
        "reverse dependency index excludes the other 511 bindings"
    );
    world.close(0);
    let mut work = UiActiveBackdropAppearanceWork::default();
    let (observed, binding_changes) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        Some(&accepted),
        &mut work,
    );
    let (portals, changes) = localized_portal_source(
        &observed,
        Some(&accepted),
        None,
        &binding_changes,
        &mut work,
    )
    .unwrap();
    let (_, motion_changes) = localized_motion_source(
        &observed,
        Some(&accepted),
        None,
        &binding_changes,
        world.surface.runtime_surface,
        &mut work,
    )
    .unwrap();
    assert_eq!(changes, [world.declarations[0]]);
    assert_eq!(
        motion_changes,
        [world.declarations[0]],
        "binding departure invalidates a Motion dependent even when the track root is unchanged"
    );
    assert_eq!(portals.rows().len(), 511);
    assert_eq!(work.portal_rows_serialized, 511);
    assert_eq!(work.portal_source_rows_visited, 511);
    assert!(
        work.source_comparison_steps < 256,
        "changed paths are bounded separately from 511 issued rows: {work:?}"
    );
    assert_eq!(work.source_changed_keys_visited, 2);
    assert_eq!(work.retained_participants_visited, 0);
    let mut retry_work = UiActiveBackdropAppearanceWork::default();
    let (_, retry_changes) = localized_portal_source(
        &observed,
        Some(&accepted),
        None,
        &binding_changes,
        &mut retry_work,
    )
    .unwrap();
    assert_eq!(
        retry_changes, changes,
        "discarding an observed candidate cannot consume its source delta"
    );

    world.close(1);
    let mut later_work = UiActiveBackdropAppearanceWork::default();
    let (later, bindings) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        Some(&observed),
        &mut later_work,
    );
    let (_, later_changes) =
        localized_portal_source(&later, Some(&observed), None, &bindings, &mut later_work).unwrap();
    assert_eq!(
        later_changes,
        [world.declarations[1]],
        "accepting an earlier observed root leaves a later mutation pending"
    );
    let (_, old_changes) = localized_portal_source(
        &observed,
        Some(&accepted),
        None,
        &binding_changes,
        &mut Default::default(),
    )
    .unwrap();
    assert_eq!(
        old_changes,
        [world.declarations[0]],
        "subsequent source mutations cannot rewrite staged roots"
    );

    let mut foreign_work = UiActiveBackdropAppearanceWork::default();
    let (unchanged, bindings) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.foreign,
        Some(&foreign),
        &mut foreign_work,
    );
    assert!(localized_portal_source(
        &unchanged,
        Some(&foreign),
        None,
        &bindings,
        &mut foreign_work
    )
    .unwrap()
    .1
    .is_empty());
    assert!(localized_motion_source(
        &unchanged,
        Some(&foreign),
        None,
        &bindings,
        world.foreign.runtime_surface,
        &mut foreign_work
    )
    .unwrap()
    .1
    .is_empty());
    assert_eq!(foreign_work.source_changed_keys_visited, 0);
    assert!(foreign_work.source_comparison_steps <= 3);

    // Remove the declared binding while other live source rows remain. Only
    // required bound rows may be visited to serialize this surface.
    world.surface.bindings.replace([]).unwrap();
    let (empty, bindings) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        Some(&later),
        &mut Default::default(),
    );
    let mut empty_work = UiActiveBackdropAppearanceWork::default();
    assert!(
        localized_portal_source(&empty, Some(&later), None, &bindings, &mut empty_work)
            .unwrap()
            .0
            .rows()
            .is_empty()
    );
    assert_eq!(empty_work.portal_source_rows_visited, 0);
    world
        .surface
        .bindings
        .bind(world.declarations[2], world.portal_ids[2])
        .unwrap();
    let (single, bindings) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        Some(&empty),
        &mut Default::default(),
    );
    let mut single_work = UiActiveBackdropAppearanceWork::default();
    let (single_output, changes) =
        localized_portal_source(&single, Some(&empty), None, &bindings, &mut single_work).unwrap();
    assert_eq!(changes, [world.declarations[2]]);
    assert_eq!(single_output.rows().len(), 1);
    assert_eq!(
        single_work.portal_source_rows_visited, 1,
        "509 unbound source rows do not enter the issued-row walk"
    );
}

#[test]
fn motion_source_delta_filters_unrelated_targets_without_advancing_composition() {
    let mut world = SourceWorld::new();
    let current = world.current();
    let (accepted, _) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        None,
        &mut Default::default(),
    );
    world.retarget(1, 2, 3);
    let mut work = UiActiveBackdropAppearanceWork::default();
    let (unrelated, bindings) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        Some(&accepted),
        &mut work,
    );
    let (export, changes) = localized_motion_source(
        &unrelated,
        Some(&accepted),
        Some(&current),
        &bindings,
        world.surface.runtime_surface,
        &mut work,
    )
    .unwrap();
    assert!(changes.is_empty());
    assert_eq!(
        Some(export.unwrap().owner_revision()),
        current.motion_revision()
    );
    assert_eq!(work.motion_bindings_visited, 1);
    assert_eq!(work.motion_targets_looked_up, 1);
    assert_eq!(work.source_changed_keys_visited, 1);
    assert!(work.source_comparison_steps < 128);
    world.retarget(0, 2, 3);
    let mut work = UiActiveBackdropAppearanceWork::default();
    let (changed, bindings) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        Some(&unrelated),
        &mut work,
    );
    let (export, changes) = localized_motion_source(
        &changed,
        Some(&unrelated),
        Some(&current),
        &bindings,
        world.surface.runtime_surface,
        &mut work,
    )
    .unwrap();
    assert_eq!(changes, [world.declarations[0]]);
    let export = export.unwrap();
    assert_eq!(
        export.owner_revision(),
        current.motion_revision().unwrap() + 1
    );
    assert_eq!(export.rows()[0].revision(), 3);
    assert_eq!(work.source_changed_keys_visited, 1);
    assert!(work.source_comparison_steps < 128);
    world.motion.shutdown();
    let (terminal, bindings) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        Some(&changed),
        &mut Default::default(),
    );
    let (export, changes) = localized_motion_source(
        &terminal,
        Some(&changed),
        None,
        &bindings,
        world.surface.runtime_surface,
        &mut Default::default(),
    )
    .unwrap();
    let export = export.unwrap();
    assert_eq!(export.rows()[0].revision(), 3);
    assert!(changes.is_empty());
    world.close(0);
    let (departed, bindings) = capture(
        Some(&world.portals),
        Some(&world.motion),
        &world.surface,
        Some(&terminal),
        &mut Default::default(),
    );
    let (export, changes) = localized_motion_source(
        &departed,
        Some(&terminal),
        None,
        &bindings,
        world.surface.runtime_surface,
        &mut Default::default(),
    )
    .unwrap();
    assert!(export.is_none());
    assert_eq!(changes, [world.declarations[0]]);
}
