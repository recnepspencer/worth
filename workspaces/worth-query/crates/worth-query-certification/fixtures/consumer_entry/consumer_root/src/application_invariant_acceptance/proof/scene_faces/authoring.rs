//! The existing Scene, rather than a display key, owns new contour context.
use super::*;
use worth_query_topology_entry::{
    CreateSceneTriangle, CreateSceneTriangleDenial, SceneBoxProbe, SceneFaceBoxesRead,
};

#[test]
fn scene_authoring_creates_a_second_contextual_triangle_and_updates_the_actual_source() {
    let world = installation::install(&crate::installed_schema());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    attach(
        &request,
        &world.application,
        "anchor-a",
        "scene",
        "first",
        0x919e01,
    );
    let original = faces(&request, "scene");
    let calls = &world.scene_box_observations;
    let builds = calls.builds.load(Ordering::SeqCst);
    let updates = calls.updates.load(Ordering::SeqCst);
    let triangle = CreateSceneTriangle {
        scene_key: "scene".into(),
        face_key: "second".into(),
        points: [
            [length(21), length(21)],
            [length(30), length(21)],
            [length(21), length(30)],
        ],
    };
    require_commit(
        request
            .advanced()
            .mutate(triangle.clone())
            .idempotency(&0x919e02_u64)
            .execute_in_program(
                &world.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap(),
    );
    let rows = faces(&request, "scene");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], original[0]);
    assert_ne!(rows[0].face, rows[1].face);
    assert_ne!(rows[0].anchor, rows[1].anchor);
    assert_eq!(rows[1].anchor_key, "face:second:vertex:0");
    assert_eq!(rows[1].points, [[21, 21], [30, 21], [21, 30]]);
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    assert!(calls.updates.load(Ordering::SeqCst) > updates);
    let all = SceneBoxProbe {
        lower_x: 0,
        lower_y: 0,
        upper_x: 100,
        upper_y: 100,
    };
    let candidates = request
        .query(SceneFaceBoxesRead {
            scene_key: "scene".into(),
            probe: all,
        })
        .execute()
        .unwrap();
    let mut expected = rows.clone();
    expected.sort_by(|a, b| a.anchor_key.cmp(&b.anchor_key));
    assert_eq!(candidates.rows(), expected);
    for point in [[25, 25], [50, 50]] {
        let probe = SceneBoxProbe {
            lower_x: point[0],
            lower_y: point[1],
            upper_x: point[0],
            upper_y: point[1],
        };
        let observed = request
            .query(SceneFaceBoxesRead {
                scene_key: "scene".into(),
                probe,
            })
            .execute()
            .unwrap();
        let mut expected = rows
            .iter()
            .filter(|row| {
                (0..2).all(|axis| {
                    let min = row.points.iter().map(|p| p[axis]).min().unwrap();
                    let max = row.points.iter().map(|p| p[axis]).max().unwrap();
                    min <= point[axis] && point[axis] <= max
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        expected.sort_by(|a, b| a.anchor_key.cmp(&b.anchor_key));
        assert_eq!(observed.rows(), expected);
    }
    let before = source_version(&request);
    let updates = calls.updates.load(Ordering::SeqCst);
    let denied = request
        .advanced()
        .mutate(triangle)
        .idempotency(&0x919e03_u64)
        .execute_in_program(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            denied,
            WorthQueryApplicationMutationOutcome::DomainDenied(
                CreateSceneTriangleDenial::FaceKeyTaken
            )
        ),
        "{denied:?}"
    );
    assert_eq!(source_version(&request), before);
    assert_eq!(faces(&request, "scene"), rows);
    assert_eq!(calls.updates.load(Ordering::SeqCst), updates);
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
}

#[test]
fn scene_authoring_tracks_each_derived_body_key_before_allocating_a_face() {
    let world = installation::install(&crate::installed_schema());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    attach(
        &request,
        &world.application,
        "anchor-a",
        "scene",
        "first",
        0x919e40,
    );
    // Collide with the middle key: checking only the first derived vertex is insufficient.
    let vertices = [
        ("free-first", 41, 41),
        ("face:collision:vertex:1", 50, 41),
        ("free-last", 41, 50),
    ]
    .map(|(key, x, y)| PlanarVertex {
        body_key: key.into(),
        x: length(x),
        y: length(y),
    })
    .to_vec();
    require_commit(mutate(
        &request,
        &world.application,
        PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: PlanarOperation::CreateCycle(vertices),
        },
        0x919e41,
    ));
    let before = source_version(&request);
    let rows = faces(&request, "scene");
    let updates = world.scene_box_observations.updates.load(Ordering::SeqCst);
    let outcome = request
        .advanced()
        .mutate(CreateSceneTriangle {
            scene_key: "scene".into(),
            face_key: "collision".into(),
            points: [
                [length(21), length(21)],
                [length(30), length(21)],
                [length(21), length(30)],
            ],
        })
        .idempotency(&0x919e42_u64)
        .execute_in_program(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(
                CreateSceneTriangleDenial::BodyKeyTaken
            )
        ),
        "{outcome:?}"
    );
    assert_eq!(source_version(&request), before);
    assert_eq!(faces(&request, "scene"), rows);
    assert_eq!(
        world.scene_box_observations.updates.load(Ordering::SeqCst),
        updates
    );
    let candidates = request
        .query(SceneFaceBoxesRead {
            scene_key: "scene".into(),
            probe: SceneBoxProbe {
                lower_x: 0,
                lower_y: 0,
                upper_x: 100,
                upper_y: 100,
            },
        })
        .execute()
        .unwrap();
    assert_eq!(candidates.rows(), rows);
    super::insertion::require_absent(&request, "face:collision:vertex:0");
    super::insertion::require_absent(&request, "face:collision:vertex:2");
}
