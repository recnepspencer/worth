use crate::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruct_parallel::read_ceiling_tests::*;
#[test]
fn map_read_preserves_its_callers_work_stop_and_releases_transient_memory() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let scope = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let old = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
            );
            let unrelated = old
                .resolve_entity(
                    &host_custody.phase_for_application_for_test(&world.application),
                    AccountStatus::at("AccountStatus"),
                    "unrelated".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
            let field = AccountStatus::at("AccountStatus");
            let locator = (*graph
                .layout_for_test())
                .field_locator_for_test(field.entity(), field.aspect(), field.field())
                .unwrap()
                .clone();
            drop(old);
            // The declared edit makes exactly two indexed roots match "open".
            publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                WorkerIntentBatch::new("two-lookup-candidates").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: unrelated.entity_id_for_test(),
                        fields: AspectFieldPatch::from(BTreeMap::from([(
                            locator,
                            StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(),
                        )])),
                    }),
                )),
            );
            let selected = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
            );
            let external = world.authenticate("alice", Duration::from_secs(60), &scope);
            let principal = selected
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &host_custody.phase_for_application_for_test(&world.application),
                    AccountLabel::at("AccountLabel"),
                    "primary".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let first_query = world
                .application
                .installed_schema()
                .certification_query(PublicScopedAccountSummaryQuery::reference())
                .unwrap();
            let second_query = world
                .application
                .installed_schema()
                .certification_query(NestedAccountQuery::reference())
                .unwrap();
            let first = selected
                .retain_selection_for_test(execution)
                .unwrap()
                .admit_application_query(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &first_query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&scope),
                )
                .unwrap();
            let second = selected
                .retain_selection_for_test(execution)
                .unwrap()
                .admit_application_query(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &second_query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .unwrap(),
                    current_controls(&scope),
                )
                .unwrap();
            let mut plans = WorthQueryDerivedPairReadPlans::new(first, second);
            execution
                .in_scope(|lease| {
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruction::owner_stage::run(lease, |context| {
                        plans.admit_for_test(execution, &world.application, root.entity_id_for_test(), context)
                    })
                })
                .unwrap()
                .unwrap();
            let input = plans
                .worker_for_test(execution,  root.entity_id_for_test())
                .unwrap();
            let map = ExecutionMap::<_, ()>::from_keyless_values(BTreeMap::from([(PartitionIdentity::new(1), input)]))
            .unwrap();
            let memory_before = execution
                .in_scope(|lease| lease.unwrap().charged_memory_bytes())
                .unwrap();
            let (outcome, _) = execution
                .run(
                    ExecutionWorkCeiling::new(Some(1)),
                    |lease| map.run_owned(lease, |pair, context| pair.run_for_test(execution, context)),
                )
                .unwrap();
            match outcome {
                MapOutcome::Stopped {
                    completed_prefix,
                    reason,
                    ..
                } => {
                    assert!(completed_prefix.is_empty());
                    assert!(
                        matches!(&reason, MapStop::WorkExhausted { identity }
                            if *identity == PartitionIdentity::new(1))
                        || matches!(reason,
                            MapStop::Failure {
                                identity,
                                cause: worth_execution::MapKernelFailure::Domain(Denial::ReadDenied {
                                    root: stopped_root, denial,
                                }),
                            } if identity == PartitionIdentity::new(1)
                                && stopped_root == root.entity_id_for_test()
                                && matches!(denial.kind(),
                                    worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(
                                        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                                            denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                                            partition_identity: None, policy_ancestor: None,
                                        }
                                    ))
                        ),
                        "the pair read preserves its caller's typed work exhaustion"
                    );
                }
                MapOutcome::Complete { .. } => panic!("the pair read cannot bypass its caller's ceiling"),
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
