use super::*;
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
            cleanup: Default::default(),
            occurrence,
            root_kind: PreparedOutputRootKind::Discovered(TypeId::of::<()>()),
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
    state.note_custody(&commit);
    state
        .prune_completed_custody(&mut super::super::super::lifecycle::release::CleanupWalk::new())
        .unwrap();
    assert_eq!(state.source_custody.len(), 1);
    state.source_custody.get_mut(&commit).unwrap().token_count = 0;
    let demand_key = key("producer", 5, 1);
    let mut active = record(occurrence, DemandState::Admitted, 1);
    active.source_commits.push(commit.clone());
    state.records.insert(demand_key.clone(), active);
    state.note_custody(&commit);
    state
        .prune_completed_custody(&mut super::super::super::lifecycle::release::CleanupWalk::new())
        .unwrap();
    assert_eq!(state.source_custody.len(), 1);
    let mut record = state.records.get_mut(&demand_key).unwrap();
    record.interests = 0;
    record.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::NoEffect,
        "terminal source",
    ));
    record.finish();
    state.note_custody(&commit);
    state
        .prune_completed_custody(&mut super::super::super::lifecycle::release::CleanupWalk::new())
        .unwrap();
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
            cleanup: Default::default(),
            occurrence: receipt.product_branch().occurrence(),
            root_kind: PreparedOutputRootKind::Discovered(TypeId::of::<()>()),
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
    state.note_custody(&commit);
    state
        .prune_completed_custody(&mut super::super::super::lifecycle::release::CleanupWalk::new())
        .unwrap();
    assert_eq!(state.source_custody.len(), 1);
    state.source_custody.get_mut(&commit).unwrap().token_count = 0;
    state.note_custody(&commit);
    state
        .prune_completed_custody(&mut super::super::super::lifecycle::release::CleanupWalk::new())
        .unwrap();
    assert!(state.source_custody.is_empty());
}
