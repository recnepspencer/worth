use super::*;

const BUDGET: UiMountedSpatialBudget = UiMountedSpatialBudget {
    node_visits: 16_384,
    candidates: 4_096,
};

#[test]
fn rectangle_candidates_include_crossings_and_exclude_shared_edges() {
    let mut tree = UiMountedSpatialTree::default();
    let horizontal = UiMountedInstanceIdentity::mint_unbound().unwrap();
    tree.replace(horizontal, None, Some([-5.0, -1.0, 5.0, 1.0]))
        .unwrap();
    // No corner of either rectangle lies inside the other rectangle.
    assert_eq!(
        tree.intersecting([-1.0, -5.0, 1.0, 5.0], BUDGET)
            .unwrap()
            .instances,
        [horizontal]
    );
    for bounds in [
        [5.0, -1.0, 7.0, 1.0],
        [-7.0, -1.0, -5.0, 1.0],
        [-5.0, 1.0, 5.0, 2.0],
        [-5.0, -2.0, 5.0, -1.0],
    ] {
        assert!(tree
            .intersecting(bounds, BUDGET)
            .unwrap()
            .instances
            .is_empty());
    }
}

#[test]
fn rectangle_queries_match_scan_after_updates_without_changing_snapshot() {
    let mut tree = UiMountedSpatialTree::default();
    let mut rows = Vec::new();
    for i in 0..257 {
        let id = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let x = f64::from((i * 37) % 83) - 41.0;
        let y = f64::from((i * 19) % 61) - 30.0;
        let bounds = [x, y, x + f64::from(i % 9 + 1), y + f64::from(i % 7 + 1)];
        tree.replace(id, None, Some(bounds)).unwrap();
        rows.push((id, Some(bounds)));
    }
    let predecessor = tree.clone();
    let old_rows = rows.clone();
    for (i, (id, bounds)) in rows.iter_mut().enumerate() {
        let old = bounds.unwrap();
        let next = match i % 3 {
            0 => None,
            1 => Some([old[0] + 0.5, old[1] - 2.0, old[2] + 0.5, old[3] - 2.0]),
            _ => *bounds,
        };
        tree.replace(*id, *bounds, next).unwrap();
        *bounds = next;
    }
    for x in (-45..45).step_by(3) {
        for y in (-35..35).step_by(5) {
            let query = [
                f64::from(x),
                f64::from(y),
                f64::from(x) + 4.5,
                f64::from(y) + 8.0,
            ];
            assert_scan(&tree, &rows, query);
            assert_scan(&predecessor, &old_rows, query);
        }
    }
}

fn assert_scan(
    tree: &UiMountedSpatialTree,
    rows: &[(UiMountedInstanceIdentity, Option<[f64; 4]>)],
    query: [f64; 4],
) {
    let mut expected = rows
        .iter()
        .filter_map(|(id, bounds)| {
            let [x0, y0, x1, y1] = (*bounds)?;
            let width = x1.min(query[2]) - x0.max(query[0]);
            let height = y1.min(query[3]) - y0.max(query[1]);
            (width > 0.0 && height > 0.0).then_some(*id)
        })
        .collect::<Vec<_>>();
    expected.sort_unstable();
    assert_eq!(
        tree.intersecting(query, BUDGET).unwrap().instances,
        expected
    );
}

#[test]
fn local_rectangle_queries_prune_unrelated_neighborhoods() {
    for count in [64, 4_096] {
        let mut tree = UiMountedSpatialTree::default();
        let mut target = None;
        for i in 0..count {
            let id = UiMountedInstanceIdentity::mint_unbound().unwrap();
            let x = f64::from(i / 64) * 8.0;
            let y = f64::from(i % 64) * 8.0;
            let bounds = [x, y, x + 2.0, y + 2.0];
            tree.replace(id, None, Some(bounds)).unwrap();
            if i == count / 2 {
                target = Some((id, bounds));
            }
        }
        let (id, bounds) = target.unwrap();
        let query = tree.intersecting(bounds, BUDGET).unwrap();
        assert_eq!(query.instances, [id]);
        let height = usize::from(tree::height(&tree.root));
        assert!(
            query.work.node_visits <= 2 * height + 1,
            "{count}: {:?}",
            query.work
        );
        assert!(query.work.region_tests <= height);
    }
}

#[test]
fn rectangle_queries_deny_invalid_geometry_and_exhaustion_without_partial_results() {
    let mut tree = UiMountedSpatialTree::default();
    for _ in 0..32 {
        tree.replace(
            UiMountedInstanceIdentity::mint_unbound().unwrap(),
            None,
            Some([-10.0, -10.0, 10.0, 10.0]),
        )
        .unwrap();
    }
    for bounds in [
        [0.0, 0.0, f64::NAN, 1.0],
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, f64::INFINITY],
    ] {
        assert!(matches!(
            tree.intersecting(bounds, BUDGET),
            Err(UiMountedSpatialQueryDenial::InvalidGeometry)
        ));
    }
    let empty = tree
        .intersecting(
            [0.0; 4],
            UiMountedSpatialBudget {
                node_visits: 0,
                candidates: 0,
            },
        )
        .unwrap();
    assert!(empty.instances.is_empty());
    assert_eq!(empty.work, UiMountedSpatialWork::default());
    let bounds = [-1.0, -1.0, 1.0, 1.0];
    assert!(matches!(tree.intersecting(bounds, UiMountedSpatialBudget {
        node_visits: 3, candidates: 32,
    }), Err(UiMountedSpatialQueryDenial::NodeBudget { work }) if work.node_visits == 3));
    assert!(matches!(tree.intersecting(bounds, UiMountedSpatialBudget {
        node_visits: 32, candidates: 2,
    }), Err(UiMountedSpatialQueryDenial::CandidateBudget { work }) if work.node_visits == 3));
    assert_eq!(
        tree.intersecting(bounds, BUDGET).unwrap().instances.len(),
        32
    );
}
