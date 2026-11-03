use crate::domain_computation::primary_graph::application_output_demand::registry::tests::demand_key;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::interest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
use std::sync::Arc;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;

use worth_query_execution::equality_type_id::EqualityTypeId as TypeId;
use std::cmp::Ordering;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::source_custody::root_revision_order;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
pub fn real_scope_and_occurrence() -> (
    worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding,
    worth_runtime_world::facade::ProductBranchIncarnation,
) {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request();
        let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let account =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(
                &custody,
                &world, "open", &request,
            );
        let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(account.entity_id());
        let product = world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .expect("the fixture product occurrence is live");
        (scope, product.observation().lifecycle_incarnation())
    }).expect("the fixture source read owns its host call")
}
pub fn custody_wait(superseded: bool, token: bool) {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let (scope, _) = real_scope_and_occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let selected = key("producer", 5, 1);
    let sources = vec![
        BoundOutputSource {
            scope,
            identity: selected.source.clone(),
        },
        BoundOutputSource {
            scope,
            identity: key("producer", 5, 2).source,
        },
    ];
    {
        let mut state = registry.state.lock().unwrap();
        state.source_custody.insert(
            commit.clone(),
            SourceCustody {
                occurrence: receipt.product_branch().occurrence(),
                root_kind: PreparedOutputRootKind::Discovered(TypeId::of::<()>()),
                source: None,
                discovery: None,
                consumed_sources: sources
                    .iter()
                    .map(|source| source.identity.clone())
                    .collect(),
                retired_sources: if superseded {
                    sources
                        .iter()
                        .map(|source| {
                            (
                                source.identity.clone(),
                                WorthQueryOutputDemandDenial::new(
                                    WorthQueryOutputDemandDenialKind::Superseded,
                                    "newer source",
                                ),
                            )
                        })
                        .collect()
                } else {
                    Vec::new()
                },
                bound_sources: Some(sources),
                retired: None,
                token_count: usize::from(token),
                cleanup: Default::default(),
                completed: !superseded,
            },
        );
        let mut active = record(
            receipt.product_branch().occurrence(),
            DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::NoEffect,
                "terminal source",
            )),
            usize::from(!token),
        );
        active.source_commits.push(commit.clone());
        state.records.insert(selected.clone(), active);
    }
    registry.complete_prepared_source(&receipt);
    registry.state.lock().unwrap().drain_completed_custody();
    {
        let state = registry.state.lock().unwrap();
        assert_eq!(state.source_custody.len(), 1);
        assert!(state.source_custody[&commit].cleanup.queued);
    }
    if token {
        registry.release_prepared_token(&commit);
    } else {
        let notifications = Arc::clone(&registry.state.lock().unwrap().records[&selected].wake);
        drop(interest(&registry, selected.clone(), notifications));
    }
    registry.state.lock().unwrap().drain_completed_custody();
    let state = registry.state.lock().unwrap();
    assert!(state.source_custody.is_empty());
    assert!(state
        .records
        .values()
        .all(|row| row.source_commits.is_empty()));
    assert!(!state.pending_custody_prune());
}
