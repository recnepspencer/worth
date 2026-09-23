use super::*;
use crate::native::presentation::glyph_observation::observe;
use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedNodeReceiptIssuer, UiMountedPresentationUnchanged,
    UiMountedPresentationUnchangedInput,
};

#[test]
fn base_glyph_observations_reuse_samples_and_invalidate_exact_committed_bases() {
    let world = DrawListWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let (command, run, key) = semantic_text(&world, frame);
    let identity = command.identity();
    let order = [UiMountedPaintOrderIdentity::for_command(identity)];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &[command],
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[run],
    )
    .unwrap();
    let atlas = populated_atlas(key);
    let extent = [96, 64];
    let original = observe(&retained, &atlas, extent);
    assert_eq!(original.alpha.len(), 1);
    assert!(original.intrinsic.is_empty());
    assert_eq!(retained.glyph_observation.build_count(), 1);
    let sample = sample_with_bounds(
        &world,
        frame,
        identity,
        viewport_box(10.0, 10.0, 40.0, 20.0),
        viewport_box(22.0, 16.0, 40.0, 20.0),
        32_768,
    );
    let basis = UiNativeRasterBasis::new(extent, 1.0);
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let (mut replay, mut undo) = retained.stage_sample(&sample).unwrap();
    retained
        .refresh_physical_sample(&sample, &mut undo, basis, &mut replay)
        .unwrap();
    let plan = build_plan(basis, &mut retained, replay, 0, &atlas).unwrap();
    assert!(plan
        .operations
        .iter()
        .any(|op| matches!(op, UiNativeRasterOperation::Glyph(_))));
    let sampled = observe(&retained, &atlas, extent);
    assert!(Arc::ptr_eq(&original.alpha, &sampled.alpha));
    retained.rollback_sample(undo).unwrap();
    assert!(Arc::ptr_eq(
        &original.alpha,
        &observe(&retained, &atlas, extent).alpha
    ));

    // A reservation changes no committed image metadata. It must neither
    // invalidate the diagnostic cache nor confer rendering permission.
    let reserved = atlas.plan_demands(&[], &Default::default()).unwrap();
    assert!(Arc::ptr_eq(
        &original.alpha,
        &observe(&retained, &atlas, extent).alpha
    ));
    drop(reserved);
    assert_eq!(retained.glyph_observation.build_count(), 1);

    let successor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let receipts = UiMountedNodeReceiptIssuer::mint_for(successor).unwrap();
    let unchanged =
        UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
            predecessor: frame,
            successor,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            production_cost: Default::default(),
        })
        .with_successor_receipt_affinity(Some(receipts.receipt_affinity()));
    let undo = retained.stage_unchanged(&unchanged).unwrap();
    retained.rollback_unchanged(undo).unwrap();
    assert!(Arc::ptr_eq(
        &original.alpha,
        &observe(&retained, &atlas, extent).alpha
    ));
    retained.apply_unchanged(&unchanged).unwrap();
    let successor_rows = observe(&retained, &atlas, extent);
    assert!(!Arc::ptr_eq(&original.alpha, &successor_rows.alpha));
    assert_eq!(original.alpha.as_ref(), successor_rows.alpha.as_ref());
    assert_eq!(retained.glyph_observation.build_count(), 2);

    let resized = observe(&retained, &atlas, [192, 128]);
    assert!(!Arc::ptr_eq(&successor_rows.alpha, &resized.alpha));
    assert_eq!(retained.glyph_observation.build_count(), 3);
    let plan = atlas.plan_demands(&[], &Default::default()).unwrap();
    assert!(matches!(
        atlas.settle(plan, &[], UiNativeTextAtlasExternalOutcome::Submitted),
        crate::native::text_atlas::UiNativeTextAtlasCommitOutcome::Committed(_)
    ));
    let settled = observe(&retained, &atlas, [192, 128]);
    assert!(!Arc::ptr_eq(&resized.alpha, &settled.alpha));
    assert_eq!(retained.glyph_observation.build_count(), 4);
    let foreign_atlas = populated_atlas(key);
    let foreign = observe(&retained, &foreign_atlas, [192, 128]);
    assert!(!Arc::ptr_eq(&settled.alpha, &foreign.alpha));
    assert_eq!(foreign.alpha.as_ref(), original.alpha.as_ref());
    assert_eq!(retained.glyph_observation.build_count(), 5);
}
