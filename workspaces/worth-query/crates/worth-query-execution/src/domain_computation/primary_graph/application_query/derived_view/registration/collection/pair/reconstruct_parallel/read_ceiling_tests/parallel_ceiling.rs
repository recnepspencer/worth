//! Canonical work refusal with two distinct roots and actual parallel dispatch.
use super::*;
#[test]
fn parallel_work_exhaustion_retains_only_the_declared_completed_root_prefix() {
    use crate::domain_computation::primary_graph::application_contribution::{
        query_test_execution_authority_for_test, test_policy,
    };
    use crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner;
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
    let root_cost = 1 + 2 * (1 + (1 + fields) + (1 + fields));
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            for workers in [2, 4] {
                let lease = query_test_execution_authority_for_test()
                    .request_lease(worth_execution::LeaseRequest {
                        policy: test_policy(NonZeroUsize::new(workers).unwrap(), 16 << 20),
                        cancellation: scope.cancellation().execution_token(),
                        deadline: scope.deadline(),
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
                                value: pair.worker(&world.application, root.entity_id()).unwrap(),
                                kernel_scratch_bytes: 0,
                                max_result_bytes: capacity,
                            },
                        )
                    })
                    .collect();
                let map = ExecutionMap::<_, ()>::from_keyless_partitions(partitions).unwrap();
                let (outcome, _) = ExecutionWorkCeiling::new(2 * root_cost - 1)
                    .run(&lease, || {
                        map.run_owned(Some(&lease), |pair, context| {
                            pair.run(&host_custody.borrowed_request_for_test(), context)
                        })
                    })
                    .unwrap();
                match outcome {
                    MapOutcome::Stopped {
                        completed_prefix,
                        reason,
                        report,
                        ..
                    } => {
                        assert_eq!(completed_prefix.len(), 1);
                        assert_eq!(
                            completed_prefix[0].first.raw.raw.rows.raw_rows()[0].entity_id(),
                            roots[0].entity_id()
                        );
                        assert_eq!(
                            reason,
                            MapStop::WorkExhausted {
                                identity: PartitionIdentity::new(2)
                            }
                        );
                        assert_eq!(
                    report.charged_work(),
                    root_cost,
                    "canonical settlement refuses the second complete root without charging it"
                );
                    }
                    MapOutcome::Complete { .. } => {
                        panic!("two roots exceed their one-short total ceiling")
                    }
                }
                let (entries, completions) = witness.observations();
                assert_eq!(entries.len(), 2);
                assert_eq!(completions[0], roots[1].entity_id());
                drop(lease.reserve_memory(16 << 20).unwrap());
            }
        })
        .expect("the fixture read owns its installed host call")
}
