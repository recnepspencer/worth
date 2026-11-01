//! Actual maintained Source custody and deliberately coarse AABB candidates.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestQueryDenial,
    primary_graph::{
        WorthQueryApplicationCommitDenial, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationQueryAdmissionDenialKind,
    },
};
use worth_query_topology_entry::{SceneBoxProbe, SceneFaceBoxesRead};

#[test]
fn scene_box_candidates_follow_native_cold_and_warm_images_and_keep_source_pinned_support() {
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
        0x919d01,
    );
    attach(
        &request,
        &world.application,
        "sibling-a",
        "scene",
        "second",
        0x919d02,
    );
    let calls = &world.scene_box_observations;
    let builds = calls.builds.load(Ordering::SeqCst);
    assert!(
        builds > 0,
        "Native preparation cold-builds the actual Source before a Parent read"
    );
    let all = probe(0, 0, 100, 100);
    let original = request.query(find(all)).execute().unwrap();
    let old = original.rows().to_vec();
    assert_eq!(old, census(&request, all));
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    let outside = probe(9, 9, 9, 9);
    let candidates = request.query(find(outside)).execute().unwrap();
    assert_eq!(candidates.rows(), census(&request, outside));
    assert_eq!(candidates.rows().len(), 1);
    assert_eq!(candidates.rows()[0].anchor_key, "anchor-a");
    // This literal point is inside [1,10]^2 but outside x+y<=11.
    // The API promises enclosure candidates, not exact triangle contact.
    assert!(9 + 9 > 11);

    let same_head = request.retain_current_read().unwrap();
    let probes = calls.probes.load(Ordering::SeqCst);
    let error = request
        .at(&same_head)
        .query(find(all))
        .execute()
        .err()
        .expect("current-only Parent refuses retained basis");
    let WorthQueryApplicationRequestQueryDenial::Admission(error) = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::BasisUnsupported
    );
    assert_eq!(calls.probes.load(Ordering::SeqCst), probes);
    let mut pinned = request
        .at(&same_head)
        .query(SceneFacesRead {
            scene_key: "scene".into(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    pinned.sort_by(|a, b| a.anchor_key.cmp(&b.anchor_key));
    assert_eq!(pinned, old);

    let updates = calls.updates.load(Ordering::SeqCst);
    require_commit(mutate(
        &request,
        &world.application,
        adjust("anchor-c", 30),
        0x919d03,
    ));
    assert!(calls.updates.load(Ordering::SeqCst) > updates);
    for p in [all, probe(1, 25, 1, 25), probe(21, 21, 30, 30)] {
        assert_eq!(
            request.query(find(p)).execute().unwrap().rows(),
            census(&request, p)
        );
    }
    let source = output_correspondence::observed_source(&request, "anchor-a");
    let replaced = request
        .advanced()
        .mutate(VertexReplacement {
            scope_key: "anchor-a".into(),
            replacement: PlanarVertexReplacement {
                retired_key: "anchor-b".into(),
                next_key: "anchor-c".into(),
                replacement: PlanarVertex {
                    body_key: "box-replacement-b".into(),
                    x: length(11),
                    y: length(1),
                },
            },
        })
        .expect_source(source)
        .idempotency(&0x919d04_u64)
        .execute_in_program(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    require_commit(replaced);
    let new_extent = probe(11, 1, 11, 1);
    let replacement_candidates = request.query(find(new_extent)).execute().unwrap();
    assert_eq!(replacement_candidates.rows(), census(&request, new_extent));
    assert_eq!(replacement_candidates.rows().len(), 1);
    assert_eq!(
        request.query(find(all)).execute().unwrap().rows(),
        census(&request, all)
    );
    assert_eq!(original.rows(), old);
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "warm Source changes do not cold-build"
    );
}

#[test]
fn original_cancelled_scene_query_stops_before_provider_probe_callback() {
    use worth_query_host::facade::{
        admission::authenticated_principal::*, application_contribution::*,
    };
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
        0x919d10,
    );
    let before = request.retain_current_read().unwrap();
    let probes = world.scene_box_observations.probes.load(Ordering::SeqCst);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    cancellation.cancel();
    let error = world
        .application
        .request(&principal, &stopped)
        .query(find(probe(0, 0, 100, 100)))
        .execute()
        .err()
        .expect("original cancelled query refuses");
    assert!(
        matches!(
            error,
            WorthQueryApplicationRequestQueryDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        ),
        "{error:?}"
    );
    assert_eq!(
        world.scene_box_observations.probes.load(Ordering::SeqCst),
        probes
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    assert_eq!(
        request
            .query(find(probe(0, 0, 100, 100)))
            .execute()
            .unwrap()
            .rows(),
        census(&request, probe(0, 0, 100, 100))
    );
}

fn probe(lower_x: u64, lower_y: u64, upper_x: u64, upper_y: u64) -> SceneBoxProbe {
    SceneBoxProbe {
        lower_x,
        lower_y,
        upper_x,
        upper_y,
    }
}
fn find(probe: SceneBoxProbe) -> SceneFaceBoxesRead {
    SceneFaceBoxesRead {
        scene_key: "scene".into(),
        probe,
    }
}
fn census(request: &Request<'_>, probe: SceneBoxProbe) -> Vec<SceneFaceRow> {
    let mut rows = faces(request, "scene");
    rows.retain(|row| {
        (0..2).all(|axis| {
            let lower = row.points.iter().map(|p| p[axis]).min().unwrap();
            let upper = row.points.iter().map(|p| p[axis]).max().unwrap();
            let (query_lower, query_upper) = if axis == 0 {
                (probe.lower_x, probe.upper_x)
            } else {
                (probe.lower_y, probe.upper_y)
            };
            lower <= query_upper && query_lower <= upper
        })
    });
    rows.sort_by(|a, b| a.anchor_key.cmp(&b.anchor_key));
    rows
}
pub(super) fn require_projection_cardinality(denial: &WorthQueryApplicationCommitDenial) {
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let native = denial
        .invariant_execution_failure()
        .unwrap()
        .managed_source_denial()
        .unwrap();
    let mut cause: &(dyn std::error::Error + 'static) = native.provider_failure().unwrap().cause();
    loop {
        if let Some(projection) = cause.downcast_ref::<WorthQueryApplicationProjectionDenial>() {
            assert_eq!(
                projection.kind(),
                WorthQueryApplicationProjectionDenialKind::RelationCardinalityMismatch
            );
            break;
        }
        cause = cause
            .source()
            .expect("original typed Source projection failure remains owned");
    }
}
