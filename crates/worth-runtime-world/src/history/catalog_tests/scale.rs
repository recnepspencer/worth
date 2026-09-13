use crate::history::reclamation::CompositeHistoryReclamationRequest;

use super::super::{CompositeHistoryCatalog, HistoryCatalogCounters};
use super::fixtures::{history_contract, linear_history};

#[test]
fn one_candidate_reclamation_has_equal_owner_counter_deltas_at_h_1_64_and_4096() {
    let deltas = [1_usize, 64, 4096].map(reclaim_one_from_history);
    assert_eq!(deltas[0], deltas[1]);
    assert_eq!(deltas[1], deltas[2]);
    assert_eq!(deltas[0].reachability_lookups, 1);
    assert_eq!(deltas[0].candidate_validations, 1);
    assert_eq!(deltas[0].dependency_decrements, 1);
    assert_eq!(deltas[0].reachability_rows_removed, 1);
    assert_eq!(deltas[0].metadata_releases, 1);
}

fn reclaim_one_from_history(non_root_commit_count: usize) -> ReclamationCounterDelta {
    let total_commits = non_root_commit_count
        .checked_add(1)
        .expect("test history length fits usize");
    let (owner, commits) = linear_history(total_commits);
    let root = commits.first().expect("root commit");
    let candidate = commits.last().expect("leaf commit");
    let owner_identity = root.identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(
        owner_identity,
        history_contract(total_commits as u64, u64::MAX),
    );
    for commit in &commits {
        catalog
            .append(commit.clone(), owner.history_pins(commit))
            .expect("history install");
    }
    let before = catalog.counters();
    let outcome = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![candidate.identity().clone()],
            1,
        ))
        .expect("leaf reclaim");
    assert_eq!(outcome.reclaimed_commits(), &[candidate.identity().clone()]);
    let after = catalog.counters();
    ReclamationCounterDelta::between(before, after)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReclamationCounterDelta {
    owner_validations: u64,
    parent_validations: u64,
    candidate_validations: u64,
    entry_lookups: u64,
    reachability_lookups: u64,
    dependency_increments: u64,
    dependency_decrements: u64,
    direct_protection_acquisitions: u64,
    direct_protection_releases: u64,
    reachability_rows_installed: u64,
    reachability_rows_removed: u64,
    metadata_reservation_checks: u64,
    metadata_reservations: u64,
    metadata_promotions: u64,
    metadata_releases: u64,
}

impl ReclamationCounterDelta {
    fn between(before: HistoryCatalogCounters, after: HistoryCatalogCounters) -> Self {
        Self {
            owner_validations: after.owner_validations() - before.owner_validations(),
            parent_validations: after.parent_validations() - before.parent_validations(),
            candidate_validations: after.candidate_validations() - before.candidate_validations(),
            entry_lookups: after.entry_lookups() - before.entry_lookups(),
            reachability_lookups: after.reachability_lookups() - before.reachability_lookups(),
            dependency_increments: after.dependency_increments() - before.dependency_increments(),
            dependency_decrements: after.dependency_decrements() - before.dependency_decrements(),
            direct_protection_acquisitions: after.direct_protection_acquisitions()
                - before.direct_protection_acquisitions(),
            direct_protection_releases: after.direct_protection_releases()
                - before.direct_protection_releases(),
            reachability_rows_installed: after.reachability_rows_installed()
                - before.reachability_rows_installed(),
            reachability_rows_removed: after.reachability_rows_removed()
                - before.reachability_rows_removed(),
            metadata_reservation_checks: after.metadata_reservation_checks()
                - before.metadata_reservation_checks(),
            metadata_reservations: after.metadata_reservations() - before.metadata_reservations(),
            metadata_promotions: after.metadata_promotions() - before.metadata_promotions(),
            metadata_releases: after.metadata_releases() - before.metadata_releases(),
        }
    }
}
