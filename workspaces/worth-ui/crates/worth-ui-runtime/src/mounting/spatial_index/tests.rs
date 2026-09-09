use super::*;

const GENEROUS: UiMountedSpatialBudget = UiMountedSpatialBudget {
    node_visits: 16_384,
    candidates: 8_192,
};

#[test]
fn spatial_candidates_match_an_independent_scan_through_replacements_and_removals() {
    let mut tree = UiMountedSpatialTree::default();
    let mut rows = Vec::new();
    for i in 0..257 {
        let id = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let x = ((i * 37) % 83) as f64 - 41.0;
        let y = ((i * 19) % 61) as f64 - 30.0;
        let bounds = [x, y, x + (i % 9 + 1) as f64, y + (i % 7 + 1) as f64];
        tree.replace(id, None, Some(bounds)).unwrap();
        rows.push((id, Some(bounds)));
    }
    let predecessor = tree.clone();
    let old_rows = rows.clone();
    for (i, (id, bounds)) in rows.iter_mut().enumerate() {
        if i % 3 == 0 {
            tree.replace(*id, *bounds, None).unwrap();
            *bounds = None;
        } else if i % 3 == 1 {
            let old = bounds.unwrap();
            let next = [old[0] + 0.5, old[1] - 2.0, old[2] + 0.5, old[3] - 2.0];
            tree.replace(*id, *bounds, Some(next)).unwrap();
            *bounds = Some(next);
        }
    }
    for x in (-45..45).step_by(3) {
        for y in (-35..35).step_by(5) {
            assert_scan(&tree, &rows, [f64::from(x), f64::from(y)]);
            assert_scan(&predecessor, &old_rows, [f64::from(x), f64::from(y)]);
        }
    }
    // Exact upper edges remain outside even when another node's envelope overlaps.
    for (_, bounds) in &rows {
        if let Some(bounds) = bounds {
            assert_scan(&tree, &rows, [bounds[2], bounds[3]]);
        }
    }
    assert_balanced(&tree.root);
    assert_balanced(&predecessor.root);
}

#[test]
fn spatial_query_and_update_work_stay_local_in_grid_and_vertical_neighborhoods() {
    for grid in [false, true] {
        for count in [64, 4_096] {
            let mut tree = UiMountedSpatialTree::default();
            let mut rows = Vec::new();
            for i in 0..count {
                let x = if grid { (i / 64) as f64 * 8.0 } else { 0.0 };
                let y = if grid {
                    (i % 64) as f64 * 8.0
                } else {
                    i as f64 * 8.0
                };
                let id = UiMountedInstanceIdentity::mint_unbound().unwrap();
                let bounds = [x, y, x + 2.0, y + 2.0];
                tree.replace(id, None, Some(bounds)).unwrap();
                rows.push((id, bounds));
            }
            let (id, bounds) = rows[count / 2];
            let point = [bounds[0] + 1.0, bounds[1] + 1.0];
            let result = tree.at_point(point, GENEROUS).unwrap();
            assert_eq!(result.instances, vec![id]);
            let height = usize::from(tree::height(&tree.root));
            assert!(
                result.work.node_visits <= 2 * height + 1,
                "{grid}/{count}: {:?}",
                result.work
            );
            assert!(result.work.region_tests <= height);
            let abandoned = tree.clone();
            let work = tree.replace(id, Some(bounds), None).unwrap();
            assert!(work.node_visits <= 2 * height);
            assert!(work.node_copies <= 3 * height);
            assert!(tree.at_point(point, GENEROUS).unwrap().instances.is_empty());
            assert_eq!(
                abandoned.at_point(point, GENEROUS).unwrap().instances,
                vec![id]
            );
            let no_op = tree.replace(id, None, None).unwrap();
            assert_eq!(no_op, UiMountedSpatialWork::default());
            assert_balanced(&tree.root);
        }
    }
}

#[test]
fn overlapping_regions_exhaust_explicit_work_budgets_without_partial_candidates() {
    let mut tree = UiMountedSpatialTree::default();
    for _ in 0..32 {
        tree.replace(
            UiMountedInstanceIdentity::mint_unbound().unwrap(),
            None,
            Some([-10.0, -10.0, 10.0, 10.0]),
        )
        .unwrap();
    }
    let denied = tree.at_point(
        [0.0, 0.0],
        UiMountedSpatialBudget {
            node_visits: 3,
            candidates: 32,
        },
    );
    let Err(UiMountedSpatialQueryDenial::NodeBudget { work }) = denied else {
        panic!("node budget must deny");
    };
    assert_eq!(work.node_visits, 3);
    assert_eq!(work.region_tests, 3);
    let denied = tree.at_point(
        [0.0, 0.0],
        UiMountedSpatialBudget {
            node_visits: 32,
            candidates: 2,
        },
    );
    let Err(UiMountedSpatialQueryDenial::CandidateBudget { work }) = denied else {
        panic!("candidate budget must deny");
    };
    assert_eq!(work.node_visits, 3);
    assert_eq!(
        tree.at_point([0.0, 0.0], GENEROUS).unwrap().instances.len(),
        32
    );
    let outside = tree.at_point([100.0, 100.0], GENEROUS).unwrap();
    assert!(outside.instances.is_empty());
    assert_eq!(outside.work.node_visits, 1);
    assert_eq!(outside.work.region_tests, 0);
}

#[test]
fn invalid_successor_bounds_cannot_remove_the_predecessor() {
    let mut tree = UiMountedSpatialTree::default();
    let id = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let bounds = [-1.0, -1.0, 1.0, 1.0];
    tree.replace(id, None, Some(bounds)).unwrap();
    let retained_bytes = tree.retained_node_payload_bytes();
    for invalid in [
        [0.0, 0.0, f64::NAN, 1.0],
        [3.0, 0.0, 2.0, 1.0],
        [0.0, 0.0, f64::INFINITY, 1.0],
    ] {
        assert_eq!(
            tree.replace(id, Some(bounds), Some(invalid)),
            Err(UiMountedSpatialMutationDenial::InvalidBounds)
        );
        assert_eq!(
            tree.at_point([0.0, 0.0], GENEROUS).unwrap().instances,
            vec![id]
        );
        assert_eq!(tree.retained_node_payload_bytes(), retained_bytes);
    }
}

fn assert_scan(
    tree: &UiMountedSpatialTree,
    rows: &[(UiMountedInstanceIdentity, Option<[f64; 4]>)],
    point: [f64; 2],
) {
    let mut expected = rows
        .iter()
        .filter_map(|(id, bounds)| {
            let [x0, y0, x1, y1] = (*bounds)?;
            (x0 <= point[0] && point[0] < x1 && y0 <= point[1] && point[1] < y1).then_some(*id)
        })
        .collect::<Vec<_>>();
    expected.sort_unstable();
    assert_eq!(tree.at_point(point, GENEROUS).unwrap().instances, expected);
}

fn assert_balanced(link: &Link) -> (u16, usize) {
    let Some(node) = link else {
        return (0, 0);
    };
    let (lh, ll) = assert_balanced(&node.left);
    let (rh, rl) = assert_balanced(&node.right);
    assert!(lh.abs_diff(rh) <= 1);
    assert_eq!(node.height, 1 + lh.max(rh));
    assert_eq!(node.len, 1 + ll + rl);
    (node.height, node.len)
}
