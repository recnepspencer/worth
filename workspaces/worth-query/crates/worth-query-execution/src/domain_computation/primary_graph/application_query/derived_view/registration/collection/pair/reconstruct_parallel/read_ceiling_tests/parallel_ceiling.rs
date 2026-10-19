//! Canonical work refusal with two distinct roots and actual parallel dispatch.
use super::*;
#[test]
fn parallel_read_request_exhaustion_keeps_a_canonical_prefix_and_releases_memory() {
    use crate::domain_computation::primary_graph::application_contribution::{
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    };
    use crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner;
    let _owner = isolated_request_owner();
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    place(Placement::Leased(NonZeroUsize::new(4).unwrap()));
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
    for workers in [2, 4] {
        world.application.with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
                                        parent.policy().budget().work_ceiling(),
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
                                .retain_selection(&host_custody.borrowed_request_for_test())
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
                        owner_stage::run(lease, |context| {
                            for (pair, root) in plans.iter_mut().zip(&roots) {
                                pair.admit(
                                    &host_custody.borrowed_request_for_test(),
                                    &world.application,
                                    root.entity_id(),
                                    context,
                                )?;
                            }
                            Ok(())
                        })
                    })
                    .unwrap()
                    .unwrap();
                let (witness, _probe) =
            crate::domain_computation::primary_graph::application_query::DispatchWitness::install(
                Some(roots[1].entity_id()),
            );
                let partitions = plans
                    .iter_mut()
                    .zip(&roots)
                    .enumerate()
                    .map(|(index, (pair, root))| {
                        let capacity = pair.result_capacity().unwrap();
                        (
                            PartitionIdentity::new(index as u64 + 1),
                            KeylessPartition {
                                value: pair
                                    .worker(&request, &world.application, root.entity_id())
                                    .unwrap(),
                                kernel_scratch_bytes: 0,
                                max_result_bytes: capacity,
                            },
                        )
                    })
                    .collect();
                let map = ExecutionMap::<_, ()>::from_keyless_partitions(partitions).unwrap();
                let (outcome, _) = ExecutionWorkCeiling::new(Some(ceiling))
                    .run(&lease, || {
                        map.run_owned(Some(&lease), |pair, context| {
                            pair.run(&host_custody.borrowed_request_for_test(), context)
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
                                && root == roots[0].entity_id()
                                && matches!(denial.kind(),
                                    crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(
                                        crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                                            denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                                            partition_identity: None, policy_ancestor: None,
                                        }))),
                            "the parallel read preserves its caller's typed work exhaustion");
                    }
                    MapOutcome::Complete { values, .. } => {
                        assert_eq!(ceiling, u64::MAX);
                        assert_eq!(values.len(), roots.len());
                        for (value, root) in values.iter().zip(&roots) {
                            assert_eq!(value.first.raw.raw.rows.raw_rows()[0].entity_id(), root.entity_id());
                        }
                        let (entries, completions) = witness.observations();
                        assert_eq!(entries.len(), 2);
                        assert_eq!(completions[0], roots[1].entity_id());
                    }
                }
                drop(lease.reserve_memory(16 << 20).unwrap());
            }
        })
        .expect("the fixture read owns its installed host call");
    }
}
