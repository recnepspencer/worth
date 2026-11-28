//! Run with `cargo run -p worth-query-topology-entry --example invariant_reads`.
//! The application observer records platform values; only the example's checks
//! compare them. It supplies neither access nor admission policy.
#![recursion_limit = "256"]
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation as installation,
    application_invariants::WorthQueryInvariantProjectionWork, primary_graph,
};
use worth_query_topology_entry::*;

#[path = "invariant_reads/guard_refusals.rs"]
mod guard_refusals;
#[path = "invariant_reads/program.rs"]
mod program;
#[allow(
    dead_code,
    reason = "Reuse the complete reference program, including its output aliases."
)]
#[path = "required_feature/program.rs"]
mod reference_program;
#[allow(
    dead_code,
    reason = "Reuse real installation ports also exercised by the required-feature example."
)]
#[path = "required_feature/installation.rs"]
mod setup;
use program::{ExampleProgram, ExampleSchema, Root};
type Application =
    installation::WorthQueryProgramApplicationRuntime<ExampleSchema, ExampleProgram<false>>;

#[derive(Debug, Eq, PartialEq)]
struct Case {
    preparation: WorthQueryInvariantProjectionWork,
    evaluation: WorthQueryInvariantProjectionWork,
    query: WorthQueryInvariantProjectionWork,
    refused_evaluation: WorthQueryInvariantProjectionWork,
}

fn main() {
    run();
}

fn run() {
    let baseline = observe(0);
    for unrelated_rings in [1, 6] {
        assert_eq!(
            baseline,
            observe(unrelated_rings),
            "unrelated rings add no reads to this edit or query"
        );
    }
}

fn observe(extra_rings: usize) -> Case {
    let events = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&events);
    let mut configuration = setup::configuration();
    let guard_mode = Arc::clone(&configuration.topology.invariant_probe);
    let preparation_calls = Arc::clone(&configuration.topology.invariant_calls);
    configuration.topology.invariant_read_observer = Some(Arc::new(move |event| {
        capture.lock().unwrap().push(event);
    }));
    let mut application = setup::install_rings_configured::<false>(
        configuration,
        setup::build_identity(),
        setup::limits(),
        extra_rings,
        || {},
    )
    .unwrap();
    // Bootstrap also validates the seeded graph. This case observes only its edit.
    events.lock().unwrap().clear();
    let case = {
        let (scope, principal) = setup::authenticate(&application);
        let request = application.request(&principal, &scope);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(source.rows()[0].y, setup::length(1));
        let changed = request
            .advanced()
            .mutate(edit(2))
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&1_u64)
            .execute_performed::<ExampleProgram<false>, Root>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(changed) = changed else {
            panic!("the lawful source edit and its required outputs co-commit");
        };
        let mut delivery = changed
            .start_required_outputs(
                &application,
                &request,
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
        let settled = (0..64)
            .find_map(|_| {
                match delivery
                    .required_output_mut()
                    .advance(&application, &request)
                    .unwrap()
                {
                    WorthQueryApplicationProgramOutputProgress::Pending
                    | WorthQueryApplicationProgramOutputProgress::Progressed => None,
                    WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
                }
            })
            .expect("the finite co-committed tree delivers");
        let expected_touched = affected_bodies(&settled);
        for (key, expected) in [("final:anchor-a", 4), ("final:anchor-b", 3)] {
            let output = request
                .query(PlanarOutputRead {
                    body_key: key.into(),
                })
                .execute()
                .unwrap();
            assert_eq!(output.rows().len(), 1);
            assert_eq!(output.rows()[0].value, setup::length(expected));
        }
        let after = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(after.rows()[0].body_key, "anchor-a");
        assert_eq!(after.rows()[0].y, setup::length(2));
        let query = after.receipt().inspect().read_work().reads();
        let recorded = std::mem::take(&mut *events.lock().unwrap());
        let (preparation, evaluation) = compare_complete_reads(&recorded, &expected_touched, query);
        assert!(
            evaluation.field_reads() > preparation.field_reads(),
            "the real evaluation read its coordinates"
        );
        let refused = request
            .advanced()
            .mutate(edit(12))
            .expect_source(after.observed_sources()[0].clone())
            .idempotency(&2_u64)
            .execute_performed::<ExampleProgram<false>, Root>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(refused) = refused else {
            panic!("the invalid source must not publish");
        };
        require_planar_violation(refused);
        let retained = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            retained.rows()[0].y,
            setup::length(2),
            "the refused coordinates never publish"
        );
        let rejected = std::mem::take(&mut *events.lock().unwrap());
        let (refused_preparation, refused_evaluation) = compare_complete_reads(
            &rejected,
            &expected_touched,
            retained.receipt().inspect().read_work().reads(),
        );
        assert_eq!(preparation, refused_preparation);
        assert!(
            refused_evaluation.field_reads() > refused_preparation.field_reads(),
            "ordinary violation retains acquired reads"
        );
        guard_refusals::verify(
            &application,
            &request,
            guard_refusals::Controls {
                mode: &guard_mode,
                calls: &preparation_calls,
                events: &events,
            },
        );
        Case {
            preparation,
            evaluation,
            query,
            refused_evaluation,
        }
    };
    assert!(application.close_live_delivery().owner_terminal());
    application.close_conditional_runtime().unwrap();
    case
}

// The installed graph preserves its source and creates two three-body final
// rings. Attribute actual observations through issued roles, not numeric IDs.
fn affected_bodies(
    settled: &WorthQueryApplicationProgramOutputSettlement<PlanarQuery>,
) -> Vec<worth_relational::facade::identity::EntityId> {
    let mut bodies = vec![settled
        .root_settlement()
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<ExampleSchema>>()
        .unwrap()
        .entity_id()];
    let main: Vec<_> = settled
        .outputs_for::<ExampleSchema, PlanarOutputToFinalConnection>()
        .collect();
    let alternate: Vec<_> = settled
        .outputs_for::<ExampleSchema, PlanarOutputToAlternateFinalConnection>()
        .collect();
    assert_eq!(
        main.iter()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-a"]
    );
    assert_eq!(
        alternate
            .iter()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-b"]
    );
    assert_eq!(settled.output_count(), main.len() + alternate.len());
    for (_, output) in main.into_iter().chain(alternate) {
        let roles = output.outputs_of::<FinalPlanarOutputs>().unwrap();
        bodies.push(
            roles
                .entity::<FinalAnchorOutput<ExampleSchema>>()
                .unwrap()
                .entity_id(),
        );
        bodies.push(
            roles
                .entity::<FinalClosingOutput<ExampleSchema>>()
                .unwrap()
                .unwrap()
                .entity_id(),
        );
        let members = roles
            .family_entries::<FinalCreatedOutputs<ExampleSchema>>()
            .unwrap();
        assert_eq!(
            members.len(),
            1,
            "one other vertex in each declared final triangle"
        );
        bodies.push(members[0].entity_id());
    }
    bodies.sort();
    bodies
}

// compiled-example: begin
fn compare_complete_reads(
    events: &[PlanarInvariantReadEvent],
    expected_touched: &[worth_relational::facade::identity::EntityId],
    query: WorthQueryInvariantProjectionWork,
) -> (
    WorthQueryInvariantProjectionWork,
    WorthQueryInvariantProjectionWork,
) {
    let mut touched: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::Touched { entity } => Some(*entity),
            _ => None,
        })
        .collect();
    // Sorting retains multiplicity: duplicate or omitted events still fail.
    touched.sort();
    assert_eq!(
        touched, expected_touched,
        "the whole candidate's issued source/output bodies"
    );
    let complete: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::CompleteOrbit {
                before,
                after,
                visited,
                physical_relations,
                ..
            } => Some((*before, *after, *visited, *physical_relations)),
            _ => None,
        })
        .collect();
    let expected_rings = 1 + (expected_touched.len() - 1) / 3;
    assert_eq!(
        complete.len(),
        expected_rings,
        "one Native orbit read per affected ring"
    );
    let identities: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::CompleteOrbit { identity, .. } => Some(*identity),
            _ => None,
        })
        .collect();
    assert_eq!(
        identities
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        expected_rings,
        "every complete orbit is distinct; repeated reads are not hidden"
    );
    for (before, after, visited, physical_relations) in complete {
        assert_eq!(
            (visited, physical_relations),
            (3, 3),
            "the actual complete three-body component"
        );
        let complete_adjacencies = after.adjacency_lists_read() - before.adjacency_lists_read();
        let complete_edges = after.adjacency_edges_inspected() - before.adjacency_edges_inspected();
        assert!(complete_adjacencies >= visited);
        assert!(
            complete_edges >= physical_relations,
            "actual acquisitions include every physical successor"
        );
    }
    assert!(
        query.adjacency_lists_read() > 0,
        "the actual query acquires its relation projection"
    );
    let prepared: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::Prepared {
                reads,
                succeeded: true,
            } => Some(*reads),
            _ => None,
        })
        .collect();
    let evaluated: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::Evaluated {
                reads,
                returned_verdict: true,
            } => Some(*reads),
            _ => None,
        })
        .collect();
    assert_eq!(prepared.len(), 1);
    assert_eq!(evaluated.len(), 1);
    (prepared[0], evaluated[0])
}
// compiled-example: end

fn edit(y: u64) -> PlanarSourceAdjustment {
    PlanarSourceAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: setup::length(y),
    }
}

fn require_planar_violation<Denial: std::fmt::Debug, Value: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<Denial, Value>,
) {
    let WorthQueryApplicationMutationOutcome::Commit(
        primary_graph::WorthQueryApplicationUncommitted::Denied(denial),
    ) = outcome
    else {
        panic!("the candidate must reach the actual invariant");
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    assert_eq!(
        denial.stage(),
        primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let Some(identity) = denial.custom_invariant_violation_identity() else {
        panic!("a domain violation, not an execution failure, must refuse the edit");
    };
    assert_eq!(identity.rule_id.as_str(), "PositivePlanarTurn");
}

#[test]
fn complete_invariant_reads_stay_local_and_preserve_real_refusals() {
    main();
}
