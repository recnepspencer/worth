//! A closed complete tree hands off genuine consumed-output custody at successor admission.
use super::*;
use worth_query_host::facade::primary_graph::{
    fact_verification_on_this_thread_for_test as take, WorthQueryExecutionObservation,
    WorthQueryRegistryCustodyTransitionForTest as Transition,
};

macro_rules! settle {
    ($demand:expr, $request:expr) => {
        (0..256)
            .find_map(|_| match $demand.advance($request).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
            })
            .expect("the installed native demand settles")
    };
}

mod value_edit;

#[test]
fn changed_native_membership_still_refuses_after_genuine_consumer_handoff() {
    use worth_query_consumer_values::{PlanarVertex, PlanarVertexReplacement};
    use worth_query_host::facade::application_contribution::WorthQueryApplicationProducerProvider;
    let _guard = checkpoint_recovery_test_guard();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    take();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<Program>(
        None,
        profile,
        32,
        128 * 1_024 * 1_024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        super::super::super::source_world::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut root = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<Program, Root>(&application)
        .unwrap();
    let original_root = settle!(root, &request);
    let open = || {
        request
            .demand(ChainDemand("anchor-b".into()))
            .start_dependent_in_program::<Program, super::super::super::program::ChainConnection>(
                &application,
            )
            .unwrap()
    };
    let mut consumer = open();
    let original = settle!(consumer, &request);
    assert_eq!(original.producer_contacts_in_this_demand(), 1);
    drop((consumer, original, root, original_root));
    let closed = take();
    assert!(
        closed
            .registry_custody
            .iter()
            .flatten()
            .any(|event| event.transition == Transition::CachedReadyClosed
                && event.prerequisites == 1),
        "B closes with actual posted consumed-output custody: {closed:?}"
    );
    assert_eq!(closed.registry_custody_omitted, 0);
    let before = request
        .query(PlanarOutputRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    let source = request
        .query(PlanarRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(VertexReplacement {
            scope_key: "anchor-b".into(),
            replacement: PlanarVertexReplacement {
                retired_key: "anchor-c".into(),
                next_key: "anchor-a".into(),
                replacement: PlanarVertex {
                    body_key: "unselected-c".into(),
                    x: length(1),
                    y: length(10),
                },
            },
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3391_u64)
        .execute_in_program::<Program>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(source);
    let after = request
        .query(PlanarOutputRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    assert_ne!(
        before.rows(),
        after.rows(),
        "the genuine source suffix changed"
    );
    let input = |value| {
        <ChainProvider as WorthQueryApplicationProducerProvider<
            CheckpointSchema,
            ChainProducer<CheckpointSchema>,
        >>::operation_input(&ChainProvider::new(false), value)
    };
    assert_eq!(
        input(&before.rows()[0]),
        input(&after.rows()[0]),
        "the whole declared input is equal"
    );
    drop((before, after));
    let edit = take();
    eprintln!("closed B ignored source edit: {edit:?}");
    let mut successor = open();
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let stopped_scope = authentication::WorthQueryRequestScope::new(None, cancellation.token());
    let stopped = application.request(&principal, &stopped_scope);
    cancellation.cancel();
    let denial = successor
        .advance(&stopped)
        .err()
        .expect("cancelled native admission refuses");
    assert!(
        matches!(&denial,
        WorthQueryApplicationOutputDemandDenial::Demand(cause)
        if matches!(cause.kind(), WorthQueryOutputDemandDenialKind::ExecutionRequest(
            WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled)))),
        "the outer native admission preserves cancellation: {denial:?}"
    );
    take_all_decisions();
    let refreshed = settle!(successor, &request);
    let observed = take();
    eprintln!("closed B ignored source successor: {observed:?}");
    assert_eq!(observed.registry_custody_omitted, 0);
    assert!(refreshed.producer_contacts_in_this_demand() > 0);
    assert!(
        observed
            .input_cutoffs
            .iter()
            .flatten()
            .any(
                |event| event.producer == "worth.query.certification.consumed-chain-producer.v1"
                    && event.gate
                        == primary_graph::WorthQueryInputCutoffGateForTest::SelectedMembership
                    && event.outcome == primary_graph::WorthQueryInputCutoffOutcomeForTest::Fresh
            ),
        "native incarnation changes still refuse after actual custody handoff: {observed:?}"
    );
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_output_demand(&refreshed, NonZeroUsize::new(1_000_000).unwrap())
        .unwrap();
    let output = request
        .query(PlanarOutputRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(output.rows()[0].value, length(16));
}

#[test]
fn closed_consumer_equal_source_successor_preserves_posted_custody() {
    let _guard = checkpoint_recovery_test_guard();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    take();
    let mut application = support::install_program::<Program>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let open = || {
        request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .retain_program_outputs::<Program, Root>(&application)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()))
    };
    let first = finish(open(), &request);
    let closed = first
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    let initial = take();
    assert!(
        initial
            .registry_custody
            .iter()
            .flatten()
            .any(|event| event.transition == Transition::SettlementPublished
                && event.prerequisites != 0),
        "the installed consumer really posted consumed-output custody: {initial:?}"
    );
    assert!(
        initial
            .registry_custody
            .iter()
            .flatten()
            .any(
                |event| event.transition == Transition::CachedReadyClosed && event.settlements != 0
            ),
        "ordinary close retains a posted Ready: {initial:?}"
    );
    assert_eq!(initial.registry_custody_omitted, 0);
    let before = request
        .query(PlanarOutputRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    let source = request
        .query(PlanarRead {
            body_key: "anchor-c".into(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-c".into(),
            replacement_y: length(11),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3390_u64)
        .execute_performed::<Program, Root>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(source);
    let after = request
        .query(PlanarOutputRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        before.rows(),
        after.rows(),
        "B's actual producer input remains equal"
    );
    drop((before, after));
    let source_edit = take();
    eprintln!("closed consumer source edit: {source_edit:?}");
    let cancellation = authentication::WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped_scope = authentication::WorthQueryRequestScope::new(None, cancellation.token());
    let stopped = application.request(&principal, &stopped_scope);
    let operation = match open().advance(&stopped) {
        WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
            assert!(
                matches!(
                    denial,
                    WorthQueryProgramOutputAdvanceDenial::Execution(
                        WorthQueryAdvancementDenial::Interrupted(
                            WorthQueryManagedComputationInterruption::Cancelled
                        )
                    )
                ),
                "the actual cancelled advance must refuse: {denial:?}"
            );
            operation
        }
        _ => panic!("cancelled advance must return the same native operation"),
    };
    take_all_decisions();
    let settled = finish(operation, &request);
    let observed = take();
    eprintln!("closed consumer successor: {observed:?}");
    assert_eq!(observed.registry_custody_omitted, 0);
    assert!(
        take_decisions("anchor-b").is_empty(),
        "the equal native consumer must reuse its completed decision: {observed:?}"
    );
    let closed_successor = settled
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    drop((closed, closed_successor));
    drop(request);
    drop((principal, scope));
    application.close_conditional_runtime().unwrap();
}
