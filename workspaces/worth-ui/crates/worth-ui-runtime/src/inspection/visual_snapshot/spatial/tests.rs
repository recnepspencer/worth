use super::record::UiVisibleOpacity;
use super::{validate_and_index, UiSpatialValidationDenial};

#[path = "tests/cost_bounds.rs"]
mod cost_bounds;
#[path = "tests/world.rs"]
mod world;
use world::*;

#[test]
fn validation_preserves_disjoint_typed_mechanics_and_half_open_edges() {
    let world = SpatialWorld::new();
    let first = paint(&world, bounds(0.0, 0.0, 8.0, 8.0), 1, u8::MAX);
    let second = paint(&world, bounds(16.0, 0.0, 8.0, 8.0), 2, 128);
    let hit = hit_test(&world, bounds(0.0, 0.0, 24.0, 8.0), 7);
    let basis = crate::mounting::UiMountedVisualRegionBasis::new(
        vec![first, second].into_boxed_slice(),
        vec![hit].into_boxed_slice(),
    );
    let observed = [
        observed_hit(hit, 7),
        observed_paint(second, 2),
        observed_paint(first, 1),
    ];

    let indexed = validate_and_index(41, &basis, &observed, transform([32, 16]))
        .expect("shuffled exact rows validate against their typed mechanics");
    let (visible, hit_test, cost) = indexed.into_parts();
    assert_eq!(visible.len(), 2);
    assert_eq!(hit_test.len(), 1);
    assert_eq!(cost.region_records_examined(), 3);
    assert!(cost.retained_structural_bytes() > 0);

    let (first_candidates, first_probes, first_exhausted) =
        visible.point_candidates(point(0, 0), 16).into_parts();
    assert!(!first_exhausted);
    assert_eq!(first_candidates.len(), 1);
    assert_first_visible_record(*first_candidates[0], &world, first);
    assert!(first_probes > 0);

    assert!(visible
        .point_candidates(point(8, 0), 16)
        .into_parts()
        .0
        .is_empty());
    let second_candidates = visible.point_candidates(point(16, 0), 16).into_parts().0;
    assert_eq!(second_candidates.len(), 1);
    assert_eq!(
        second_candidates[0].opacity(),
        UiVisibleOpacity::Unsupported
    );
    let hit_candidates = hit_test.point_candidates(point(23, 7), 16).into_parts().0;
    assert_eq!(hit_candidates.len(), 1);
    assert_eq!(hit_candidates[0].node_receipt(), world.receipt);
    assert_eq!(hit_candidates[0].total_order().rank(), 7);
    assert_eq!(
        hit_candidates[0].source_projection_digest(),
        hit.semantic_digest()
    );
}

#[test]
fn unsupported_text_paint_is_validated_without_becoming_exact_attribution() {
    let world = SpatialWorld::new();
    let base = paint(&world, bounds(0.0, 0.0, 24.0, 8.0), 1, u8::MAX);
    let text_bounds = bounds(8.0, 0.0, 8.0, 8.0);
    let unsupported = crate::mounting::UiMountedUnsupportedPaintBasis::new(
        world.receipt,
        text_bounds,
        text_bounds,
        2,
        71,
    );
    let basis = crate::mounting::UiMountedVisualRegionBasis::new(
        Box::new([base, unsupported]),
        Box::new([]),
    );
    let observed = [
        observed_paint(base, 1),
        observed(
            unsupported.node_receipt(),
            realized_geometry(unsupported.bounds(), unsupported.clip()),
            realized_ordering(
                unsupported.semantic_order(),
                worth_ui_host_contract::UiHostRealizedRegionParticipation::Paint,
            ),
        ),
    ];

    let indexed = validate_and_index(43, &basis, &observed, transform([32, 16]))
        .expect("unsupported text paint remains an exact validated host observation");
    let (visible, _, cost) = indexed.into_parts();
    assert_eq!(visible.len(), 2);
    assert_eq!(visible.supported_len(), 0);
    assert_eq!(cost.region_records_examined(), 2);
    let text_candidates = visible.point_candidates(point(8, 0), 16).into_parts().0;
    assert_eq!(text_candidates.len(), 2);
    assert!(text_candidates
        .iter()
        .any(|record| record.opacity() == UiVisibleOpacity::Unsupported));
}

#[test]
fn validation_rejects_missing_duplicate_cross_participation_and_wrong_order_rows() {
    let world = SpatialWorld::new();
    let paint = paint(&world, bounds(0.0, 0.0, 8.0, 8.0), 3, u8::MAX);
    let hit = hit_test(&world, bounds(0.0, 0.0, 8.0, 8.0), 5);
    let basis = crate::mounting::UiMountedVisualRegionBasis::new(
        vec![paint].into_boxed_slice(),
        vec![hit].into_boxed_slice(),
    );
    let exact_paint = observed_paint(paint, 3);
    let exact_hit = observed_hit(hit, 5);
    let wrong_hit_order = observed_hit(hit, 6);
    let paint_claimed_as_hit = observed(
        paint.node_receipt(),
        realized_geometry(paint.bounds(), paint.clip()),
        realized_ordering(
            3,
            worth_ui_host_contract::UiHostRealizedRegionParticipation::HitTest,
        ),
    );
    let wrong_bounds = bounds(1.0, 0.0, 8.0, 8.0);
    let wrong_geometry = observed(
        paint.node_receipt(),
        realized_geometry(wrong_bounds, wrong_bounds),
        realized_ordering(
            3,
            worth_ui_host_contract::UiHostRealizedRegionParticipation::Paint,
        ),
    );
    let foreign_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let foreign_receipt = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(world.frame)
        .unwrap()
        .receipt_for(foreign_instance);
    let foreign_identity = observed(
        foreign_receipt,
        realized_geometry(paint.bounds(), paint.clip()),
        realized_ordering(
            3,
            worth_ui_host_contract::UiHostRealizedRegionParticipation::Paint,
        ),
    );

    for rows in [
        vec![exact_paint],
        vec![exact_paint, exact_paint],
        vec![exact_paint, wrong_hit_order],
        vec![paint_claimed_as_hit, exact_hit],
        vec![wrong_geometry, exact_hit],
        vec![foreign_identity, exact_hit],
    ] {
        assert_eq!(
            validate_and_index(7, &basis, &rows, transform([16, 16])).err(),
            Some(UiSpatialValidationDenial::ProtocolMismatch)
        );
    }
}

#[test]
fn viewport_clipping_excludes_offscreen_and_right_edge_pixels() {
    let world = SpatialWorld::new();
    let partially_offscreen = paint(&world, bounds(-4.0, 0.0, 8.0, 8.0), 1, u8::MAX);
    let basis = crate::mounting::UiMountedVisualRegionBasis::new(
        vec![partially_offscreen].into_boxed_slice(),
        Box::new([]),
    );
    let indexed = validate_and_index(
        11,
        &basis,
        &[observed_paint(partially_offscreen, 1)],
        transform([16, 16]),
    )
    .expect("a partially offscreen canonical row clips to the client viewport");
    let (visible, _, _) = indexed.into_parts();
    let clipped = visible.point_candidates(point(0, 0), 16).into_parts().0;
    assert_eq!(clipped.len(), 1);
    assert_eq!(
        clipped[0].clip_lineage().canonical(),
        partially_offscreen.clip()
    );
    assert_eq!(
        clipped[0].clip_lineage().realized(),
        observed_paint(partially_offscreen, 1).clip()
    );
    assert_eq!(
        visible
            .point_candidates(point(3, 7), 16)
            .into_parts()
            .0
            .len(),
        1
    );
    assert!(visible
        .point_candidates(point(4, 0), 16)
        .into_parts()
        .0
        .is_empty());
}

#[test]
fn sparse_1024_record_index_uses_bounded_point_probes() {
    let world = SpatialWorld::new();
    let paint = (0..1_024)
        .map(|index| {
            paint(
                &world,
                bounds((index * 2) as f32, 0.0, 1.0, 1.0),
                index,
                u8::MAX,
            )
        })
        .collect::<Vec<_>>();
    let observed = paint
        .iter()
        .map(|row| observed_paint(*row, row.semantic_order()))
        .collect::<Vec<_>>();
    let basis =
        crate::mounting::UiMountedVisualRegionBasis::new(paint.into_boxed_slice(), Box::new([]));
    let indexed = validate_and_index(13, &basis, &observed, transform([2_048, 2]))
        .expect("the admitted sparse maximum validates and indexes");
    let (visible, _, cost) = indexed.into_parts();
    let (candidates, probes, exhausted) =
        visible.point_candidates(point(1_022, 0), 16).into_parts();
    assert_eq!(candidates.len(), 1);
    assert!(!exhausted);
    assert!(probes <= 24, "balanced sparse lookup took {probes} probes");
    assert_eq!(cost.region_records_examined(), 1_024);
}

#[test]
fn overlapping_point_candidates_stop_at_the_explicit_budget() {
    let world = SpatialWorld::new();
    let paint = (0..64)
        .map(|order| paint(&world, bounds(0.0, 0.0, 8.0, 8.0), order, u8::MAX))
        .collect::<Vec<_>>();
    let observed = paint
        .iter()
        .map(|row| observed_paint(*row, row.semantic_order()))
        .collect::<Vec<_>>();
    let basis =
        crate::mounting::UiMountedVisualRegionBasis::new(paint.into_boxed_slice(), Box::new([]));
    let indexed = validate_and_index(17, &basis, &observed, transform([16, 16]))
        .expect("overlapping admitted rows validate and index");
    let (visible, _, _) = indexed.into_parts();

    let (candidates, probes, exhausted) = visible.point_candidates(point(4, 4), 7).into_parts();

    assert_eq!(candidates.len(), 7);
    assert!(exhausted);
    assert!(probes <= 15, "budgeted lookup took {probes} probes");
}

fn assert_first_visible_record(
    record: super::record::UiVisibleRegionRecord,
    world: &SpatialWorld,
    mechanic: crate::mounting::UiMountedUnsupportedPaintBasis,
) {
    assert_eq!(record.node_receipt(), world.receipt);
    assert_eq!(record.layer_order(), 1);
    assert_eq!(record.paint_order(), 1);
    assert_eq!(record.opacity(), UiVisibleOpacity::Unsupported);
    assert_eq!(record.clip_lineage().canonical(), mechanic.clip());
    assert_eq!(
        record.clip_lineage().realized(),
        observed_paint(mechanic, 1).clip()
    );
    assert_eq!(record.source_projection_digest(), mechanic.source_digest());
}
