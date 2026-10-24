//! A first native row supplies another root's scope on the same issued basis.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world_with_label, AccountSummaryResult, PublicAccountMembershipQuery,
    PublicScopedAccountSummaryQuery,
};
use std::{cell::Cell, num::NonZeroUsize};
use worth_execution::{CancellationSource, ExecutionRequest};
use WorthQueryManagedDerivedViewDenial as Denial;

#[test]
fn dependent_scope_and_batch_custody_produce_the_retained_image() {
    journey(Case::Success);
}
#[test]
fn narrowed_host_request_lends_dependent_reads_and_preserves_custody() {
    journey(Case::LeasedSuccess);
}
#[test]
fn wrong_second_scope_cannot_replace_the_retained_image() {
    journey(Case::WrongScope);
}
#[test]
fn wrong_link_cannot_replace_the_retained_image() {
    journey(Case::WrongLink);
}
#[test]
fn foreign_second_request_cannot_replace_the_retained_image() {
    journey(Case::ForeignRequest);
}
#[test]
fn cancellation_between_reads_cannot_replace_the_retained_image() {
    journey(Case::Cancelled);
}
#[test]
fn insufficient_request_work_never_publishes_a_prefix() {
    journey(Case::Work);
}
#[test]
fn foreign_planned_loan_cannot_supply_the_second_read() {
    journey(Case::ForeignBatch);
}
#[test]
fn expired_deadline_prevents_reads_and_retention() {
    journey(Case::Deadline);
}
#[test]
fn insufficient_request_memory_prevents_reads_and_retention() {
    journey(Case::Memory);
}
#[test]
fn cancellation_after_projection_still_prevents_retention() {
    journey(Case::AfterProjection);
}
#[test]
fn another_genuine_product_basis_is_refused_before_second_execution() {
    journey(Case::ForeignBasis);
}
#[test]
fn publication_during_projection_refuses_the_now_stale_product() {
    journey(Case::StaleProduct);
}
#[test]
fn ignored_repeated_execution_refusal_cannot_publish_an_earlier_result() {
    journey(Case::RepeatedRead);
}
#[derive(Clone, Copy, PartialEq)]
enum Case {
    LeasedSuccess,
    RepeatedRead,
    ForeignBasis,
    StaleProduct,
    ForeignBatch,
    Deadline,
    Memory,
    AfterProjection,
    Success,
    WrongScope,
    WrongLink,
    ForeignRequest,
    Cancelled,
    Work,
}

fn journey(case: Case) {
    let world = installed_authorization_world_with_label("unrelated", Default::default());
    let scope = live_scope();
    let foreign_scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world.application.with_host_advancement(|phase| {
        let host = phase.test_host_request();
        let issued = host.borrowed_request_for_test();
        let selected = world.selected_product(&host);
        let principal = selected.resolve_authenticated_principal(
            &phase, &world.binding, &external, &scope, WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap();
        let root = selected.resolve_entity(&phase, AccountStatus::reference(), "open".into(),
            &scope, WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
        let sibling = (case == Case::ForeignBasis).then(|| {
            let fork = world.application.branches().fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase).unwrap();
            let product = world.application.product_runtime().admit_product_occurrence(fork.occurrence()).unwrap();
            let sibling = world.application.select_product_branch(&issued, product.observation().branch_identity()).unwrap();
            assert_ne!(sibling.product().relational_basis().descriptor(), selected.product().relational_basis().descriptor());
            sibling
        });
        let sibling_principal = sibling.as_ref().map(|sibling| sibling.resolve_authenticated_principal(
            &phase, &world.binding, &external, &scope, WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap());
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
        let schema = world.application.installed_schema();
        let membership_query = schema.certification_query(PublicAccountMembershipQuery::reference()).unwrap();
        let entry_query = schema.certification_query(PublicScopedAccountSummaryQuery::reference()).unwrap();
        let view = world.application.open_owner_sized_managed_derived_collection_pair(
            "dependent-neutral", &membership_query, &entry_query, &entry_query, selected.product(),
        ).unwrap();
        let membership = world.application.execute_application_query_one_shot(&phase,
            selected.retain_selection(&issued).unwrap().admit_application_query(&phase,
                &membership_query, &access, ApplicationQueryParameterSet::new(), crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new((&membership_query).read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &scope),
            ).unwrap(),
        ).unwrap();
        let nz = |value| NonZeroUsize::new(value).unwrap();
        let cancellation = CancellationSource::new();
        let second_calls = Cell::new(0);
        let projections = Cell::new(0);
        let read_work = Cell::new(0);
        let run = |failure: bool| {
            let issue_batch = || world.application.application_query_batch_from_installed_reads(&[
                world.application.plan_installed_query_batch_reads(&entry_query, 2, std::num::NonZeroUsize::new((&entry_query).read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract")).unwrap()
            ]).unwrap();
            let batch = issue_batch();
            let foreign_batch = issue_batch();
            let narrowed = (failure || case == Case::LeasedSuccess).then(|| {
                issued.in_scope(|parent| {
                    let parent = parent.expect("the installed host phase carries native custody");
                    parent.child(worth_execution::LeaseRequest {
                        policy: worth_foundational::ExecutionRequestPolicy::new(
                            parent.policy().posture(), parent.policy().determinism(),
                            worth_foundational::ExecutionBudget::new(nz(1),
                                if case == Case::Memory { Some(0) } else { None },
                                if case == Case::Work { Some(1) } else { parent.policy().work_ceiling() }),
                        ), cancellation: cancellation.token(),
                        deadline: (case == Case::Deadline).then(std::time::Instant::now),
                    }).unwrap()
                }).unwrap()
            });
            let request = narrowed.as_ref().map_or(issued, ExecutionRequest::leased);
            let second_before = second_calls.get();
            let result = world.application.reconstruct_managed_derived_collection_pair_dependent_in_batch(
                request, &view, selected.product(), &membership,
                |row| row.members.iter().map(|root| (*root, row.tag.clone())).collect(), &batch,
                |_: &String, read| {
                    let local = selected.resolve_entity(&phase, AccountStatus::reference(), "open".into(),
                        &scope, WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
                    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &local);
                    let plan = selected.retain_selection(&issued).unwrap().admit_application_query(
                        &phase, &entry_query, &access, ApplicationQueryParameterSet::new(), crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new((&entry_query).read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &scope),
                    ).unwrap();
                    let result = read.execute(plan)?;
                    read_work.set(result.receipt().work().total_work_units());
                    assert_eq!(result.rows()[0].label(), "unrelated");
                    if failure && case == Case::RepeatedRead {
                        let plan = selected.retain_selection(&issued).unwrap().admit_application_query(
                            &phase, &entry_query, &access, ApplicationQueryParameterSet::new(), crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new((&entry_query).read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &scope),
                        ).unwrap();
                        assert_eq!(read.execute(plan).err(), Some(Denial::IncompleteDependencies));
                        // Intentionally ignore the returned refusal: the owner must still deny the image.
                    }
                    Ok(())
                },
                |first: &AccountSummaryResult, read| {
                    second_calls.set(second_calls.get() + 1);
                    let selected = if failure && case == Case::ForeignBasis { sibling.as_ref().unwrap() } else { &selected };
                    let principal = if failure && case == Case::ForeignBasis { sibling_principal.as_ref().unwrap() } else { &principal };
                    let local_scope = if failure && case == Case::ForeignRequest { &foreign_scope } else { &scope };
                    let selector = if failure && case == Case::WrongScope { "open" } else { first.label() };
                    let local = selected.resolve_entity(&phase, AccountStatus::reference(), selector.to_owned(),
                        local_scope, WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
                    assert!(failure && case == Case::WrongScope || local.entity_id() != root.entity_id());
                    let access = WorthQueryApplicationQueryAccessContext::new(principal, &local);
                    let plan = if failure && case == Case::ForeignBatch {
                        let mut retained = selected.retain_selection(&issued).unwrap();
                        selected.admit_retained_application_query_from_selection(
                            &phase, &mut retained, &entry_query, &access,
                            ApplicationQueryParameterSet::new(),
                            // The batch slot was planned at the installed result contract.
                            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), local_scope),
                            &foreign_batch,
                        ).unwrap()
                    } else {

                        selected.retain_selection(&issued).unwrap().admit_application_query(
                        &phase, &entry_query, &access, ApplicationQueryParameterSet::new(), crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new((&entry_query).read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), local_scope),
                    ).unwrap()
                    };
                    if failure && case == Case::Cancelled { cancellation.cancel(); }
                    let result = read.execute(plan)?;
                    read_work.set(read_work.get() + result.receipt().work().total_work_units());
                    Ok(())
                },
                |row| row.label().to_owned(),
                |row| if failure && case == Case::WrongLink { "wrong".into() } else { row.status().to_owned() },
                |_, second| {
                    projections.set(projections.get() + 1);

                    if failure && case == Case::AfterProjection { cancellation.cancel(); }
                    if failure && case == Case::StaleProduct {
                        let graph = world.application.runtime.primary_graph().unwrap();
                        let field = AccountLabel::reference();
                        let locator = graph.layout.field_locator(field.entity(), field.aspect(), field.field()).unwrap().clone();
                        let prepared = graph.managed_derived_views().prepare_publication(
                            publication_basis(selected.product()),
                            &[ViewChange::Aspect(root.entity_id(), locator.aspect().aspect_key().clone())], 100_000,
                        );
                        request.in_scope(|lease| {
                        let active = ExecutionRequest::leased(lease.expect("projection lends the active host descendant"));
                        crate::domain_computation::primary_graph::tests::fixture::publish_relational_mutation(
                            &active, &world, WorkerIntentBatch::new("dependent-during-projection").push(
                                MutationIntent::Entity(EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                                    entity_id: root.entity_id(), fields: AspectFieldPatch::from(BTreeMap::from([(
                                        locator, StringApplicationValueBinding::encode(&"changed-link".to_owned()).unwrap(),
                                    )])),
                                })),
                            ),
                        );
                        }).unwrap();
                        let fresh = world.selected_product(&host);
                        assert_ne!(fresh.product().selected_commit(), selected.product().selected_commit());
                        prepared.apply(fresh.product().selected_commit());
                    }
                    SceneLabel(second.label().to_owned())
                },
            );

            if failure && matches!(case, Case::Deadline | Case::Memory | Case::Work) {
                assert_eq!(second_calls.get(), second_before);
            }
            if failure && case == Case::ForeignBasis {
            }
            if result.is_ok() {
            }
            result
        };
        let keys = run(false).unwrap();
        assert_eq!(keys.len(), 1);
        let key = &keys[0];
        let snapshot = world.application.observe_managed_derived_view(&view).unwrap();
        assert_eq!(snapshot.selected_commit(), selected.product().selected_commit());
        let original = snapshot.get(key).unwrap().unwrap();
        assert_eq!(original.0, "secondary");
        assert_eq!(key.root(), root.entity_id());
        if !matches!(case, Case::Success | Case::LeasedSuccess) {
            let projected_before = projections.get();
            let result = run(true);
            let denial = result.unwrap_err();
            match case {
                Case::WrongScope | Case::WrongLink | Case::ForeignRequest | Case::ForeignBasis => assert_eq!(denial, Denial::IncompleteDependencies),
                Case::RepeatedRead => assert_eq!(denial, Denial::QueryExecutionDenied),
                Case::StaleProduct => assert_eq!(denial, Denial::StaleSource),
                Case::Deadline => assert_eq!(denial, Denial::DeadlineElapsed),
                Case::Memory => assert!(matches!(denial, Denial::PolicyMemoryExhausted(_))),
                Case::AfterProjection => assert_eq!(denial, Denial::Cancelled),
                Case::ForeignBatch => assert!(matches!(denial, Denial::BatchResource {
                    denial: crate::facade::primary_graph::WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan, .. })),
                Case::Cancelled => assert!(matches!(denial, Denial::Cancelled | Denial::ReadDenied { .. })),
                Case::Work => assert!(matches!(denial, Denial::WorkExhausted { .. })),
                Case::Success | Case::LeasedSuccess => unreachable!(),
            }
            assert_eq!(projections.get(), projected_before + usize::from(matches!(case, Case::AfterProjection | Case::StaleProduct)));
            let after = world.application.observe_managed_derived_view(&view).unwrap();
            if case == Case::StaleProduct {
                assert_eq!(snapshot.get(key).err(), Some(Denial::StaleSource));
                assert_eq!(after.get(key).err(), Some(Denial::EntryRefreshRequired));
            } else {
                assert!(std::sync::Arc::ptr_eq(&original, &after.get(key).unwrap().unwrap()));
            }
        }
    }).unwrap();
}
