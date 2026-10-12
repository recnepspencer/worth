use crate::equality_type_id::EqualityTypeId as TypeId;
use std::cmp::Ordering;

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::registry::{
    source_custody::root_revision_order, BoundOutputSource, PreparedOutputRootKind, SourceCustody,
};

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn real_scope_and_occurrence(
) -> (
    crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    worth_runtime_world::facade::ProductBranchIncarnation,
) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request();
        let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
        let account =
            crate::domain_computation::primary_graph::tests::application_attempt::resolved_account(
                &custody,
                &world, "open", &request,
            );
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(account.entity_id());
        let product = world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .expect("the fixture product occurrence is live");
        (scope, product.observation().lifecycle_incarnation())
    }).expect("the fixture source read owns its host call")
}

#[test]
fn newer_root_retires_only_its_matching_source() {
    let (scope, occurrence) = real_scope_and_occurrence();
    let older = BoundOutputSource {
        scope,
        identity: key("producer", 5, 1).source,
    };
    let sibling = BoundOutputSource {
        scope,
        identity: key("producer", 5, 2).source,
    };
    let newer = BoundOutputSource {
        scope,
        identity: key("producer", 6, 1).source,
    };
    assert_eq!(root_revision_order(&older, &newer), Some(Ordering::Less));
    assert_eq!(
        root_revision_order(&sibling, &sibling),
        Some(Ordering::Equal)
    );
    assert_eq!(root_revision_order(&older, &sibling), None);
    let mut custody = SourceCustody {
        occurrence,
        root_kind: PreparedOutputRootKind::Discovered(TypeId::of::<()>()),
        source: None,
        discovery: None,
        bound_sources: Some(vec![older.clone(), sibling.clone()]),
        consumed_sources: Vec::new(),
        retired_sources: Vec::new(),
        retired: None,
        token_count: 0,
        cleanup: Default::default(),
        completed: false,
    };
    custody.retire_superseded_roots(&[newer, sibling.clone()]);
    assert_eq!(
        custody.source_denial(&older.identity).unwrap().kind(),
        WorthQueryOutputDemandDenialKind::Superseded
    );
    assert!(custody.source_denial(&sibling.identity).is_none());
    assert!(
        custody.retired.is_none(),
        "one stale root cannot retire its sibling custody"
    );
}

#[test]
fn registry_custody_waits_for_its_token_without_an_active_record() {
    for superseded in [false, true] {
        custody_wait(superseded, true);
    }
}

#[test]
fn registry_custody_waits_for_its_active_record_without_a_token() {
    for superseded in [false, true] {
        custody_wait(superseded, false);
    }
}

fn custody_wait(superseded: bool, token: bool) {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support
        ::committed_recoverable_application();
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
    registry
        .state
        .lock()
        .unwrap()
        .drain_completed_custody(&mut record_admission())
        .unwrap();
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
    registry
        .state
        .lock()
        .unwrap()
        .drain_completed_custody(&mut record_admission())
        .unwrap();
    let state = registry.state.lock().unwrap();
    assert!(state.source_custody.is_empty());
    assert!(state
        .records
        .values()
        .all(|row| row.source_commits.is_empty()));
    assert!(!state.pending_custody_prune());
}

mod retained_proofs;
