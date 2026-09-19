use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_query::ApplicationQueryParameterSet, primary_graph,
    product::WorthQueryProductBranch,
};

use super::Population;
use crate::product_query_support::principal;
use crate::query_probe::{read, QueryWork};
use crate::schema::*;
use crate::world::{self, CourtroomWorld};

pub(super) fn read_work(size: usize) -> (QueryWork, Population) {
    let world = CourtroomWorld::publish("ready");
    with_population(&world, size, || {
        let population = Population::capture(&world);
        let work = read(&world, world.application.current_world()).work;
        (work, population)
    })
}

pub(crate) fn assert_live_publication_slopes() {
    let mut baseline = None;
    for size in [1, 8, 64] {
        let world = CourtroomWorld::publish("ready");
        let branch = world.application.current_world();
        let work = with_population(&world, size, || {
            let receipt = world
                .change_input_on_branch_with_ordinal(
                    branch,
                    &format!("live-population-{size}"),
                    size as u8,
                )
                .require_committed()
                .expect("the configured live population must leave mutation capacity");
            MutationWork::from_receipt(&receipt)
        });
        assert_eq!(*baseline.get_or_insert(work), work, "L={size}");
    }
}

pub(crate) fn assert_graph_work_capacity_bounds() {
    let exact = CourtroomWorld::publish_with_graph_work_limit("ready", 2);
    with_population(&exact, 2, || ());

    let below = CourtroomWorld::publish_with_graph_work_limit("ready", 1);
    let request = world::request_scope();
    let principal = principal(&below, &request);
    let root = below.application.current_world();
    let open = || {
        let query = below
            .application
            .installed_schema()
            .certification_query(TemporalIntentLiveQuery::reference())
            .unwrap();
        let scope = live_scope(&below, root, &request);
        below
            .application
            .on_branch(root)
            .select()
            .unwrap()
            .open_application_query_live::<
                TemporalIntentLiveQuery,
                IntentLiveQueryParameters,
                IntentLiveQueryResult,
                _,
                _,
                TemporalIntent,
                TemporalIntent,
                TemporalIntentLiveCause,
            >(
                query,
                &principal,
                scope,
                ApplicationQueryParameterSet::new(),
                live_controls(&request),
            )
    };
    let first = open().expect("the installed one-slot limit must admit exactly one lease");
    let denial = match open() {
        Ok(_) => panic!("one below the required two-slot population must deny the second lease"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryApplicationLiveOpenDenialKind::Admission(
            primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::GraphWorkAdmissionUnavailable,
        )
    );
    assert!(matches!(
        first.close(),
        primary_graph::WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
    let retried = open().expect("releasing the first lease must restore the installed slot");
    assert!(matches!(
        retried.close(),
        primary_graph::WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}

fn with_population<Result>(
    world: &CourtroomWorld,
    size: usize,
    probe: impl FnOnce() -> Result,
) -> Result {
    let request = world::request_scope();
    let principal = principal(world, &request);
    let root = world.application.current_world();
    let live = (0..size)
        .map(|_| {
            let query = world
                .application
                .installed_schema()
                .certification_query(TemporalIntentLiveQuery::reference())
                .unwrap();
            let scope = live_scope(world, root, &request);
            world
                .application
                .on_branch(root)
                .select()
                .unwrap()
                .open_application_query_live::<
                    TemporalIntentLiveQuery,
                    IntentLiveQueryParameters,
                    IntentLiveQueryResult,
                    _,
                    _,
                    TemporalIntent,
                    TemporalIntent,
                    TemporalIntentLiveCause,
                >(
                    query,
                    &principal,
                    scope,
                    ApplicationQueryParameterSet::new(),
                    live_controls(&request),
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    let result = probe();
    for consumer in live {
        assert!(matches!(
            consumer.close(),
            primary_graph::WorthQueryApplicationLiveCloseOutcome::Completed(_)
        ));
    }
    result
}

fn live_scope(
    world: &CourtroomWorld,
    root: WorthQueryProductBranch,
    request: &WorthQueryRequestScope,
) -> primary_graph::WorthQueryApplicationEntityIdentity<TemporalHostSchema, TemporalIntent> {
    world
        .application
        .on_branch(root)
        .select()
        .unwrap()
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_owned(),
            request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

fn live_controls(
    request: &WorthQueryRequestScope,
) -> primary_graph::WorthQueryApplicationLiveControls {
    primary_graph::WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16, 2_048)
        .unwrap()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MutationWork {
    changed_records: usize,
    touched_records: usize,
    decision_facts: usize,
    proposed_facts: usize,
    invariant_state_facts: usize,
    invariant_work_units: u64,
    preimage_targets: usize,
    performed_targets: usize,
}

impl MutationWork {
    fn from_receipt(receipt: &primary_graph::WorthQueryApplicationCommitReceipt) -> Self {
        let mutation = receipt
            .mutation_work()
            .expect("a performed public mutation must carry owner-derived work");
        Self {
            changed_records: receipt.changed_record_count(),
            touched_records: mutation.touched_record_count(),
            decision_facts: mutation.decision_fact_count(),
            proposed_facts: mutation.proposed_fact_count(),
            invariant_state_facts: mutation.invariant_state_fact_count(),
            invariant_work_units: mutation.invariant_work_units(),
            preimage_targets: mutation.preimage_mutation_targets_materialized(),
            performed_targets: mutation.performed_touch_targets_materialized(),
        }
    }
}
