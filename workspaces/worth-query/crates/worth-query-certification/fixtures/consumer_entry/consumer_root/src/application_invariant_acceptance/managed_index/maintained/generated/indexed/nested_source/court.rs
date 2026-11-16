//! A selected Parent retains nonmatching nested Source content until publication.
use super::program;
use crate::application_invariant_acceptance::{
    authentication,
    managed_index::{lifecycle, maintained as catalog, prefix_observation},
};
use catalog::{
    bounds::Coordinates,
    generated::{indexed::request, source},
    geometry, host, query, rule,
};
use geometry::{authoring::Edit, provider};
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::application_contribution::WorthQueryApplicationOutputDemand;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

#[test]
fn later_nonmatching_nested_endpoint_change_invalidates_applied_source_before_publication() {
    lifecycle::run(journey);
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let configuration = geometry::Configuration::new(calls.clone());
    let observations = configuration.rules.clone();
    observations
        .capture_complete_source
        .store(true, Ordering::SeqCst);
    let prefix = Arc::new(prefix_observation::Calls::default());
    let world = host::install_geometry(
        prefix.clone(),
        Arc::new(rule::Calls::new(prefix)),
        configuration,
    );
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let current = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    for (offset, edit) in [
        Edit::Vertex {
            directory: "A".into(),
            identity: "near-start".into(),
            x: 0,
            y: 0,
        },
        Edit::Vertex {
            directory: "A".into(),
            identity: "near-end".into(),
            x: 1,
            y: 0,
        },
        Edit::Edge {
            directory: "A".into(),
            identity: "selected-edge".into(),
            start: "near-start".into(),
            end: "near-end".into(),
        },
        Edit::Vertex {
            directory: "A".into(),
            identity: "far-start".into(),
            x: 50,
            y: 0,
        },
        Edit::Vertex {
            directory: "A".into(),
            identity: "far-end".into(),
            x: 60,
            y: 0,
        },
        Edit::Edge {
            directory: "A".into(),
            identity: "future-edge".into(),
            start: "far-start".into(),
            end: "far-end".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let outcome = current
            .advanced()
            .mutate(edit)
            .idempotency(&(0x919d_0100u64 + offset as u64))
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
    let all_before = current
        .query(geometry::query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(all_before.rows().len(), 2);
    let future = all_before
        .rows()
        .iter()
        .find(|row| row.identity == "future-edge")
        .unwrap();
    assert_eq!(
        (&*future.identity, future.start, future.end),
        (
            "future-edge",
            Coordinates::new(50, 0, 50, 0),
            Coordinates::new(60, 0, 60, 0)
        )
    );
    let selected = current
        .query(
            super::Demand {
                directory: "A".into(),
            }
            .source_intent(),
        )
        .execute()
        .unwrap();
    assert_eq!(selected.rows().len(), 1);
    assert_eq!(
        (
            &*selected.rows()[0].identity,
            selected.rows()[0].start,
            selected.rows()[0].end
        ),
        (
            "selected-edge",
            Coordinates::new(0, 0, 0, 0),
            Coordinates::new(1, 0, 1, 0)
        )
    );
    assert!(
        selected
            .rows()
            .iter()
            .all(|row| row.identity != "future-edge"),
        "independent endpoint coordinates exclude the future entrant from [-1,1]"
    );
    let note_before = current
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let rows_before = current
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(note_before.rows().len(), 1);
    assert_eq!(note_before.rows()[0].note, "");
    assert_eq!(rows_before.rows().len(), 1);
    assert_eq!(
        (
            &*rows_before.rows()[0].identity,
            &*rows_before.rows()[0].name,
            rows_before.rows()[0].enabled
        ),
        ("x", "shared", true)
    );
    let head = current
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    observations.records.lock().unwrap().clear();
    let probes = calls.probe_validations.load(Ordering::SeqCst);
    let output_observation = super::operation::Observation::open();
    let mut envelope = current
        .edit_envelope()
        .idempotency(0x919d_0120u64)
        .runtime_stages(&world)
        .ordered_authored()
        .unwrap();
    envelope
        .push_stage::<program::Root, _>(request::Request(source::operation::RequestItem {
            directory: "A".into(),
            name: "nested-base".into(),
        }))
        .unwrap();
    envelope
        .push_stage::<catalog::generated::program::Root, _>(super::motion::Input {
            directory: "A".into(),
            vertex: "far-start".into(),
            replacement: Coordinates::new(51, 0, 51, 0),
            output: "managed-selected-edge-nested-selected-edge".into(),
            expected_name: "produced:nested-selected-edge".into(),
        })
        .unwrap();
    let outcome = envelope
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let denial = match outcome {
        WorthQueryEditEnvelopeOutcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) => denial,
        WorthQueryEditEnvelopeOutcome::StageExecutionDenied(cause) => panic!(
            "nested endpoint setup stage={:?} binding={} cause={:?}",
            cause.identity(),
            cause.binding(),
            cause.cause()
        ),
        WorthQueryEditEnvelopeOutcome::StageAfterProducer(dependency) => {
            panic!("nested endpoint setup producer barrier: {dependency:?}")
        }
        other => panic!(
            "expected final nested support refusal, outcome={:?}",
            std::mem::discriminant(&other)
        ),
    };
    assert_eq!(
        denial
            .invariant_execution_failure()
            .unwrap_or_else(|| panic!(
                "expected final-support refusal; actual kind={:?} stage={:?} detail={:?}",
                denial.kind(),
                denial.stage(),
                denial.detail()
            ))
            .kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
    );
    assert!(matches!(
        output_observation.take().as_slice(),
        [WorthQueryCurrentOutputSelection::Missing]
    ));
    assert!(
        calls.probe_validations.load(Ordering::SeqCst) > probes,
        "actual managed Source selection ran during the envelope"
    );
    assert!(
        observations
            .records
            .lock()
            .unwrap()
            .iter()
            .filter_map(|record| record.source_after.as_ref())
            .flatten()
            .any(|row| row.identity == "future-edge"
                && row.start == Coordinates::new(51, 0, 51, 0)
                && row.end == Coordinates::new(60, 0, 60, 0)),
        "the real invariant observed the later disjoint endpoint mutation on this candidate"
    );
    assert_eq!(
        current.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        current
            .query(geometry::query::ReadSource {
                directory: "A".into()
            })
            .execute()
            .unwrap()
            .rows(),
        all_before.rows()
    );
    assert_eq!(
        current
            .query(
                super::Demand {
                    directory: "A".into()
                }
                .source_intent()
            )
            .execute()
            .unwrap()
            .rows(),
        selected.rows()
    );
    assert_eq!(
        current
            .query(source::ReadDirectory {
                directory: "A".into()
            })
            .execute()
            .unwrap()
            .rows(),
        note_before.rows()
    );
    let rows_after = current
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(rows_after.rows(), rows_before.rows());
    for identity in [
        "managed-selected-edge-nested-selected-edge",
        "generated-item",
    ] {
        assert!(rows_after.rows().iter().all(|row| row.identity != identity));
    }
    assert!(rows_before
        .rows()
        .iter()
        .all(|row| row.identity != "managed-future-edge-nested-future-edge"));
    drop((denial, output_observation));
    // The same genuine Move is independently admissible after the refused owner
    // releases. Its independently known coordinates change the nonmatching edge.
    let outcome = current
        .advanced()
        .mutate(Edit::Move {
            directory: "A".into(),
            vertex: "far-start".into(),
            x: 51,
            y: 0,
        })
        .idempotency(&0x919d_0121u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let entered = current
        .query(
            super::Demand {
                directory: "A".into(),
            }
            .source_intent(),
        )
        .execute()
        .unwrap();
    assert_eq!(entered.rows(), selected.rows());
    let changed_source = current
        .query(geometry::query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let changed = changed_source
        .rows()
        .iter()
        .find(|row| row.identity == "future-edge")
        .unwrap();
    assert_eq!(
        (&*changed.identity, changed.start, changed.end),
        (
            "future-edge",
            Coordinates::new(51, 0, 51, 0),
            Coordinates::new(60, 0, 60, 0)
        )
    );
    let closed = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .on_branch(world.current_world())
                .close(&phase)
                .unwrap()
        })
        .unwrap();
    assert!(closed.is_complete());
}
