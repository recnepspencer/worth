//! Canonical occurrence searches keep their producer/source boundary.
use super::super::refreshed_rejoin::{
    awaited_by_stale_owner, newest_of_occurrence, occurrence_rows, OccurrenceSearchCharge,
};
use super::*;
use crate::domain_computation::primary_graph::tests::recoverable_commit_support;
use core::convert::Infallible;

pub(in super::super) fn check<C>(search: usize, mut payer: impl FnMut() -> C)
where
    C: OccurrenceSearchCharge<Refusal = Infallible>,
{
    let mut contacts = Vec::new();
    let receipts = receipt_batch();
    for (neighbors, entries) in [(1, 1), (64, 4)] {
        let mut state = DemandRegistryState::default();
        let selected = key("own", 1, 1);
        for generation in 1..=3 {
            state.records.insert(
                key("own", generation, 1),
                record(occurrence(), DemandState::Admitted, 0),
            );
        }
        for generation in 1..=neighbors {
            state.records.insert(
                key("larger-other", generation, 2),
                record(occurrence(), DemandState::Admitted, 0),
            );
        }
        add_custody(&mut state, &receipts[..entries], neighbors);
        let before = super::super::record_map::occurrence_rows_visited();
        let mut charge = payer();
        match search {
            0 => assert_eq!(
                super::super::refreshed_rejoin::infallible(occurrence_rows(
                    &state.records,
                    &selected,
                    &mut charge
                ))
                .count(),
                3
            ),
            1 => assert_eq!(
                super::super::refreshed_rejoin::infallible(newest_of_occurrence(
                    &state.records,
                    &selected,
                    &mut charge
                )),
                Some(key("own", 3, 1))
            ),
            2 => assert!(!super::super::refreshed_rejoin::infallible(
                awaited_by_stale_owner(&state.records, &selected, &mut charge)
            )),
            _ => panic!("unknown occurrence search"),
        }
        contacts.push(super::super::record_map::occurrence_rows_visited() - before);
    }
    assert_eq!(contacts, vec![3, 3], "search {search}");
}

#[test]
fn registry_occurrence_rows_body_keeps_its_bound() {
    check(0, OracleWalk::after_effect);
}
#[test]
fn registry_newest_fold_body_keeps_its_bound() {
    check(1, OracleWalk::after_effect);
}
#[test]
fn registry_stale_owner_body_keeps_its_bound() {
    check(2, OracleWalk::after_effect);
}

use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
pub(in super::super) fn receipt_batch() -> Vec<WorthQueryApplicationCommitReceipt> {
    (0..4)
        .map(|_| recoverable_commit_support::committed_recoverable_application())
        .collect()
}
pub(in super::super) fn add_custody(
    state: &mut DemandRegistryState,
    receipts: &[WorthQueryApplicationCommitReceipt],
    rows: u64,
) {
    let (scope, _) = super::source_custody::real_scope_and_occurrence();
    add_custody_in_scope(state, receipts, rows, scope);
}
pub(in super::super) fn add_custody_in_scope(
    state: &mut DemandRegistryState,
    receipts: &[WorthQueryApplicationCommitReceipt],
    rows: u64,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
) {
    for (i, receipt) in receipts.iter().enumerate() {
        let commit = receipt
            .committed_product_publication()
            .composite_commit()
            .clone();
        let selected = key(
            &format!("custody-unrelated-{i}"),
            1,
            u8::try_from(i).unwrap() + 100,
        );
        for generation in 1..=rows {
            let mut row = record(
                receipt.product_branch().occurrence(),
                DemandState::Admitted,
                1,
            );
            row.source_commits.push(commit.clone());
            row.source_scope = Some(scope);
            state.records.insert(
                key(
                    &selected.producer,
                    generation,
                    u8::try_from(i).unwrap() + 100,
                ),
                row,
            );
        }
        state.source_custody.insert(
            commit.clone(),
            super::super::SourceCustody {
                occurrence: receipt.product_branch().occurrence(),
                root_kind: super::super::PreparedOutputRootKind::Discovered(
                    crate::equality_type_id::EqualityTypeId::of::<()>(),
                ),
                source: None,
                discovery: None,
                bound_sources: Some(vec![super::super::BoundOutputSource {
                    scope,
                    identity: selected.source.clone(),
                }]),
                consumed_sources: vec![selected.source],
                retired_sources: Vec::new(),
                retired: None,
                token_count: 0,
                cleanup: Default::default(),
                completed: true,
            },
        );
        state.note_custody(&commit);
    }
}
