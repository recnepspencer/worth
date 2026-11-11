//! Canonical work refusal with two distinct roots and actual parallel dispatch.
use crate::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruct_parallel::read_ceiling_tests::*;
#[test]
fn parallel_read_request_exhaustion_keeps_a_canonical_prefix_and_releases_memory() {
    use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
    use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
    use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner;
    let _owner = isolated_request_owner();
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    place(Placement::Leased(NonZeroUsize::new(4).unwrap()));
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
                        AccountStatus::at("AccountStatus"),
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
    for workers in [2, 4] {
        world.application.with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            for ceiling in [u64::MAX, 1] {
                let lease = host_custody
                    .borrowed_request_for_test()
                    .in_scope(|parent| {
                        let parent = parent.expect("the active phase carries a lease");
                        parent
                            .child(worth_execution::LeaseRequest {
                                policy: worth_foundational::ExecutionRequestPolicy::new(
                                    parent.policy().posture(),
                                    parent.policy().determinism(),
                                    worth_foundational::ExecutionBudget::new(
                                        NonZeroUsize::new(workers).unwrap(),
                                        None,
                                        parent.policy().work_ceiling(),
                                    ),
                                ),
                                cancellation: scope.cancellation().execution_token(),
                                deadline: scope.deadline(),
                            })
                            .unwrap()
                    })
                    .unwrap();
                let request = worth_execution::ExecutionRequest::leased(&lease);
                let access = roots
                    .iter()
                    .map(|root| WorthQueryApplicationQueryAccessContext::new(&principal, root))
                    .collect::<Vec<_>>();
                let mut plans = access
                    .iter()
                    .map(|access| {
                        let admit = || {
                            selected
                                .retain_selection_for_test(&host_custody.borrowed_request_for_test())
                                .unwrap()
                                .admit_application_query(
                                    &phase,
                                    &query,
                                    access,
                                    ApplicationQueryParameterSet::new(),
                                    current_controls(&scope),
                                )
                                .unwrap()
                        };
                        WorthQueryDerivedPairReadPlans::new(admit(), admit())
                    })
                    .collect::<Vec<_>>();
                request
                    .in_scope(|lease| {
                        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruction::owner_stage::run(lease, |context| {
                            for (pair, root) in plans.iter_mut().zip(&roots) {
                                pair.admit_for_test(
                                    &host_custody.borrowed_request_for_test(),
                                    &world.application,
                                    root.entity_id_for_test(),
                                    context,
                                )?;
                            }
                            Ok(())
                        })
                    })
                    .unwrap()
                    .unwrap();
                let (witness, _probe) =
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::dispatch_witness::DispatchWitness::install(
                Some(roots[1].entity_id_for_test()),
            );

let partitions = plans
                    .iter_mut()
                    .zip(&roots)
                    .enumerate()
                    .map(|(index, (pair, root))| {

                        (
                            PartitionIdentity::new(index as u64 + 1),
                            pair
                                    .worker_for_test(&request,  root.entity_id_for_test())
                                    .unwrap(),
                        )
                    })
                    .collect();
                let map = ExecutionMap::<_, ()>::from_keyless_values(partitions).unwrap();
                let (outcome, _) = ExecutionWorkCeiling::new(Some(ceiling))
                    .run(&lease, || {
                        map.run_owned(Some(&lease), |pair, context| {
                            pair.run_for_test(&host_custody.borrowed_request_for_test(), context)
                        })
                    })
                    .unwrap();
                match outcome {
                    MapOutcome::Stopped { completed_prefix, reason, .. } => {
                        assert_eq!(ceiling, 1);
                        assert!(completed_prefix.is_empty());
                        assert!(matches!(&reason, MapStop::WorkExhausted { identity }
                            if *identity == PartitionIdentity::new(1))
                            || matches!(reason, MapStop::Failure {
                                identity,
                                cause: worth_execution::MapKernelFailure::Domain(Denial::ReadDenied { root, denial }),
                            } if identity == PartitionIdentity::new(1)
                                && root == roots[0].entity_id_for_test()
                                && matches!(denial.kind(),
                                    worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(
                                        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                                            denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                                            partition_identity: None, policy_ancestor: None,
                                        }))),
                            "the parallel read preserves its caller's typed work exhaustion");
                    }
                    MapOutcome::Complete { values, .. } => {
                        assert_eq!(ceiling, u64::MAX);
                        assert_eq!(values.len(), roots.len());
                        for (value, root) in values.iter().zip(&roots) {
                            assert_eq!((*(*value.first_for_test().raw.raw_for_test()).rows_for_test()).raw_rows()[0].entity_id(), root.entity_id_for_test());
                        }
                        let (entries, completions) = witness.observations();
                        assert_eq!(entries.len(), 2);
                        assert_eq!(completions[0], roots[1].entity_id_for_test());
                    }
                }
                drop(lease.reserve_memory(16 << 20).unwrap());
            }
        })
        .expect("the fixture read owns its installed host call");
    }
}
