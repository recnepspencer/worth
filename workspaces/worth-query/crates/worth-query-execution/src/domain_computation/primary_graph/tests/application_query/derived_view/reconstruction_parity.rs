//! Master parity at the real reconstruction boundary after a commit.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    publish_relational_mutation, PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};
use crate::domain_computation::primary_graph::WorthQueryDerivedPairReadPlans;
use worth_execution::{ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest};
#[test]
fn members_admitted_across_a_commit_are_stale_after_the_same_charged_reads() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
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
            assert!(roots[0].entity_id() < roots[1].entity_id());
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
                    old.retain_selection(&execution)
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
                old.retain_selection(&execution)
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
            let graph = &world.application.primary_provider.graph;
            let label = AccountLabel::reference();
            let locator = graph
                .layout
                .field_locator(label.entity(), label.aspect(), label.field())
                .unwrap()
                .clone();
            let second_access = WorthQueryApplicationQueryAccessContext::new(&principal, &roots[1]);
            let mut first = Some(first);
            let serial = SerialRequest::from_memory(
                SerialMemoryBudget::new(16 << 20),
                scope.cancellation().execution_token(),
                scope.deadline(),
            );
            let request = worth_execution::ExecutionRequest::serial(&serial);
            let (answer, work) = request
                .run(ExecutionWorkCeiling::new(u64::MAX), |_| {
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
                                    .map(|(root, status)| (root.entity_id(), status.to_owned()))
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
                                                entity_id: roots[1].entity_id(),
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
                                        .retain_selection(&execution)
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
            println!("stale-member-refusal charged={}", work.charged_work());
            // Frozen on bdb91675a9: includes the real publication and later admission
            // inside preparation, plus all four reads before the stale-source refusal.
            const MASTER_STALE_MEMBER_WORK: u64 = 2271;
            assert_eq!(work.charged_work(), MASTER_STALE_MEMBER_WORK);
        })
        .unwrap();
}
