//! Run with `cargo run -p worth-query-topology-entry --example invariant_reads`.
//! The application observer records platform values; only the example's checks
//! compare them. It supplies neither access nor admission policy.
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

#[allow(
    dead_code,
    reason = "Reuse the complete reference program, including its output aliases."
)]
#[path = "required_feature/program.rs"]
mod program;
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
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
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
        assert!(matches!(
            changed,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ));
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
        let (preparation, evaluation) = compare_forward_reads(&recorded, query);
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
        let (refused_preparation, refused_evaluation) =
            compare_forward_reads(&rejected, retained.receipt().inspect().read_work().reads());
        assert_eq!(preparation, refused_preparation);
        assert!(
            refused_evaluation.field_reads() > refused_preparation.field_reads(),
            "ordinary violation retains acquired reads"
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

// compiled-example: begin
fn compare_forward_reads(
    events: &[PlanarInvariantReadEvent],
    query: WorthQueryInvariantProjectionWork,
) -> (
    WorthQueryInvariantProjectionWork,
    WorthQueryInvariantProjectionWork,
) {
    let touched: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::Touched { entity } => Some(*entity),
            _ => None,
        })
        .collect();
    assert_eq!(touched.len(), 1, "this command edits one identified body");
    let forward: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::ForwardHops {
                root,
                before,
                after,
            } if *root == touched[0] => Some((*before, *after)),
            _ => None,
        })
        .collect();
    assert_eq!(forward.len(), 1, "compare the same directly touched root");
    let (before, after) = forward[0];
    assert!(query.adjacency_lists_read() > 0);
    assert_eq!(
        after.adjacency_lists_read() - before.adjacency_lists_read(),
        query.adjacency_lists_read()
    );
    assert_eq!(
        after.adjacency_edges_inspected() - before.adjacency_edges_inspected(),
        query.adjacency_edges_inspected()
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
    let Some(primary_graph::WorthQueryCustomInvariantDenial::Violation { identity, .. }) =
        denial.custom_invariant_denial()
    else {
        panic!("a domain violation, not an execution failure, must refuse the edit");
    };
    assert_eq!(identity.rule_id.as_str(), "PositivePlanarTurn");
}

#[test]
fn invariant_reads_match_query_edges_and_stay_local_through_refusal() {
    main();
}
