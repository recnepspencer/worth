//! Real model, Native eligibility and complete structural Source receivers.
use super::*;
mod bounds;
mod box_candidates;
mod length_codec;
mod structural;
use worth_query_consumer_values::{PlanarVertex, PlanarVertexReplacement};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_topology_entry::{
    AttachPlanarFace, PlanarDiscoveryRead, SceneFaceRow, SceneFacesRead, VertexReplacement,
};

#[test]
fn scene_faces_preserve_actual_identity_and_indirect_coordinate_and_rewire_reads() {
    let world = installation::install(&crate::installed_schema());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let old_discovery = request
        .query(PlanarDiscoveryRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    attach(
        &request,
        &world.application,
        "anchor-a",
        "scene",
        "first",
        0x919b01,
    );
    attach(
        &request,
        &world.application,
        "sibling-a",
        "scene",
        "second",
        0x919b02,
    );
    let original = faces(&request, "scene");
    assert_eq!(original.len(), 2);
    assert_ne!(original[0].face, original[1].face);
    assert_ne!(original[0].anchor, original[1].anchor);
    assert_eq!(original[0].points, [[1, 1], [10, 1], [1, 10]]);
    assert_eq!(original[1].points, [[21, 21], [30, 21], [21, 30]]);
    require_commit(mutate(
        &request,
        &world.application,
        adjust("anchor-b", 2),
        0x919b03,
    ));
    let moved = faces(&request, "scene");
    assert_eq!(moved[0].face, original[0].face);
    assert_eq!(moved[0].anchor, original[0].anchor);
    assert_eq!(moved[0].points, [[1, 1], [10, 2], [1, 10]]);
    assert_eq!(moved[1], original[1]);
    let source = output_correspondence::observed_source(&request, "anchor-a");
    let replaced = request
        .advanced()
        .mutate(VertexReplacement {
            scope_key: "anchor-a".into(),
            replacement: PlanarVertexReplacement {
                retired_key: "anchor-b".into(),
                next_key: "anchor-c".into(),
                replacement: PlanarVertex {
                    body_key: "face-replacement-b".into(),
                    x: length(11),
                    y: length(2),
                },
            },
        })
        .expect_source(source)
        .idempotency(&0x919b04_u64)
        .execute_in_program(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    require_commit(replaced);
    let rewired = faces(&request, "scene");
    assert_eq!(rewired[0].face, original[0].face);
    assert_eq!(rewired[0].anchor, original[0].anchor);
    assert_eq!(rewired[0].points, [[1, 1], [11, 2], [1, 10]]);
    assert_eq!(rewired[1], original[1]);
    let before = source_version(&request);
    let updates = world.scene_box_observations.updates.load(Ordering::SeqCst);
    let builds = world.scene_box_observations.builds.load(Ordering::SeqCst);
    let duplicate = request
        .advanced()
        .mutate(AttachPlanarFace {
            scope_key: "anchor-a".into(),
            scene_key: "scene".into(),
            face_key: "duplicate-anchor".into(),
        })
        .idempotency(&0x919b05_u64)
        .execute_in_program(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = duplicate
    else {
        panic!("{duplicate:?}")
    };
    box_candidates::require_projection_cardinality(&denial);
    assert_eq!(
        world.scene_box_observations.updates.load(Ordering::SeqCst),
        updates
    );
    assert_eq!(
        world.scene_box_observations.builds.load(Ordering::SeqCst),
        builds
    );
    assert!(denial.custom_invariant_denial().is_none());
    assert_eq!(source_version(&request), before);
    assert_eq!(faces(&request, "scene"), rewired);
    assert_eq!(
        request
            .query(PlanarDiscoveryRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows(),
        old_discovery
    );
}

#[test]
fn genuine_convex_four_cycle_is_refused_only_by_triangle_eligibility_and_keeps_the_head() {
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
        0x919b10,
    );
    let original = faces(&request, "scene");
    let vertices = [[41, 1], [50, 1], [50, 10], [41, 10]]
        .into_iter()
        .enumerate()
        .map(|(i, [x, y])| PlanarVertex {
            body_key: format!("quad-{i}"),
            x: length(x),
            y: length(y),
        })
        .collect();
    require_commit(mutate(
        &request,
        &world.application,
        PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: PlanarOperation::CreateCycle(vertices),
        },
        0x919b11,
    ));
    let before = source_version(&request);
    let outcome = request
        .advanced()
        .mutate(AttachPlanarFace {
            scope_key: "quad-0".into(),
            scene_key: "quad-scene".into(),
            face_key: "invalid-quad".into(),
        })
        .idempotency(&0x919b12_u64)
        .execute_in_program(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let Some(WorthQueryCustomInvariantDenial::Violation { identity, .. }) =
        denial.custom_invariant_denial()
    else {
        panic!("{denial:?}")
    };
    assert_eq!(identity.rule_id.as_str(), "TriangleFaceContour");
    assert_eq!(
        (
            identity.semantic_version.major,
            identity.semantic_version.minor
        ),
        (1, 0)
    );
    assert_eq!(source_version(&request), before);
    assert_eq!(faces(&request, "scene"), original);
}

fn faces(request: &Request<'_>, scene: &str) -> Vec<SceneFaceRow> {
    let mut rows = request
        .query(SceneFacesRead {
            scene_key: scene.into(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    rows.sort_by(|a, b| a.face_key.cmp(&b.face_key));
    rows
}
fn attach(
    request: &Request<'_>,
    application: &ProgramApplication,
    anchor: &str,
    scene: &str,
    face: &str,
    key: u64,
) {
    require_commit(
        request
            .advanced()
            .mutate(AttachPlanarFace {
                scope_key: anchor.into(),
                scene_key: scene.into(),
                face_key: face.into(),
            })
            .idempotency(&key)
            .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    );
}
fn require_commit<D: std::fmt::Debug, R: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
) {
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
}
