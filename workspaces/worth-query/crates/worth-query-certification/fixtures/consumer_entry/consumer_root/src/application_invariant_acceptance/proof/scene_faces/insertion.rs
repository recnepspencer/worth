//! A genuine positive quad is legal until its actual anchor requires a triangle.
use super::*;
use worth_query_consumer_values::PlanarMutationDenial;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestQueryDenial,
    declaration::application_operation::WorthQueryCreateOutput,
    primary_graph::{
        RecordStructuralChange, WorthQueryEntityResolutionDenialKind,
        WorthQueryEntityResolutionRefusalKind,
    },
};
use worth_query_topology_entry::{
    PlanarCreatedOutputs, PlanarOutputs, SceneBoxProbe, SceneFaceBoxesRead,
};

fn insert(previous: &str, added: &str) -> PlanarMutation {
    PlanarMutation {
        scope_key: "anchor-a".into(),
        operation: PlanarOperation::InsertVertexAfter {
            after_key: "anchor-b".into(),
            previous_next_key: previous.into(),
            vertex: PlanarVertex {
                body_key: added.into(),
                x: length(10),
                y: length(10),
            },
        },
    }
}

#[test]
fn scene_body_only_insertion_reaches_triangle_validation_even_when_source_points_do_not_change() {
    // Two independently installed Worlds perform the identical authored edit.
    for anchored in [false, true] {
        let world = installation::install(&crate::installed_schema());
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let original_ids = [("anchor-a", 1), ("anchor-b", 1), ("anchor-c", 10)].map(|(key, y)| {
            output_correspondence::preserved_identity(
                &request,
                &world.application,
                key,
                y,
                0x919e10 + y + if key == "anchor-b" { 100 } else { 0 },
            )
        });
        if anchored {
            attach(
                &request,
                &world.application,
                "anchor-a",
                "scene",
                "first",
                0x919e20,
            );
        }
        let old = anchored.then(|| faces(&request, "scene"));
        let before = source_version(&request);
        let builds = world.scene_box_observations.builds.load(Ordering::SeqCst);
        let outcome = mutate(
            &request,
            &world.application,
            insert("anchor-c", "inserted-d"),
            0x919e21,
        );
        if anchored {
            let WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::Denied(denial),
            ) = outcome
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
            let old = old.unwrap();
            assert_eq!(faces(&request, "scene"), old);
            let rows = request
                .query(SceneFaceBoxesRead {
                    scene_key: "scene".into(),
                    probe: SceneBoxProbe {
                        lower_x: 0,
                        lower_y: 0,
                        upper_x: 20,
                        upper_y: 20,
                    },
                })
                .execute()
                .unwrap();
            assert_eq!(rows.rows(), old);
            assert_eq!(
                world.scene_box_observations.builds.load(Ordering::SeqCst),
                builds
            );
            require_absent(&request, "inserted-d");
        } else {
            let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome
            else {
                panic!("ordinary positive quad must publish: {outcome:?}")
            };
            assert_eq!(result.changed_vertices, 1);
            let created = receipt
                .outputs_of::<PlanarOutputs>()
                .unwrap()
                .member::<PlanarCreatedOutputs<ConsumerSchema>, WorthQueryCreateOutput>(
                    "inserted-d",
                )
                .unwrap()
                .entity_id();
            assert!(!original_ids.contains(&created));
            let patches = receipt
                .committed_changes()
                .entity_changes()
                .collect::<Vec<_>>();
            assert!(patches.contains(&(created, RecordStructuralChange::Created)));
            assert!(
                !patches.iter().any(|(id, _)| original_ids.contains(id)),
                "old coordinate-bearing entities must have no authoritative patch: {patches:?}"
            );
            assert_eq!(read_y(&request, "inserted-d"), 10);
        }
        assert_eq!(
            [
                read_y(&request, "anchor-a"),
                read_y(&request, "anchor-b"),
                read_y(&request, "anchor-c")
            ],
            [1, 1, 10]
        );
    }
}

#[test]
fn scene_insertion_tracks_old_successor_and_rejects_an_existing_new_key_atomically() {
    let world = installation::install(&crate::installed_schema());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let before = source_version(&request);
    for (previous, added, expected, key) in [
        (
            "anchor-a",
            "wrong-old-d",
            PlanarMutationDenial::UnexpectedSuccessor,
            0x919e30,
        ),
        (
            "anchor-c",
            "sibling-a",
            PlanarMutationDenial::VertexKeyTaken,
            0x919e31,
        ),
    ] {
        let outcome = mutate(&request, &world.application, insert(previous, added), key);
        assert!(
            matches!(outcome,WorthQueryApplicationMutationOutcome::DomainDenied(cause) if cause==expected),
            "{outcome:?}"
        );
        assert_eq!(source_version(&request), before);
    }
    require_absent(&request, "wrong-old-d");
}

pub(super) fn require_absent(request: &Request<'_>, key: &str) {
    let error = request
        .query(PlanarRead {
            body_key: key.into(),
        })
        .execute()
        .err()
        .expect("rejected vertex remains absent");
    let WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause) = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryEntityResolutionDenialKind::Refused(
            WorthQueryEntityResolutionRefusalKind::UnknownEntity
        )
    );
}
