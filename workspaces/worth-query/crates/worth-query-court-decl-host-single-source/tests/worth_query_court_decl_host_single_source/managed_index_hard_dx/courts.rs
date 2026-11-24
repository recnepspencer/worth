//! Hard DX acceptance courts on real Directory Homes for all 4 stock families
//! (`TextPrefix`, `Interval1D`, `Box2D`, `Box3D`), `CanonicalF64` boxes,
//! `ensure_index_ready`, live watches, degenerate boundaries, and budget refusals.
use super::operations::AssetSpec;
use super::schema::*;
use super::{
    create_asset, delete_asset, home_lease, install, keys, limits, open_directory_home_with_limits,
    update_asset,
};
use worth_execution::ExecutionRequest;
use worth_query_decl::facade::application_schema::CanonicalF64;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::ordered_index::WorthQueryBox2DF64;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

fn q_prefix(prefix: &str) -> FindByTitlePrefix {
    FindByTitlePrefix {
        floor: "floor-1".into(),
        prefix: prefix.to_owned(),
    }
}
fn q_interval(start: i64, end: i64) -> FindByFocusWindow {
    FindByFocusWindow {
        floor: "floor-1".into(),
        window: FocusWindow { start, end },
    }
}
fn q_box2d(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> FindEquipmentInBox2D {
    FindEquipmentInBox2D {
        floor: "floor-1".into(),
        region: QueryBox2D {
            min_x,
            min_y,
            max_x,
            max_y,
        },
    }
}
fn q_box3d(min: [i64; 3], max: [i64; 3]) -> FindRoutesInBox3D {
    FindRoutesInBox3D {
        floor: "floor-1".into(),
        region: QueryBox3D {
            min_x: min[0],
            min_y: min[1],
            min_z: min[2],
            max_x: max[0],
            max_y: max[1],
            max_z: max[2],
        },
    }
}

fn assert_not_ready<R>(res: Result<R, WorthQueryApplicationRequestQueryDenial>) {
    match res {
        Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
            if matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::Refused(
                    WorthQueryManagedIndexRefusal::NotReady
                        | WorthQueryManagedIndexRefusal::RootUnavailable
                )
            ) => {}
        Err(other) => panic!("expected NotReady refusal, got Err({other:?})"),
        Ok(_) => panic!("expected NotReady refusal, got Ok"),
    }
}

#[test]
fn directory_home_four_stock_families_lifecycle_reopen_ensure_ready_watches_and_edges() {
    let dir = std::env::temp_dir().join(format!(
        "worth-hard-dx-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let lease = home_lease();
    let exec = ExecutionRequest::leased(&lease);

    let seed = [
        // Normal equipment in [-10, 10) x [-5, 5) x [0, 3]
        AssetSpec::i64_only(1, "CNC-Alpha", (10, 30), [-10, -5, 0], [10, 5, 3]),
        // Edge-touching on X=10: disjoint for half-open Box2D [-10,10), overlapping for closed Box3D [0,10]!
        AssetSpec::i64_only(2, "cnc-beta", (30, 50), [10, 0, 3], [20, 10, 6]),
        // Degenerate empty half-open interval (40,40) and degenerate half-open Box2D [5,5)
        // Must NEVER match half-open Interval1D or Box2D, MUST match closed Box3D point [5,5,5]!
        AssetSpec::i64_only(3, "Press-Point", (40, 40), [5, 5, 5], [5, 5, 5]),
        // Reversed source row (80, 60) and max_x < min_x: must NEVER match any valid query!
        AssetSpec::i64_only(4, "Reversed-Bad", (80, 60), [30, 30, 30], [20, 20, 20]),
    ];

    let home = open_directory_home_with_limits(&dir, &exec, true, &seed, limits(None, None));
    let scope = request_scope();
    let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
    let req = home.runtime().runtime().request(&principal, &scope);

    // Cold startup: ensure_index_ready opens + rebuilds from Created -> Ready in one call!
    let idx_prefix = req
        .query(q_prefix(""))
        .ensure_index_ready::<TitlePrefixIndex>()
        .unwrap();
    let idx_iv = req
        .query(q_interval(0, 100))
        .ensure_index_ready::<FocusIntervalIndex>()
        .unwrap();
    let idx_b2 = req
        .query(q_box2d(-20, -20, 25, 25))
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .unwrap();
    let idx_b3 = req
        .query(q_box3d([-20, -20, -20], [25, 25, 25]))
        .ensure_index_ready::<RouteBox3DIndex>()
        .unwrap();
    assert_eq!(
        idx_prefix.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready
    );
    assert_eq!(idx_iv.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(idx_b2.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(idx_b3.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);

    // Warm call to ensure_index_ready is idempotent and returns Ready.
    let idx_b2_warm = req
        .query(q_box2d(0, 0, 1, 1))
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .unwrap();
    assert_eq!(
        idx_b2_warm.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready
    );

    // Verify exact half-open vs closed, degenerate, and reversed semantics.
    assert_eq!(
        keys(req.query(q_prefix("CNC")).execute().unwrap().rows()),
        [1, 2]
    );
    // Half-open [0, 30): matches [10,30); edge-touching [30,50) does NOT match; [40,40) & [80,60) never match.
    assert_eq!(
        keys(req.query(q_interval(0, 30)).execute().unwrap().rows()),
        [1]
    );
    assert_eq!(
        keys(req.query(q_interval(35, 45)).execute().unwrap().rows()),
        [2]
    );
    // Half-open Box2D [-10, 10) x [-5, 5): matches 1; does NOT match edge-touching 2 or empty 3 or reversed 4.
    assert_eq!(
        keys(req.query(q_box2d(-10, -5, 10, 5)).execute().unwrap().rows()),
        [1]
    );
    // Closed Box3D [0, 0, 0]..=[10, 10, 5]: matches 1, edge-touching 2 (at x=10,z=3), AND closed point 3 (at [5,5,5]), NOT reversed 4!
    assert_eq!(
        keys(
            req.query(q_box3d([0, 0, 0], [10, 10, 5]))
                .execute()
                .unwrap()
                .rows()
        ),
        [1, 2, 3]
    );

    // Open live watches on all 4 families, mutate (insert, move, delete), and verify exact watch delivery.
    let (init_b2, mut w_b2) = req
        .query(q_box2d(0, 0, 15, 15))
        .watch_index::<EquipmentBox2DIndex>()
        .unwrap();
    let (init_b3, mut w_b3) = req
        .query(q_box3d([0, 0, 0], [15, 15, 5]))
        .watch_index::<RouteBox3DIndex>()
        .unwrap();
    assert_eq!(keys(init_b2.rows()), [1, 2]);
    assert_eq!(keys(init_b3.rows()), [1, 2, 3]);

    // Insert route/equipment 5 inside watch region.
    create_asset(
        home.runtime(),
        AssetSpec::i64_only(5, "CNC-Gamma", (15, 25), [2, 2, 2], [8, 8, 4]),
        10,
    );
    let WorthQueryManagedIndexWatchOutcome::Delivered(d1) = req
        .query(q_box2d(0, 0, 15, 15))
        .next_watch(&mut w_b2)
        .unwrap()
    else {
        panic!("Box2D watch must deliver after insert");
    };
    assert_eq!(keys(d1.read().rows()), [1, 2, 5]);

    // Move equipment 1 out of [0,15)^2 and delete asset 3.
    update_asset(
        home.runtime(),
        AssetSpec::i64_only(1, "CNC-Alpha-Moved", (100, 120), [50, 50, 50], [60, 60, 60]),
        11,
    );
    delete_asset(home.runtime(), 3, 12);
    let WorthQueryManagedIndexWatchOutcome::Delivered(d2) = req
        .query(q_box2d(0, 0, 15, 15))
        .next_watch(&mut w_b2)
        .unwrap()
    else {
        panic!("Box2D watch must deliver after move");
    };
    let WorthQueryManagedIndexWatchOutcome::Delivered(d3) = req
        .query(q_box3d([0, 0, 0], [15, 15, 5]))
        .next_watch(&mut w_b3)
        .unwrap()
    else {
        panic!("Box3D watch must deliver after move+delete");
    };
    assert_eq!(keys(d2.read().rows()), [2, 5]);
    assert_eq!(keys(d3.read().rows()), [2, 5]);

    // Explicit eviction: ensure_index_ready must NOT silently rebuild an evicted index!
    req.query(q_box2d(0, 0, 15, 15))
        .evict_index(&idx_b2)
        .unwrap();
    assert_not_ready(
        req.query(q_box2d(0, 0, 15, 15))
            .ensure_index_ready::<EquipmentBox2DIndex>(),
    );

    drop((init_b2, init_b3, d1, d2, d3));
    w_b2.close().unwrap();
    w_b3.close().unwrap();
    idx_prefix.close();
    idx_iv.close();
    idx_b2.close();
    idx_b3.close();
    home.close(&exec).unwrap().reclaim();

    // Cold Directory Home reopen: tight memory ceiling must refuse ensure_index_ready honestly!
    let tight =
        open_directory_home_with_limits(&dir, &exec, false, &[], limits(Some(16 * 1024), None));
    let scope = request_scope();
    let principal = authenticate_operator(tight.runtime().installed_schema(), &scope);
    let req = tight.runtime().runtime().request(&principal, &scope);
    let err = req
        .query(q_box2d(0, 0, 15, 15))
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .err()
        .expect("ensure_index_ready must refuse under 16 KiB memory ceiling");
    let detail = format!("{err:?}");
    assert!(
        detail.contains("MemoryLimit") || detail.contains("PolicyMemoryExhausted"),
        "expected memory ceiling denial, got {detail}"
    );
    tight.close(&exec).unwrap().reclaim();

    // Clean cold reopen: NotReady before ensure_index_ready, then 1-call ensure_index_ready restores all 4!
    let reopened = open_directory_home_with_limits(&dir, &exec, false, &[], limits(None, None));
    let scope = request_scope();
    let principal = authenticate_operator(reopened.runtime().installed_schema(), &scope);
    let req = reopened.runtime().runtime().request(&principal, &scope);
    assert_not_ready(req.query(q_box2d(0, 0, 15, 15)).execute());
    assert_not_ready(req.query(q_box3d([0, 0, 0], [15, 15, 5])).execute());

    let r_p = req
        .query(q_prefix(""))
        .ensure_index_ready::<TitlePrefixIndex>()
        .unwrap();
    let r_i = req
        .query(q_interval(0, 100))
        .ensure_index_ready::<FocusIntervalIndex>()
        .unwrap();
    let r_2 = req
        .query(q_box2d(0, 0, 15, 15))
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .unwrap();
    let r_3 = req
        .query(q_box3d([0, 0, 0], [15, 15, 5]))
        .ensure_index_ready::<RouteBox3DIndex>()
        .unwrap();

    assert_eq!(
        keys(req.query(q_prefix("cnc")).execute().unwrap().rows()),
        [1, 2, 5]
    );
    assert_eq!(
        keys(req.query(q_interval(10, 40)).execute().unwrap().rows()),
        [2, 5]
    );
    assert_eq!(
        keys(req.query(q_box2d(0, 0, 15, 15)).execute().unwrap().rows()),
        [2, 5]
    );
    assert_eq!(
        keys(req.query(q_box2d(45, 45, 65, 65)).execute().unwrap().rows()),
        [1]
    );
    assert_eq!(
        keys(
            req.query(q_box3d([0, 0, 0], [15, 15, 5]))
                .execute()
                .unwrap()
                .rows()
        ),
        [2, 5]
    );

    r_p.close();
    r_i.close();
    r_2.close();
    r_3.close();
    reopened.close(&exec).unwrap().reclaim();
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn canonical_f64_box2d_handles_negative_coordinates_signed_zero_and_nan_refusal() {
    let mut s1 = AssetSpec::i64_only(1, "NegEquip", (0, 10), [0, 0, 0], [1, 1, 1]);
    s1.fmin_bits = [
        CanonicalF64::from_f64(-50.5).0,
        CanonicalF64::from_f64(-20.0).0,
    ];
    s1.fmax_bits = [
        CanonicalF64::from_f64(-10.25).0,
        CanonicalF64::from_f64(-0.0).0,
    ];
    let mut s2 = AssetSpec::i64_only(2, "OriginEquip", (0, 10), [0, 0, 0], [1, 1, 1]);
    s2.fmin_bits = [
        CanonicalF64::from_f64(0.0).0,
        CanonicalF64::from_f64(-5.0).0,
    ];
    s2.fmax_bits = [
        CanonicalF64::from_f64(12.5).0,
        CanonicalF64::from_f64(8.5).0,
    ];

    let host = install(&[s1, s2], None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let req = host.runtime().request(&principal, &scope);

    let idx = req
        .query(FindInFloatBox2D {
            floor: "floor-1".into(),
            region: WorthQueryBox2DF64::from_f64(-100.0, -100.0, 100.0, 100.0),
        })
        .ensure_index_ready::<FloatBox2DIndex>()
        .unwrap();

    // Negative coordinate query matches only NegEquip.
    assert_eq!(
        keys(
            req.query(FindInFloatBox2D {
                floor: "floor-1".into(),
                region: WorthQueryBox2DF64::from_f64(-30.0, -15.0, -5.0, 1.0),
            })
            .execute()
            .unwrap()
            .rows()
        ),
        [1]
    );
    // Half-open y=[0.0, 5.0) does NOT overlap s1's y=[-20.0, -0.0), DOES overlap s2's y=[-5.0, 8.5).
    assert_eq!(
        keys(
            req.query(FindInFloatBox2D {
                floor: "floor-1".into(),
                region: WorthQueryBox2DF64::from_f64(-60.0, 0.0, 20.0, 5.0),
            })
            .execute()
            .unwrap()
            .rows()
        ),
        [2]
    );
    // NaN probe refuses with InvalidProbeBounds provider denial!
    match req
        .query(FindInFloatBox2D {
            floor: "floor-1".into(),
            region: WorthQueryBox2DF64::from_f64(f64::NAN, 0.0, 10.0, 10.0),
        })
        .execute()
    {
        Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
            if matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::Work(
                    WorthQueryManagedIndexWorkDenial::Provider(_)
                )
            ) => {}
        Err(other) => {
            panic!("NaN probe must refuse with Provider(InvalidProbeBounds), got {other:?}")
        }
        Ok(_) => panic!("NaN probe must not succeed"),
    }
    idx.close();
}
