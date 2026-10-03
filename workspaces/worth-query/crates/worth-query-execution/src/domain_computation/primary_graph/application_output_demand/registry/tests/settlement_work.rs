use std::sync::Arc;

use super::*;
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, RecordedSettlementIdentity,
};
use worth_relational::facade::mvcc::CompanionPreflightBudget;

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 8 * 1024 * 1024,
    })
}

fn populated_index(
    identity: &Arc<RecordedSettlementIdentity>,
) -> (DemandRegistryState, Arc<WorthQueryOutputDemandKey>) {
    let mut state = DemandRegistryState::default();
    let upstream = Arc::new(key("historical-upstream", 1, 1));
    let (posting, _unused_cancellation) = state
        .reserve_settlement_vacancy(identity, &mut admission(1_000_000))
        .expect("real lineage identity reserves one exact address");
    super::super::settlement_index::SettlementIndex::fill_prepared(
        &posting,
        identity,
        Arc::clone(identity),
        Arc::clone(&upstream),
    );
    (state, upstream)
}

#[test]
fn nonempty_exact_settlement_lookup_denies_before_pinning_and_retries() {
    let (_lineage, identity) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let (state, upstream) = populated_index(&identity);
    let mut measured = admission(1_000_000);
    let selected = state
        .settlement_keys
        .get_exact_admitted(&identity, &mut measured)
        .unwrap()
        .expect("the real restored settlement is indexed");
    assert!(Arc::ptr_eq(&selected, &upstream));
    drop(selected);
    let full_work = measured.charged_work();
    assert!(
        measured.charged_navigation() > 0,
        "both nonempty tree paths are reported as physical navigation"
    );
    assert!(full_work > 1, "both nonempty tree paths must be visited");
    let pins_before = Arc::strong_count(&upstream);
    let mut short = admission(full_work - 1);
    let denial = state
        .settlement_keys
        .get_exact_admitted(&identity, &mut short)
        .err()
        .expect("one short work unit denies before selecting the upstream key");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(Arc::strong_count(&upstream), pins_before);
    assert!(Arc::ptr_eq(
        &state
            .settlement_keys
            .get_exact_admitted(&identity, &mut admission(full_work))
            .unwrap()
            .unwrap(),
        &upstream,
    ));
}

fn queued_terminal(
    identity: &Arc<RecordedSettlementIdentity>,
) -> (DemandRegistryState, Arc<WorthQueryOutputDemandKey>) {
    let (mut state, _upstream) = populated_index(identity);
    let downstream = Arc::new(key("terminal-dependent", 2, 2));
    let mut row = record(
        identity.address().0,
        DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::Closed,
            "retired exact historical output",
        )),
        0,
    );
    row.settlements.push((Arc::clone(identity), 0));
    state.required_reserved_bytes += row.settlements.capacity()
        * std::mem::size_of::<(Arc<RecordedSettlementIdentity>, usize)>();
    state.records.insert(downstream.as_ref().clone(), row);
    assert!(state.defer_terminal_cleanup(&downstream, 0));
    (state, downstream)
}

#[test]
fn queued_historical_settlement_cleanup_denies_before_unlink_and_retries() {
    let (_lineage, identity) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let (mut completed, _) = queued_terminal(&identity);
    let mut measured = admission(1_000_000);
    completed
        .drain_terminal_cleanup(&mut measured)
        .expect("admitted cleanup removes the selected historical posting");
    let full_work = measured.charged_work();
    assert!(full_work > 1);
    assert!(completed.pending_cleanup_head.is_none());
    assert_eq!(completed.required_reserved_bytes, 0);

    let (mut retry, downstream) = queued_terminal(&identity);
    let bytes_before = retry.required_reserved_bytes;
    let mut short = admission(full_work - 1);
    let denial = retry
        .drain_terminal_cleanup(&mut short)
        .err()
        .expect("one short work unit preserves the queued row and exact posting");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert!(retry.pending_cleanup_head.is_some());
    assert!(retry.records.contains_key(downstream.as_ref()));
    assert_eq!(retry.required_reserved_bytes, bytes_before);
    assert!(retry
        .settlement_keys
        .get_exact_admitted(&identity, &mut admission(1_000_000))
        .unwrap()
        .is_some());

    retry
        .drain_terminal_cleanup(&mut admission(full_work))
        .expect("the same selected cleanup succeeds with enough work");
    assert!(retry.pending_cleanup_head.is_none());
    assert!(retry
        .settlement_keys
        .get_exact_admitted(&identity, &mut admission(1_000_000))
        .unwrap()
        .is_none());
    assert_eq!(retry.required_reserved_bytes, 0);
}

fn queued_obligations_only(
    commit: &worth_runtime_world::facade::CompositeCommitIdentity,
) -> (DemandRegistryState, Arc<WorthQueryOutputDemandKey>) {
    let mut state = DemandRegistryState::default();
    let key = Arc::new(key("terminal-obligations", 3, 3));
    let mut row = record(
        occurrence(),
        DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::Closed,
            "",
        )),
        0,
    );
    row.performed_obligations.reserve_exact(3);
    for _ in 0..2 {
        row.performed_obligations
            .push(super::super::PerformedOutputObligation {
                source_commit: commit.clone(),
                source: key.source.clone(),
            });
    }
    assert!(row.performed_obligations.capacity() > row.performed_obligations.len());
    state.obligation_reserved_bytes = row.obligation_reserved_bytes();
    state.records.insert(key.as_ref().clone(), row);
    let member = state.prepare_required_member(key.as_ref()).unwrap();
    state.install_required_member(member);
    assert!(state.defer_terminal_cleanup(&key, 0));
    (state, key)
}

#[test]
fn obligations_only_terminal_cleanup_preserves_custody_on_denial_and_refunds_on_retry() {
    // This is registry lifecycle evidence over a real performed commit, not
    // an output-currentness oracle. No synthetic Ready authority is involved.
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt.committed_product_publication().composite_commit();
    let (mut completed, key) = queued_obligations_only(commit);
    let mut measured = admission(1_000_000);
    completed.drain_terminal_cleanup(&mut measured).unwrap();
    assert!(!completed.records.contains_key(key.as_ref()));
    assert!(completed.required_keys.is_empty());
    assert_eq!(completed.obligation_reserved_bytes, 0);
    assert_eq!(completed.required_reserved_bytes, 0);

    let (mut retry, key) = queued_obligations_only(commit);
    let obligation_bytes = retry.obligation_reserved_bytes;
    let required_bytes = retry.required_reserved_bytes;
    let denial = retry
        .drain_terminal_cleanup(&mut admission(measured.charged_work() - 1))
        .expect_err("one short unit must preserve the entire queued obligation row");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(retry.records[key.as_ref()].performed_obligations.len(), 2);
    assert!(retry.required_keys.contains(key.as_ref()));
    assert!(retry.pending_cleanup_head.is_some());
    assert_eq!(retry.obligation_reserved_bytes, obligation_bytes);
    assert_eq!(retry.required_reserved_bytes, required_bytes);

    retry
        .drain_terminal_cleanup(&mut admission(measured.charged_work()))
        .unwrap();
    assert!(!retry.records.contains_key(key.as_ref()));
    assert!(retry.required_keys.is_empty());
    assert!(retry.pending_cleanup_head.is_none());
    assert_eq!(retry.obligation_reserved_bytes, 0);
    assert_eq!(retry.required_reserved_bytes, 0);
}

#[test]
fn last_interest_drop_preserves_queued_obligations_for_admitted_cleanup() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let (mut state, key) =
        queued_obligations_only(receipt.committed_product_publication().composite_commit());
    let row = state.records.get_mut(key.as_ref()).unwrap();
    row.interests = 1;
    let wake = Arc::clone(&row.wake);
    let obligation_bytes = state.obligation_reserved_bytes;
    let required_bytes = state.required_reserved_bytes;
    let registry = WorthQueryOutputDemandRegistry::default();
    *registry.state.lock().unwrap() = state;
    let closing = interest(&registry, key.as_ref().clone(), wake);
    drop(closing);
    let state = registry.state.lock().unwrap();
    let row = &state.records[key.as_ref()];
    assert_eq!(row.interests, 0);
    assert_eq!(row.performed_obligations.len(), 2);
    assert!(row.pending_cleanup_queued);
    assert!(state.required_keys.contains(key.as_ref()));
    assert_eq!(state.obligation_reserved_bytes, obligation_bytes);
    assert_eq!(state.required_reserved_bytes, required_bytes);
    drop(state);
    registry
        .drain_terminal_cleanup_admitted(&mut admission(1_000_000))
        .unwrap();
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(key.as_ref()));
    assert!(state.required_keys.is_empty());
    assert_eq!(state.obligation_reserved_bytes, 0);
    assert_eq!(state.required_reserved_bytes, 0);
}
