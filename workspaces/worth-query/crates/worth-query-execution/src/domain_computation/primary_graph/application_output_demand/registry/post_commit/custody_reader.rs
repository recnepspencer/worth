//! Binding has a World receipt but no admission with which to drain custody.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

pub(in super::super) fn resolve_before_binding(
    state: &mut DemandRegistryState,
    receipt: &WorthQueryApplicationCommitReceipt,
) {
    let commit = receipt.committed_product_publication().composite_commit();
    state.drain_custody_entry(commit);
}

/// This reads a publication already committed into source custody; there is
/// no request admission. All readers project the same eager entry eligibility.
pub(in super::super) fn visible(
    state: &DemandRegistryState,
    commit: &worth_runtime_world::facade::CompositeCommitIdentity,
) -> bool {
    // The commit was the publication that installed this custody entry.
    !state.custody_prunable(commit)
}

/// A new committed publication replaces a logically absent slot. Consume that
/// slot's own debt before reusing its identity; unrelated notes are untouched.
pub(in super::super) fn prune_invisible_slot(
    state: &mut DemandRegistryState,
    commit: &worth_runtime_world::facade::CompositeCommitIdentity,
) {
    if state.source_custody.contains_key(commit) {
        state.note_custody(commit);
        state.prune_custody_entry(commit);
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::walk_bounds;
    use super::*;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

    #[test]
    fn binding_reader_drains_only_its_named_queued_custody() {
        let receipts = walk_bounds::receipt_batch();
        let mut state = DemandRegistryState::default();
        walk_bounds::add_custody(&mut state, &receipts[..4], 1);
        for mut row in state.records.values_mut() {
            row.interests = 0;
            row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::NoEffect,
                "completed reader row",
            ));
        }
        for generation in 1..=64 {
            state.records.insert(
                super::super::super::tests::key("unrelated", generation, 2),
                super::super::super::tests::record(
                    receipts[0].product_branch().occurrence(),
                    DemandState::Admitted,
                    1,
                ),
            );
        }
        let commit = |i: usize| {
            receipts[i]
                .committed_product_publication()
                .composite_commit()
        };
        assert_eq!(state.source_custody.len(), 4);
        assert_eq!(state.custody_cleanup.len, 4);
        for i in 0..4 {
            assert!(state.source_custody[commit(i)].cleanup.queued);
        }
        resolve_before_binding(&mut state, &receipts[2]);
        assert!(!state.source_custody.contains_key(commit(2)));
        for i in [0, 1, 3] {
            assert!(
                state.source_custody.contains_key(commit(i)),
                "unrelated custody stays owned"
            );
            assert!(state.source_custody[commit(i)].cleanup.queued);
        }
        assert_eq!(
            state.custody_cleanup.len, 3,
            "only the named middle entry was drained"
        );
        assert_eq!(
            state
                .records
                .values()
                .filter(|row| row.interests == 1)
                .count(),
            64,
            "binding cleanup preserves unrelated live rows"
        );
    }
}
