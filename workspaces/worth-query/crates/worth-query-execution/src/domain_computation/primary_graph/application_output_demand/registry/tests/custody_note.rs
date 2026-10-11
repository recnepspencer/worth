//! A held custody note survives rotation until the last stale claim leaves.
use super::super::{BoundOutputSource, PreparedOutputRootKind, SourceCustody};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::equality_type_id::EqualityTypeId;

fn fixture(
    receipt: &WorthQueryApplicationCommitReceipt,
    ready: &DemandState,
    seed: u64,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("upstream", seed + 1, 1, 10);
    let newest = key_with_identity("upstream", seed + 2, 1, 11);
    let dependent = key("dependent", seed + 1, 2);
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let mut state = registry.state.lock().unwrap();
    let mut next = record(receipt.product_branch().occurrence(), copy_ready(ready), 1);
    next.source_commits.push(commit.clone());
    let next_wake = Arc::clone(&next.wake);
    state.records.insert(newest.clone(), next);
    state.records.insert(
        old.clone(),
        DemandRecord {
            framework_required_count: 1,
            ..record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "newer row",
                )),
                0,
            )
        },
    );
    let row = DemandRecord {
        prerequisites: vec![Arc::new(old)],
        ..record(occurrence(), copy_ready(ready), 1)
    };
    let mut row = row;
    row.required_interests = 1;
    state.required_reserved_bytes +=
        row.prerequisites.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
    let wake = Arc::clone(&row.wake);
    state.records.insert(dependent.clone(), row);
    let member = state.prepare_required_member(&newest).unwrap();
    state.install_required_member(member);
    state.source_custody.insert(
        commit,
        SourceCustody {
            occurrence: receipt.product_branch().occurrence(),
            root_kind: PreparedOutputRootKind::Discovered(EqualityTypeId::of::<()>()),
            source: None,
            discovery: None,
            bound_sources: Some(vec![BoundOutputSource {
                scope,
                identity: newest.source.clone(),
            }]),
            consumed_sources: vec![newest.source.clone()],
            retired_sources: Vec::new(),
            retired: None,
            token_count: 0,
            cleanup: Default::default(),
            completed: true,
        },
    );
    drop(state);
    registry.complete_prepared_source(receipt);
    let next_owner = interest(&registry, newest.clone(), next_wake);
    let dependent_owner = required_interest(&registry, dependent, wake);
    (registry, next_owner, dependent_owner, newest)
}

#[test]
fn registry_custody_note_survives_a_held_drain_and_claim_transfer() {
    let receipt = walk_bounds::receipt_batch().remove(0);
    let ready = closed_retirement::ready();
    let (registry, owner, dependent, newest) = fixture(
        &receipt,
        &ready,
        0,
        source_custody::real_scope_and_occurrence().0,
    );
    let commit = receipt.committed_product_publication().composite_commit();
    drop(owner);
    registry
        .state
        .lock()
        .unwrap()
        .drain_foreign_custody_quota(&mut record_admission());
    {
        let state = registry.state.lock().unwrap();
        assert!(
            state.source_custody.contains_key(commit),
            "stale claim still holds custody"
        );
        assert!(
            state.source_custody[commit].cleanup.queued,
            "held debt stays noted"
        );
    }
    drop(dependent);
    let mut state = registry.state.lock().unwrap();
    assert_eq!(state.records[&newest].framework_required_count, 1);
    state.drain_foreign_custody_quota(&mut record_admission());
    assert!(!state.source_custody.contains_key(commit));
    super::super::post_commit::custody_reader::resolve_before_binding(&mut state, &receipt);
    assert!(
        !state.pending_custody_prune(),
        "binding cannot see the pruned newer source"
    );
}

fn copy_ready(ready: &DemandState) -> DemandState {
    let DemandState::Output(output) = ready else {
        panic!("Ready fixture")
    };
    let Some(super::super::WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint
    else {
        panic!("Ready fixture")
    };
    DemandState::Output(super::super::WorthQueryOutputProgress::new(
        super::super::WorthQueryOutputCheckpoint::Ready(completion.clone()),
    ))
}

mod differential;
