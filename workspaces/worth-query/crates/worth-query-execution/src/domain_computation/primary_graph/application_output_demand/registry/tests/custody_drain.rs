//! Noted custody makes bounded, durable progress without starving its reader.
use super::super::{BoundOutputSource, DemandAdmissionKind, SourceEpoch};
use super::*;
mod rotation;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_runtime_world::facade::CompositeCommitIdentity;

const ENTRY_VISIT: u64 = 1;
const OCCURRENCE_SEEKS: u64 = 2;
const ROW_INSPECTION: u64 = 1;
const ENTRY_PRICE: u64 = 2 * ENTRY_VISIT + 7 * (OCCURRENCE_SEEKS + ROW_INSPECTION);
const QUOTA: usize = super::super::cleanup::custody::FOREIGN_CUSTODY_DRAIN_QUOTA;

thread_local! {
    static SCOPE: (
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        worth_runtime_world::facade::ProductBranchIncarnation,
    ) = source_custody::real_scope_and_occurrence();
}
fn fixture_scope() -> (
    crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    worth_runtime_world::facade::ProductBranchIncarnation,
) {
    SCOPE.with(|scope| *scope)
}

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(work),
        maximum_preparation_bytes: None,
    })
}

fn fixture(
    receipts: &[WorthQueryApplicationCommitReceipt],
    rows: u64,
) -> (WorthQueryOutputDemandRegistry, WorthQueryOutputDemandKey) {
    let (registry, own) = live_fixture(receipts, rows);
    for receipt in receipts {
        registry
            .state
            .lock()
            .unwrap()
            .source_custody
            .get_mut(receipt.committed_product_publication().composite_commit())
            .unwrap()
            .completed = true;
        registry.complete_prepared_source(receipt);
    }
    (registry, own)
}

fn live_fixture(
    receipts: &[WorthQueryApplicationCommitReceipt],
    rows: u64,
) -> (WorthQueryOutputDemandRegistry, WorthQueryOutputDemandKey) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let own = key("reader", 1, 250);
    {
        let mut state = registry.state.lock().unwrap();
        walk_bounds::add_custody_with_completion(
            &mut state,
            receipts,
            rows,
            fixture_scope().0,
            false,
        );
        for mut row in state.records.values_mut() {
            row.interests = 0;
            row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::NoEffect,
                "completed",
            ));
        }
        state
            .records
            .insert(own.clone(), record(occurrence(), DemandState::Admitted, 0));
    }
    (registry, own)
}

fn admit(
    registry: &WorthQueryOutputDemandRegistry,
    key: &WorthQueryOutputDemandKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial> {
    let (scope, occurrence) = fixture_scope();
    registry
        .admit(
            key.clone(),
            None,
            scope,
            occurrence,
            DemandAdmissionKind::Ordinary,
            None,
            None,
            admission,
        )
        .map(|(interest, _pause)| interest)
}

#[test]
fn registry_foreign_custody_debt_cannot_starve_admission() {
    let receipts = walk_bounds::receipt_batch();
    let (eager, own) = fixture(&receipts, 128);
    eager_cleanup(&mut eager.state.lock().unwrap());
    let mut own_work = admission(u64::MAX);
    let mut eager_interests = vec![admit(&eager, &own, &mut own_work).unwrap()];
    let entry_price =
        2 * ENTRY_VISIT + 3 * (OCCURRENCE_SEEKS + 128) + 4 * 128 * (OCCURRENCE_SEEKS + 128);
    let ceiling =
        own_work.charged_work() + OCCURRENCE_SEEKS + ROW_INSPECTION + QUOTA as u64 * entry_price;
    assert!(
        receipts.len() as u64 * entry_price > ceiling,
        "own price {}",
        own_work.charged_work()
    );
    let (deferred, own) = fixture(&receipts, 128);
    let mut interests = Vec::new();
    for step in 1..=receipts.len().div_ceil(QUOTA) {
        interests.push(
            admit(&deferred, &own, &mut admission(ceiling))
                .expect("foreign debt cannot refuse work that covers the fixed quota"),
        );
        let state = deferred.state.lock().unwrap();
        assert_eq!(
            state.source_custody.len(),
            receipts.len().saturating_sub(step * QUOTA)
        );
        assert_eq!(
            state
                .records
                .values()
                .filter(|row| row.source_commits.is_empty())
                .count(),
            1 + 128 * (step * QUOTA).min(receipts.len())
        );
    }
    for _ in 1..interests.len() {
        eager_interests.push(admit(&eager, &own, &mut admission(u64::MAX)).unwrap());
    }
    assert_eq!(
        value(&deferred.state.lock().unwrap()),
        value(&eager.state.lock().unwrap())
    );
    assert!(!deferred.state.lock().unwrap().pending_custody_prune());
}

#[test]
fn registry_foreign_custody_pause_preserves_the_admission_own_budget() {
    let receipts = walk_bounds::receipt_batch();
    let (eager, own) = fixture(&receipts, 1);
    eager_cleanup(&mut eager.state.lock().unwrap());
    let mut full = admission(u64::MAX);
    let _eager_interest = admit(&eager, &own, &mut full).unwrap();
    let (deferred, own) = fixture(&receipts, 1);
    let ceiling = full.charged_work();
    let _interest = admit(&deferred, &own, &mut admission(ceiling))
        .expect("a foreign maintenance pause cannot consume the own edit's budget");
    let state = deferred.state.lock().unwrap();
    assert_eq!(state.source_custody.len(), receipts.len());
    assert!(state.pending_custody_prune());
    assert_eq!(state.records[&own].interests, 1);
    assert_eq!(
        state.required_reserved_bytes,
        eager.state.lock().unwrap().required_reserved_bytes
    );
}

#[test]
fn registry_own_custody_is_drained_before_its_admission_edit() {
    let receipts = walk_bounds::receipt_batch();
    let (registry, _) = fixture(&receipts[..1], 1);
    let own = key("custody-unrelated-0", 1, 100);
    // Own occurrence: two seeks, its row and its source-commit probe; then
    // the entry quote/execution. No work remains for the admission's edit.
    let discovery = OCCURRENCE_SEEKS + ROW_INSPECTION + ENTRY_VISIT;
    let drain_work = discovery + OCCURRENCE_SEEKS + ROW_INSPECTION + ENTRY_VISIT + ENTRY_PRICE;
    let denial = match admit(&registry, &own, &mut admission(drain_work)) {
        Ok(_) => panic!("no budget remains for the admission's own edit"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    let state = registry.state.lock().unwrap();
    assert!(state.source_custody.is_empty());
    assert!(state.records[&own].source_commits.is_empty());
    assert_eq!(state.records[&own].interests, 0);
    assert!(state.required_keys.is_empty());
    drop(state);
    let _interest = admit(&registry, &own, &mut admission(u64::MAX)).unwrap();
}

#[test]
fn registry_mid_custody_drain_stop_keeps_completed_entries() {
    let receipts = walk_bounds::receipt_batch();
    let (registry, _) = fixture(&receipts, 1);
    let mut state = registry.state.lock().unwrap();
    let denial = state
        .drain_completed_custody(&mut admission(2 * ENTRY_PRICE + 1))
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(state.source_custody.len(), receipts.len() - 2);
    assert_eq!(
        state
            .records
            .values()
            .filter(|row| row.source_commits.is_empty())
            .count(),
        3
    );
    assert!(state.pending_custody_prune());
    state
        .drain_completed_custody(&mut admission(u64::MAX))
        .unwrap();
    let completed = value(&state);
    drop(state);
    let (eager, _) = fixture(&receipts, 1);
    eager_cleanup(&mut eager.state.lock().unwrap());
    assert_eq!(completed, value(&eager.state.lock().unwrap()));
}

#[test]
fn registry_entrywise_custody_drain_matches_eager_for_sixteen_seeds() {
    let receipts = walk_bounds::receipt_batch();
    for seed in 0..16 {
        let make = || fixture(&receipts[..1 + seed % 4], 1 + (seed % 3) as u64);
        let (eager, _) = make();
        eager_cleanup(&mut eager.state.lock().unwrap());
        let expected = value(&eager.state.lock().unwrap());
        let (measured, _) = make();
        let mut full = admission(u64::MAX);
        measured
            .state
            .lock()
            .unwrap()
            .drain_completed_custody(&mut full)
            .unwrap();
        for k in 0..full.charged_work() {
            let (retry, _) = make();
            let mut state = retry.state.lock().unwrap();
            let denial = state
                .drain_completed_custody(&mut admission(k))
                .unwrap_err();
            assert_eq!(
                denial.kind(),
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            );
            state
                .drain_completed_custody(&mut admission(u64::MAX))
                .unwrap();
            assert_eq!(value(&state), expected, "seed {seed}, k {k}");
        }
        assert_eq!(value(&measured.state.lock().unwrap()), expected);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DrainValue {
    rows: Vec<DrainRow>,
    members: Vec<WorthQueryOutputDemandKey>,
    custody: Vec<DrainCustody>,
    totals: [usize; 3],
    noted: bool,
}
#[derive(Debug, PartialEq, Eq)]
struct DrainRow {
    key: WorthQueryOutputDemandKey,
    denial: Option<WorthQueryOutputDemandDenialKind>,
    interests: [usize; 3],
    commits: Vec<CompositeCommitIdentity>,
}
#[derive(Debug, PartialEq, Eq)]
struct DrainCustody {
    commit: CompositeCommitIdentity,
    sources: Option<Vec<BoundOutputSource>>,
    consumed: Vec<SourceEpoch>,
    tokens: usize,
    completed: bool,
}
fn value(state: &DemandRegistryState) -> DrainValue {
    use std::sync::atomic::Ordering::Acquire;
    DrainValue {
        rows: state
            .records
            .iter()
            .map(|(key, row)| {
                assert!(row.prerequisites.is_empty() && row.performed_obligations.is_empty());
                assert!(row.settlements.is_empty());
                DrainRow {
                    key: key.clone(),
                    denial: match &row.state {
                        DemandState::Admitted => None,
                        DemandState::Failed(denial) => Some(denial.kind()),
                        _ => panic!("unexpected debt fixture state"),
                    },
                    interests: [
                        row.interests,
                        row.required_interests,
                        row.framework_required_count,
                    ],
                    commits: row.source_commits.clone(),
                }
            })
            .collect(),
        members: state
            .required_keys
            .iter()
            .map(|key| key.as_ref().clone())
            .collect(),
        custody: state
            .source_custody
            .iter()
            .map(|(commit, custody)| {
                assert!(custody.source.is_none() && custody.discovery.is_none());
                assert!(custody.retired.is_none() && custody.retired_sources.is_empty());
                DrainCustody {
                    commit: commit.clone(),
                    sources: custody.bound_sources.clone(),
                    consumed: custody.consumed_sources.clone(),
                    tokens: custody.token_count,
                    completed: custody.completed,
                }
            })
            .collect(),
        totals: [
            state.required_reserved_bytes,
            state.obligation_reserved_bytes,
            state.required_custody_retained_bytes.load(Acquire),
        ],
        noted: state.pending_custody_prune(),
    }
}

// Independent eager reference for these completed, consumed, unheld shapes.
// It directly states their final ownership; it does not call the drain body.
fn eager_cleanup(state: &mut DemandRegistryState) {
    assert!(state
        .source_custody
        .values()
        .all(|entry| entry.completed && entry.token_count == 0 && entry.source.is_none()));
    assert!(state.records.values().all(|row| row.interests == 0));
    state.source_custody.clear();
    state.custody_cleanup = Default::default();
    for mut row in state.records.values_mut() {
        row.source_commits.clear();
    }
}

#[test]
fn registry_custody_quote_prices_noted_entries_only() {
    let receipts = walk_bounds::receipt_batch();
    let (registry, _) = live_fixture(&receipts, 1);
    for receipt in &receipts[..2] {
        registry
            .state
            .lock()
            .unwrap()
            .source_custody
            .get_mut(receipt.committed_product_publication().composite_commit())
            .unwrap()
            .completed = true;
        registry.complete_prepared_source(receipt);
    }
    let mut state = registry.state.lock().unwrap();
    let mut paid = admission(u64::MAX);
    state.drain_completed_custody(&mut paid).unwrap();
    assert_eq!(
        paid.charged_work(),
        46,
        "two noted entries at 23: two entry visits and seven one-row walks"
    );
    assert_eq!(state.source_custody.len(), 2);
    assert!(!state.pending_custody_prune());
}
