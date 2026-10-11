//! Drop notes completed custody; the next admitted drain owns its sweep.
use super::super::lifecycle::completed_custody::contacts;
use super::*;

fn fixture(
    token: bool,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandKey,
    Vec<crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt>,
) {
    let receipts = walk_bounds::receipt_batch();
    let registry = WorthQueryOutputDemandRegistry::default();
    let selected = key("custody-unrelated-0", 1, 100);
    let ready = closed_retirement::ready();
    let mut state = registry.state.lock().unwrap();
    walk_bounds::add_live_custody(&mut state, &receipts, 1);
    for row in state.records.values_mut() {
        row.interests = 0;
        row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::NoEffect,
            "completed",
        ));
    }
    for (index, receipt) in receipts.iter().enumerate() {
        let commit = receipt.committed_product_publication().composite_commit();
        state.source_custody.get_mut(commit).unwrap().token_count =
            usize::from(token || index != 0);
    }
    if !token {
        let row = state.records.get_mut(&selected).unwrap();
        row.state = ready;
        row.interests = 1;
    }
    assert!(!state.pending_custody_prune());
    state
        .source_custody
        .get_mut(
            receipts[0]
                .committed_product_publication()
                .composite_commit(),
        )
        .unwrap()
        .completed = true;
    drop(state);
    registry.complete_prepared_source(&receipts[0]);
    (registry, selected, receipts)
}

#[test]
fn registry_interest_drop_leaves_custody_note_without_sweeping() {
    let (registry, selected, receipts) = fixture(false);
    let wake = Arc::clone(&registry.state.lock().unwrap().records[&selected].wake);
    let owner = interest(&registry, selected, wake);
    let before = contacts();
    drop(owner);
    assert_eq!(
        contacts(),
        before,
        "Drop does not inspect any custody entry"
    );
    let commit = receipts[0]
        .committed_product_publication()
        .composite_commit();
    let mut state = registry.state.lock().unwrap();
    assert!(state.pending_custody_prune());
    assert!(state.source_custody[commit].cleanup.queued);
    state
        .drain_completed_custody(&mut record_admission())
        .unwrap();
    assert!(!state.source_custody.contains_key(commit));
    assert!(!state.pending_custody_prune());
    assert_eq!(state.required_reserved_bytes, 0);
    assert_eq!(state.obligation_reserved_bytes, 0);
    assert_eq!(
        state
            .required_custody_retained_bytes
            .load(std::sync::atomic::Ordering::Acquire),
        0
    );
}

#[test]
fn registry_prepared_token_drop_leaves_custody_note_without_sweeping() {
    let (registry, _, receipts) = fixture(true);
    let commit = receipts[0]
        .committed_product_publication()
        .composite_commit();
    let before = contacts();
    registry.release_prepared_token(commit);
    assert_eq!(contacts(), before, "token Drop does not inspect custody");
    let mut state = registry.state.lock().unwrap();
    assert_eq!(state.source_custody[commit].token_count, 0);
    assert!(state.source_custody[commit].cleanup.queued);
    state
        .drain_completed_custody(&mut record_admission())
        .unwrap();
    assert!(!state.source_custody.contains_key(commit));
    assert!(!state.pending_custody_prune());
    assert_eq!(state.required_reserved_bytes, 0);
    assert_eq!(state.obligation_reserved_bytes, 0);
    assert_eq!(
        state
            .required_custody_retained_bytes
            .load(std::sync::atomic::Ordering::Acquire),
        0
    );
}
