use super::*;
#[test]
fn query_local_read_ceiling_keeps_the_ordinary_denial_and_admitted_work() {
    let world = installed_authorization_world(true);
    let scope = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let selected = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            let external = world.authenticate("alice", Duration::from_secs(60), &scope);
            let principal = selected
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application(&world.application),
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &host_custody.phase_for_application(&world.application),
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let query = world
                .application
                .installed_schema()
                .certification_query(PublicScopedAccountSummaryQuery::reference())
                .unwrap();
            let admit = |work| {
                selected
                    .retain_selection(execution)
                    .unwrap()
                    .admit_application_query(
                        &host_custody.phase_for_application(&world.application),
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        WorthQueryProductQueryControls::new(
                            NonZeroUsize::new(10).unwrap(),
                            NonZeroUsize::new(work).unwrap(),
                            &scope,
                        ),
                    )
                    .unwrap()
            };
            let ordinary = world
                .application
                .execute_application_query_one_shot(
                    &host_custody.phase_for_application(&world.application),
                    admit(3),
                )
                .err()
                .expect("the three-unit read cannot materialize its two fields");
            let fields = query
                .read_family_binding()
                .planning_contract()
                .projections()
                .len() as u64;
            let first_read = 1 + (1 + fields) + (1 + fields);
            let mut plans = WorthQueryDerivedPairReadPlans::new(admit(100000), admit(3));
            execution
                .in_scope(|lease| {
                    owner_stage::run(lease, |context| {
                        plans.admit(execution, &world.application, root.entity_id(), context)
                    })
                })
                .unwrap()
                .unwrap();
            let capacity = plans.result_capacity().unwrap();
            let map = ExecutionMap::<_, ()>::from_keyless_partitions(BTreeMap::from([(
                PartitionIdentity::new(1),
                KeylessPartition {
                    value: plans
                        .worker(execution, &world.application, root.entity_id())
                        .unwrap(),
                    kernel_scratch_bytes: 0,
                    max_result_bytes: capacity,
                },
            )]))
            .unwrap();
            let memory_before = execution
                .in_scope(|lease| lease.unwrap().charged_memory_bytes())
                .unwrap();
            let (outcome, _) = execution
                .run(ExecutionWorkCeiling::new(u64::MAX), |lease| {
                    map.run_owned(lease, |pair, context| pair.run(execution, context))
                })
                .unwrap();
            match outcome {
                MapOutcome::Stopped {
                    completed_prefix,
                    reason,
                    report,
                    ..
                } => {
                    assert!(completed_prefix.is_empty());
                    assert_eq!(
                        reason,
                        MapStop::Failure {
                            identity: PartitionIdentity::new(1),
                            cause: worth_execution::MapKernelFailure::Domain(Denial::ReadDenied {
                                root: root.entity_id(),
                                denial: ordinary
                            })
                        }
                    );
                    // The second selection's one unit is admitted; its three-unit
                    // projection is refused by the existing Query-local limit.
                    assert_eq!(report.charged_work(), 1 + first_read + 1);
                }
                MapOutcome::Complete { .. } => panic!("the Query-local read must deny"),
            }
            let memory_after = execution
                .in_scope(|lease| lease.unwrap().charged_memory_bytes())
                .unwrap();
            assert_eq!(
                memory_after, memory_before,
                "the stopped reconstruction read releases all transient memory"
            );
        })
        .expect("the fixture block owns its installed host call")
}
