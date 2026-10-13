//! Master parity at the real reconstruction boundary after a commit.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};
use crate::domain_computation::primary_graph::WorthQueryDerivedPairReadPlans;
use worth_execution::ExecutionWorkCeiling;
#[test]
fn members_admitted_on_another_branch_are_foreign_after_the_reads() {
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
                        "foreign-branch-parity",
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
            let fork = world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap();
            let fork_product = world
                .application
                .product_runtime()
                .admit_product_occurrence(fork.occurrence())
                .unwrap();
            let sibling = world
                .application
                .select_product_branch(&execution, fork_product.observation().branch_identity())
                .unwrap();
            let sibling_principal = sibling
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let sibling_root = sibling
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "unrelated".into(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            assert_eq!(sibling_root.entity_id(), roots[1].entity_id());
            assert_ne!(
                sibling.product().relational_basis().identity().branch_id(),
                old.product().relational_basis().identity().branch_id()
            );
            let second_access =
                WorthQueryApplicationQueryAccessContext::new(&sibling_principal, &sibling_root);
            let mut first = Some(first);
            let request = execution;
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
                                let current = &sibling;
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
            assert_eq!(
                answer,
                Err(WorthQueryManagedDerivedViewDenial::ForeignBranch)
            );
            assert!(work.charged_work() > 0);
        })
        .unwrap();
}
