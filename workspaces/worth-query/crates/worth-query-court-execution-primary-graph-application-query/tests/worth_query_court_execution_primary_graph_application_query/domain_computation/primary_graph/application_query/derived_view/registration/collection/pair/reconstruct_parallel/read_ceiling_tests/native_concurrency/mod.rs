//! Native read overlap is established by events, never elapsed throughput.
use crate::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruct_parallel::read_ceiling_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::dispatch_witness::native_rendezvous::NativeReadRendezvousGuardForTest;
#[test]
fn two_distinct_roots_hold_the_whole_native_read_without_the_owner_lock() {
    use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner;
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let _owner = isolated_request_owner();
    worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test(
        worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest::Leased(
            NonZeroUsize::new(2).unwrap(),
        ),
    );
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let scope = live_scope();
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let roots = world
        .application
        .with_host_advancement(|phase| {
            ["open", "unrelated"].map(|status| {
                selected
                    .resolve_entity(
                        &phase,
                        AccountStatus::reference(),
                        status.to_owned(),
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
            })
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(PublicScopedAccountSummaryQuery::reference())
        .unwrap();
    let owner_probe = (*(*(*(*world.application.runtime_for_test()).primary_provider_for_test())
        .graph_for_test())
    .source_owner_for_test())
    .read_lock_probe_for_test();
    assert!(!owner_probe.is_locked_for_test());
    world.application.with_host_advancement(|phase| {
        let host_custody = phase.test_host_request_for_test();
        let lease = host_custody.borrowed_request_for_test().in_scope(|parent| {
            let parent = parent.expect("the host lends its active lease");
            parent.child(worth_execution::LeaseRequest {
                policy: *parent.policy(), cancellation: scope.cancellation().execution_token(), deadline: scope.deadline(),
            }).unwrap()
        }).unwrap();
        let request = ExecutionRequest::leased(&lease);
        let acquisitions = owner_probe.acquisitions_for_test();
        owner_probe.with_owner_borrow_for_test(&request, || owner_probe.count_bridge_lock_for_test(&request));
        assert_eq!(owner_probe.acquisitions_for_test(), acquisitions, "nested Query and Bridge reads need no runtime mutex");
        let before_bridge = owner_probe.acquisitions_for_test();
        owner_probe.count_bridge_lock_for_test(&request);
        assert_eq!(owner_probe.acquisitions_for_test(), before_bridge, "Query and Bridge share the same immutable Native owner");

        let access = roots.iter().map(|root| {
            WorthQueryApplicationQueryAccessContext::new(&principal, root)
        }).collect::<Vec<_>>();
        let mut plans = access.iter().map(|access| {
            let admit = || selected.retain_selection_for_test(&host_custody.borrowed_request_for_test())
                .unwrap().admit_application_query(
                    &phase, &query, access, ApplicationQueryParameterSet::new(), current_controls(&scope),
                ).unwrap();
            WorthQueryDerivedPairReadPlans::new(admit(), admit())
        }).collect::<Vec<_>>();
        request.in_scope(|lease| worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruction::owner_stage::run(lease, |context| {
            for (pair, root) in plans.iter_mut().zip(&roots) {
                pair.admit_for_test(&host_custody.borrowed_request_for_test(), &world.application, root.entity_id_for_test(), context)?;
            }
            Ok(())
        })).unwrap().unwrap();
        let (witness, _installed) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::dispatch_witness::DispatchWitness::install(None);
        let held: NativeReadRendezvousGuardForTest = witness.hold_native_reads_for_test(
            [roots[0].entity_id_for_test(), roots[1].entity_id_for_test()], owner_probe.clone(), &request,
        );
        let before_release = owner_probe.acquisitions_for_test();
        witness.release_native_snapshot_for_test();
        assert_eq!(owner_probe.acquisitions_for_test(), before_release, "snapshot release borrows the same immutable Native owner");
        witness.with_native_owner_borrow_for_test(&request, || owner_probe.count_bridge_lock_for_test(&request));
        let mut pins = Default::default();
        let reads_before = worth_query_execution::facade::domain_computation::installed_source_reads_on_this_thread_for_test();

let partitions = plans.iter_mut().zip(&roots).enumerate().map(|(index, (pair, root))| {

            (PartitionIdentity::new(index as u64 + 1), pair.worker_with_pins_for_test(&request,  root.entity_id_for_test(), &mut pins).unwrap())
        }).collect();
        assert_eq!(worth_query_execution::facade::domain_computation::installed_source_reads_on_this_thread_for_test() - reads_before, 2, "one owner pin per query side, shared across both roots");
        let map = ExecutionMap::<_, ()>::from_keyless_values(partitions).unwrap();
        let (outcome, _) = ExecutionWorkCeiling::new(lease.policy().work_ceiling()).run(&lease, || {
            map.run_owned(Some(&lease), |pair, context| pair.run_for_test(&host_custody.borrowed_request_for_test(), context))
        }).unwrap();
        match outcome {
            MapOutcome::Complete { values: results, .. } => {
                assert_eq!(results.len(), 2);
                for (result, root) in results.iter().zip(&roots) {
                    assert_eq!((*(*(*result.first_for_test()).raw.raw_for_test()).rows_for_test()).raw_rows()[0].entity_id(), root.entity_id_for_test());
                    assert_eq!((*(*(*result.second_for_test()).raw.raw_for_test()).rows_for_test()).raw_rows()[0].entity_id(), root.entity_id_for_test());
                }
            }
            MapOutcome::Stopped { reason, .. } => panic!("the declared complete read was stopped: {reason:?}"),
        }
        held.assert_overlap_for_test();
    }).unwrap();
}
