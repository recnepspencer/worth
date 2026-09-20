use std::cmp::Ordering;

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::registry::{
    source_custody::root_revision_order, BoundOutputSource, PreparedOutputRootKind, SourceCustody,
};

fn real_scope_and_occurrence() -> (
    crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    worth_runtime_world::facade::ProductBranchIncarnation,
) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let account =
        crate::domain_computation::primary_graph::tests::application_attempt::resolved_account(
            &world, "open", &request,
        );
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(account.entity_id());
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture product occurrence is live");
    (scope, product.observation().lifecycle_incarnation())
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
        root_kind: PreparedOutputRootKind::Discovered(std::any::TypeId::of::<()>()),
        source: None,
        discovery: None,
        bound_sources: Some(vec![older.clone(), sibling.clone()]),
        consumed_sources: Vec::new(),
        retired_sources: Vec::new(),
        retired: None,
        token_count: 0,
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
fn completed_multi_root_custody_waits_for_token_and_active_record() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let occurrence = receipt.product_branch().occurrence();
    let (scope, _) = real_scope_and_occurrence();
    let first = BoundOutputSource {
        scope,
        identity: key("producer", 5, 1).source,
    };
    let second = BoundOutputSource {
        scope,
        identity: key("producer", 5, 2).source,
    };
    let mut state = DemandRegistryState::default();
    state.source_custody.insert(
        commit.clone(),
        SourceCustody {
            occurrence,
            root_kind: PreparedOutputRootKind::Discovered(std::any::TypeId::of::<()>()),
            source: None,
            discovery: None,
            bound_sources: Some(vec![first.clone(), second.clone()]),
            consumed_sources: vec![first.identity.clone(), second.identity.clone()],
            retired_sources: Vec::new(),
            retired: None,
            token_count: 1,
            completed: true,
        },
    );
    state.prune_completed_custody();
    assert_eq!(state.source_custody.len(), 1);
    state.source_custody.get_mut(&commit).unwrap().token_count = 0;
    let demand_key = key("producer", 5, 1);
    let mut active = record(occurrence, DemandState::Admitted, 1);
    active.source_commits.push(commit.clone());
    state.records.insert(demand_key.clone(), active);
    state.prune_completed_custody();
    assert_eq!(state.source_custody.len(), 1);
    let record = state.records.get_mut(&demand_key).unwrap();
    record.interests = 0;
    record.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::NoEffect,
        "terminal source",
    ));
    state.prune_completed_custody();
    assert!(state.source_custody.is_empty());
}

#[test]
fn fully_superseded_multi_root_custody_prunes_after_its_last_token() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let (scope, _) = real_scope_and_occurrence();
    let first = BoundOutputSource {
        scope,
        identity: key("producer", 5, 1).source,
    };
    let second = BoundOutputSource {
        scope,
        identity: key("producer", 5, 2).source,
    };
    let superseded = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "newer source owns this root",
    );
    let mut state = DemandRegistryState::default();
    state.source_custody.insert(
        commit.clone(),
        SourceCustody {
            occurrence: receipt.product_branch().occurrence(),
            root_kind: PreparedOutputRootKind::Discovered(std::any::TypeId::of::<()>()),
            source: None,
            discovery: None,
            bound_sources: Some(vec![first.clone(), second.clone()]),
            consumed_sources: Vec::new(),
            retired_sources: vec![
                (first.identity.clone(), superseded.clone()),
                (second.identity.clone(), superseded),
            ],
            retired: None,
            token_count: 1,
            completed: false,
        },
    );
    state.prune_completed_custody();
    assert_eq!(state.source_custody.len(), 1);
    state.source_custody.get_mut(&commit).unwrap().token_count = 0;
    state.prune_completed_custody();
    assert!(state.source_custody.is_empty());
}
