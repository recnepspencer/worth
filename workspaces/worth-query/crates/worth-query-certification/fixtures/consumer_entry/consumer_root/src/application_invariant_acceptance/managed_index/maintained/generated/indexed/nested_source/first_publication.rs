//! First applied absence publishes the genuine authored prefix and required output together.
use super::{operation::Observation, program};
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
use std::sync::Arc;
use worth_query_host::facade::{
    application_contribution::WorthQueryApplicationOutputDemand, application_entry::*,
    primary_graph::WorthQueryCurrentOutputSelection, runtime::ExecutionAllocationPolicy,
};

#[test]
fn applied_first_publication_absence_copublishes_the_authored_request_and_nested_output() {
    lifecycle::run(journey);
}
fn journey() {
    let prefix = Arc::new(prefix_observation::Calls::default());
    let world = host::install_geometry(
        prefix.clone(),
        Arc::new(rule::Calls::new(prefix)),
        geometry::Configuration::new(Arc::new(provider::Calls::default())),
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
    ]
    .into_iter()
    .enumerate()
    {
        let outcome = current
            .advanced()
            .mutate(edit)
            .idempotency(&(0x919d_0200u64 + offset as u64))
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
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
    let before_note = current
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(before_note.rows().len(), 1);
    assert_eq!(before_note.rows()[0].note, "");
    let before_rows = current
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(before_rows.rows().len(), 1);
    assert_eq!(
        (
            &*before_rows.rows()[0].identity,
            &*before_rows.rows()[0].name,
            before_rows.rows()[0].enabled
        ),
        ("x", "shared", true)
    );
    let before = current
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let observation = Observation::open();
    let mut envelope = current
        .edit_envelope()
        .idempotency(0x919d_0210u64)
        .runtime_stages(&world)
        .ordered_authored()
        .unwrap();
    let stage = envelope
        .push_stage::<program::Root, _>(request::Request(source::operation::RequestItem {
            directory: "A".into(),
            name: "nested-base".into(),
        }))
        .unwrap();
    let outcome = envelope
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let receipt = match outcome {
        WorthQueryEditEnvelopeOutcome::Committed(receipt) => receipt,
        WorthQueryEditEnvelopeOutcome::StageExecutionDenied(cause) => panic!(
            "first publication stage={:?} binding={} cause={:?}",
            cause.identity(),
            cause.binding(),
            cause.cause()
        ),
        WorthQueryEditEnvelopeOutcome::StageAfterProducer(dependency) => {
            panic!("first publication producer barrier: {dependency:?}")
        }
        other => panic!(
            "expected co-publication, outcome={:?}",
            std::mem::discriminant(&other)
        ),
    };
    assert!(matches!(
        observation.take().as_slice(),
        [WorthQueryCurrentOutputSelection::Missing]
    ));
    receipt.result(&stage).unwrap();
    assert_eq!(
        receipt.stage_receipt(&stage).unwrap().commit_reference(),
        receipt.receipt().commit_reference()
    );
    let after = current
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    assert_ne!(after, before);
    assert_eq!(
        &after,
        receipt
            .receipt()
            .committed_product_publication()
            .composite_commit()
    );
    let note = current
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(note.rows().len(), 1);
    assert_eq!(
        (&*note.rows()[0].directory, &*note.rows()[0].note),
        ("A", "nested-base")
    );
    let rows = current
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(rows.rows().len(), 2);
    let output = rows
        .rows()
        .iter()
        .find(|row| row.identity == "managed-selected-edge-nested-selected-edge")
        .unwrap();
    assert_eq!(
        (&*output.name, output.enabled),
        ("produced:nested-selected-edge", true)
    );
    assert_eq!(
        rows.rows().iter().find(|row| row.identity == "x").unwrap(),
        &before_rows.rows()[0]
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
    drop((receipt, observation));
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
