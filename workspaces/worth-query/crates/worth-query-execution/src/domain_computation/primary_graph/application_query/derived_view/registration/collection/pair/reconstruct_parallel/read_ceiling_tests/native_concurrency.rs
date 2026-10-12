//! Native read overlap is established by events, never elapsed throughput.
use super::*;
use crate::domain_computation::primary_graph::application_query::read_execution::dispatch_witness::NativeReadRendezvousGuardForTest;
#[test]
fn two_distinct_roots_hold_the_whole_native_read_without_the_owner_lock() {
    use crate::domain_computation::primary_graph::application_contribution::query_test_execution_authority_for_test;
    use crate::domain_computation::primary_graph::application_contribution::test_policy;
    use crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner;
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _owner = isolated_request_owner();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
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
    let fields = query
        .read_family_binding()
        .planning_contract()
        .projections()
        .len() as u64;
    // One invocation. Each of two reads selects one root, projects one record
    // and each declared field, and observes one entity and each declared field.
    let invocation = 1;
    let root_selection = 1;
    let projected_record = 1;
    let projected_field = 1;
    let observed_entity = 1;
    let observed_field = 1;
    let reads_per_root = 2;
    let root_cost = invocation
        + reads_per_root
            * (root_selection
                + projected_record
                + fields * projected_field
                + observed_entity
                + fields * observed_field);
    let owner_probe = world
        .application
        .primary_provider
        .graph
        .source_owner
        .read_lock_probe_for_test();
    assert!(!owner_probe.is_locked_for_test());
    world.application.with_host_advancement(|phase| {
        let host_custody = phase.test_host_request();
        let lease = query_test_execution_authority_for_test().request_lease(worth_execution::LeaseRequest {
            policy: test_policy(NonZeroUsize::new(2).unwrap(), 16 << 20),
            cancellation: scope.cancellation().execution_token(),
            deadline: scope.deadline(),
        }).unwrap();
        let request = ExecutionRequest::leased(&lease);
        let acquisitions = owner_probe.acquisitions_for_test();
        owner_probe.with_owner_lock_for_test(&request, || assert!(owner_probe.is_locked_for_test()));
        assert_eq!(owner_probe.acquisitions_for_test() - acquisitions, 1);
        let before_bridge = owner_probe.acquisitions_for_test();
        owner_probe.count_bridge_lock_for_test(&request);
        assert_eq!(owner_probe.acquisitions_for_test() - before_bridge, 2, "Query basis lookup and cloned Bridge handle both acquire the owner");

        let access = roots.iter().map(|root| {
            WorthQueryApplicationQueryAccessContext::new(&principal, root)
        }).collect::<Vec<_>>();
        let mut plans = access.iter().map(|access| {
            let admit = || selected.retain_selection(&host_custody.borrowed_request_for_test())
                .unwrap().admit_application_query(
                    &phase, &query, access, ApplicationQueryParameterSet::new(), current_controls(&scope),
                ).unwrap();
            WorthQueryDerivedPairReadPlans::new(admit(), admit())
        }).collect::<Vec<_>>();
        request.in_scope(|lease| owner_stage::run(lease, |context| {
            for (pair, root) in plans.iter_mut().zip(&roots) {
                pair.admit(&host_custody.borrowed_request_for_test(), &world.application, root.entity_id(), context)?;
            }
            Ok(())
        })).unwrap().unwrap();
        let (witness, _installed) = crate::domain_computation::primary_graph::application_query::DispatchWitness::install(None);
        let held: NativeReadRendezvousGuardForTest = witness.hold_native_reads_for_test(
            [roots[0].entity_id(), roots[1].entity_id()], owner_probe.clone(), &request,
        );
        let before_release = owner_probe.acquisitions_for_test();
        witness.release_native_snapshot_for_test();
        assert_eq!(owner_probe.acquisitions_for_test() - before_release, 1, "a real snapshot release acquires the same owner");
        witness.with_native_owner_lock_for_test(&request, || assert!(owner_probe.is_locked_for_test()));
        let mut pins = Default::default();
        let reads_before = crate::facade::primary_graph::installed_source_reads_on_this_thread_for_test();
        let partitions = plans.iter_mut().zip(&roots).enumerate().map(|(index, (pair, root))| {
            let capacity = pair.result_capacity().unwrap();
            (PartitionIdentity::new(index as u64 + 1), KeylessPartition {
                value: pair.worker_with_pins(&request, &world.application, root.entity_id(), &mut pins).unwrap(),
                kernel_scratch_bytes: 0, max_result_bytes: capacity,
            })
        }).collect();
        assert_eq!(crate::facade::primary_graph::installed_source_reads_on_this_thread_for_test() - reads_before, 2, "one owner pin per query side, shared across both roots");
        let map = ExecutionMap::<_, ()>::from_keyless_partitions(partitions).unwrap();
        let (outcome, _) = ExecutionWorkCeiling::new(2 * root_cost).run(&lease, || {
            map.run_owned(Some(&lease), |pair, context| pair.run(&host_custody.borrowed_request_for_test(), context))
        }).unwrap();
        match outcome {
            MapOutcome::Complete { values: results, report, .. } => {
                assert_eq!(results.len(), 2);
                assert_eq!(report.charged_work(), 2 * root_cost);
                for (result, root) in results.iter().zip(&roots) {
                    assert_eq!(result.first.raw.raw.rows.raw_rows()[0].entity_id(), root.entity_id());
                    assert_eq!(result.second.raw.raw.rows.raw_rows()[0].entity_id(), root.entity_id());
                }
            }
            MapOutcome::Stopped { reason, .. } => panic!("the declared complete read was stopped: {reason:?}"),
        }
        held.assert_overlap_for_test();
    }).unwrap();
}
