//! Master parity at the real reconstruction boundary after a commit.
use crate::domain_computation::primary_graph::tests::application_query::derived_view::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PublicAccountMembershipQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PublicScopedAccountSummaryQuery;
use worth_query_execution::facade::WorthQueryDerivedPairReadPlans;
use worth_execution::ExecutionWorkCeiling;
#[test]
fn members_admitted_across_a_commit_are_stale_after_the_same_charged_reads() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let execution = host.borrowed_request_for_test();
            let old = world.selected_product(&host);
            let principal = old
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let roots = ["open", "unrelated"].map(|status| {
                old.resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    status.into(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
            });
            assert!(roots[0].entity_id_for_test() < roots[1].entity_id_for_test());
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &roots[0]);
            let schema = world.application.installed_schema();
            let membership_query = schema
                .certification_query(PublicAccountMembershipQuery::reference())
                .unwrap();
            let query = schema
                .certification_query(PublicScopedAccountSummaryQuery::reference())
                .unwrap();
            let view = world
                .application
                .open_managed_derived_collection_pair(
                    &ApplicationDerivedViewDefinition::new(
                        "stale-member-parity",
                        PublicAccountMembershipQuery::reference(),
                        ApplicationDerivedViewLimits::bounded(8, 32768),
                    ),
                    &membership_query,
                    &query,
                    &query,
                    old.product(),
                )
                .unwrap();
            let membership = world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    old.retain_selection_for_test(&execution)
                        .unwrap()
                        .admit_application_query(
                            &phase,
                            &membership_query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            current_controls(&scope),
                        )
                        .unwrap(),
                )
                .unwrap();
            let admit = || {
                old.retain_selection_for_test(&execution)
                    .unwrap()
                    .admit_application_query(
                        &phase,
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&scope),
                    )
                    .unwrap()
            };
            let first = WorthQueryDerivedPairReadPlans::new(admit(), admit());
            let graph = (*(*world.application.runtime_for_test()).primary_provider_for_test())
                .graph_for_test();
            let label = AccountLabel::reference();
            let locator = (*graph.layout_for_test())
                .field_locator_for_test(label.entity(), label.aspect(), label.field())
                .unwrap()
                .clone();
            let second_access = WorthQueryApplicationQueryAccessContext::new(&principal, &roots[1]);
            let mut first = Some(first);
            let request = execution;
            let (answer, work) = request
                .run(ExecutionWorkCeiling::new(None), |_| {
                    world
                        .application
                        .reconstruct_managed_derived_collection_pair(
                            request,
                            &view,
                            old.product(),
                            &membership,
                            |_| {
                                roots
                                    .iter()
                                    .zip(["open", "unrelated"])
                                    .map(|(root, status)| {
                                        (root.entity_id_for_test(), status.to_owned())
                                    })
                                    .collect()
                            },
                            |member: &String| {
                                if member == "open" {
                                    return Ok(first.take().unwrap());
                                }
                                publish_relational_mutation(
                                    &execution,
                                    &world,
                                    WorkerIntentBatch::new("advance-before-second-member").push(
                                        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                                            UpdateEntityFieldsIntent {
                                                entity_id: roots[1].entity_id_for_test(),
                                                fields: AspectFieldPatch::from(BTreeMap::from([(
                                                    locator.clone(),
                                                    StringApplicationValueBinding::encode(
                                                        &"changed".to_owned(),
                                                    )
                                                    .unwrap(),
                                                )])),
                                            },
                                        )),
                                    ),
                                );
                                let current = world.selected_product(&host);
                                let admit = || {
                                    current
                                        .retain_selection_for_test(&execution)
                                        .unwrap()
                                        .admit_application_query(
                                            &phase,
                                            &query,
                                            &second_access,
                                            ApplicationQueryParameterSet::new(),
                                            current_controls(&scope),
                                        )
                                        .unwrap()
                                };
                                Ok(WorthQueryDerivedPairReadPlans::new(admit(), admit()))
                            },
                            |row| row.status().to_owned(),
                            |row| row.status().to_owned(),
                            |first, _| SceneLabel(first.label().to_owned()),
                        )
                })
                .unwrap();
            assert_eq!(answer, Err(WorthQueryManagedDerivedViewDenial::StaleSource));
            assert!(work.charged_work() > 0);
        })
        .unwrap();
}
