use super::*;
use crate::mounting::spatial_index::UiMountedSpatialBudget;
use worth_ui_host_contract::*;

#[test]
fn presented_index_queries_and_updates_are_local_and_preserve_retained_versions() {
    for size in [64, 4096] {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let mut index = UiPresentedHitIndex::default();
        let mut rows = Vec::new();
        for n in 0..size {
            let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
            let row = row(
                issuer,
                binding,
                surface,
                instance,
                n,
                [((n % 64) * 20) as f32, ((n / 64) * 20) as f32, 10.0, 10.0],
            );
            index.replace_base(instance, Some(row));
            rows.push(row);
        }
        let retained = index.clone();
        let found = index.at_point(binding, [5.0, 5.0], budget()).unwrap();
        assert_eq!(
            found
                .rows
                .iter()
                .map(|row| row.mounted_instance())
                .collect::<Vec<_>>(),
            [rows[0].mounted_instance()]
        );
        assert!(found.work.node_visits < 128);
        assert!(found.work.map_key_probes < 64);
        assert!(
            index
                .at_point(binding, [10.0, 5.0], budget())
                .unwrap()
                .rows
                .is_empty(),
            "right edge is excluded"
        );
        assert!(index
            .at_point(
                UiSurfaceBindingGeneration::mint_unbound().unwrap(),
                [5.0, 5.0],
                budget()
            )
            .unwrap()
            .rows
            .is_empty());
        let removed = index.replace_base(rows[0].mounted_instance(), None);
        assert!(removed.node_copies < 64);
        assert!(index
            .at_point(binding, [5.0, 5.0], budget())
            .unwrap()
            .rows
            .is_empty());
        assert_eq!(
            retained
                .at_point(binding, [5.0, 5.0], budget())
                .unwrap()
                .rows
                .len(),
            1
        );
        // Independent scan over the original input rows catches missing regions.
        for point in [[25.0, 5.0], [85.0, 25.0], [1205.0, 1205.0], [-1.0, 0.0]] {
            let mut expected = rows
                .iter()
                .skip(1)
                .filter(|row| {
                    let b = row.bounds();
                    point[0] >= f64::from(b.x())
                        && point[0] < f64::from(b.x() + b.width())
                        && point[1] >= f64::from(b.y())
                        && point[1] < f64::from(b.y() + b.height())
                })
                .map(|row| row.mounted_instance())
                .collect::<Vec<_>>();
            let mut actual = index
                .at_point(binding, point, budget())
                .unwrap()
                .rows
                .iter()
                .map(|row| row.mounted_instance())
                .collect::<Vec<_>>();
            expected.sort_unstable();
            actual.sort_unstable();
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn presented_index_budget_exhaustion_returns_no_partial_candidates() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let mut index = UiPresentedHitIndex::default();
    for n in 0..20 {
        let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        index.replace_base(
            instance,
            Some(row(
                issuer,
                binding,
                surface,
                instance,
                n,
                [0.0, 0.0, 10.0, 10.0],
            )),
        );
    }
    assert!(matches!(
        index.at_point(
            binding,
            [5.0, 5.0],
            UiMountedSpatialBudget {
                node_visits: 64,
                candidates: 3
            }
        ),
        Err(UiPresentedHitQueryDenial::CandidateBudget { work })
            if work.node_visits <= 64 && work.region_tests > 0 && work.map_key_probes > 0
    ));
    assert!(matches!(
        index.at_point(
            binding,
            [5.0, 5.0],
            UiMountedSpatialBudget {
                node_visits: 1,
                candidates: 64
            }
        ),
        Err(UiPresentedHitQueryDenial::NodeBudget { work })
            if work.node_visits == 1 && work.map_key_probes > 0
    ));
    assert_eq!(
        index
            .at_point(binding, [5.0, 5.0], budget())
            .unwrap()
            .rows
            .len(),
        20
    );
}

fn budget() -> UiMountedSpatialBudget {
    UiMountedSpatialBudget {
        node_visits: 1024,
        candidates: 256,
    }
}

pub(super) fn row(
    issuer: UiMountedNodeReceiptIssuer,
    binding: UiSurfaceBindingGeneration,
    surface: UiSemanticSurfaceIdentity,
    instance: UiMountedInstanceIdentity,
    order: u32,
    [x, y, width, height]: [f32; 4],
) -> UiPresentedHitTestRow {
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap();
    let receipt = issuer.receipt_for(instance);
    let row =
        UiMountedHitTestMechanic::complete_from_runtime_mounting(UiMountedHitTestCompletionInput {
            frame: issuer.frame_identity(),
            surface,
            binding,
            mounted_instance: instance,
            node_receipt: receipt,
            bounds,
            clip_bounds: bounds,
            order: UiMountedHitTestOrder::from_runtime_plan(order),
        })
        .unwrap();
    UiPresentedHitTestRow::from_mounted(crate::mounting::UiMountedHitTestPresentation::for_test(
        row,
    ))
}

#[test]
fn presented_index_matches_fractional_and_large_half_open_edges() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    for [x, width] in [[0.1, 0.2], [16_777_216.0, 3.0]] {
        let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let mut index = UiPresentedHitIndex::default();
        index.replace_base(
            instance,
            Some(row(
                issuer,
                binding,
                surface,
                instance,
                1,
                [x, 0.0, width, 4.0],
            )),
        );
        let end = x + width;
        let before = f32::from_bits(end.to_bits() - 1);
        for point in [x, before, end] {
            let expected = point >= x && point < end;
            let found = index
                .at_point(binding, [f64::from(point), 2.0], budget())
                .unwrap();
            assert_eq!(
                !found.rows.is_empty(),
                expected,
                "x={x}, width={width}, point={point}"
            );
        }
    }
}

#[test]
fn presented_index_removing_overlap_winner_exposes_the_next_rank() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let second = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut index = UiPresentedHitIndex::default();
    for (instance, rank) in [(first, 1), (second, 2)] {
        index.replace_base(
            instance,
            Some(row(
                issuer,
                binding,
                surface,
                instance,
                rank,
                [0.0, 0.0, 10.0, 10.0],
            )),
        );
    }
    let winner = |index: &UiPresentedHitIndex| {
        index
            .at_point(binding, [5.0, 5.0], budget())
            .unwrap()
            .rows
            .into_iter()
            .min_by_key(|row| row.order())
            .unwrap()
            .mounted_instance()
    };
    let retained = index.clone();
    assert_eq!(winner(&index), first);
    index.replace_base(first, None);
    assert_eq!(winner(&index), second);
    assert_eq!(winner(&retained), first);
}
