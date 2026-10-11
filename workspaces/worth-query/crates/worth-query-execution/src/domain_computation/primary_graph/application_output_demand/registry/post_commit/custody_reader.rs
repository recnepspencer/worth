//! Binding has a World receipt but no admission with which to drain custody.
use super::super::refreshed_rejoin::infallible;
use super::walk_charge::PostCommitSweep;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

pub(in super::super) fn resolve_before_binding(
    state: &mut DemandRegistryState,
    receipt: &WorthQueryApplicationCommitReceipt,
) {
    let visits = state
        .custody_cleanup
        .len
        .min(super::super::cleanup::custody::FOREIGN_CUSTODY_DRAIN_QUOTA);
    for _ in 0..visits {
        let Some(commit) = state.custody_cleanup.head.clone() else {
            break;
        };
        infallible(
            state.prune_custody_entry(&commit, &mut PostCommitSweep::after_source_commit(receipt)),
        );
    }
}

/// This reads a publication already committed into source custody; there is
/// no request admission. All readers project the same eager entry eligibility.
pub(in super::super) fn visible(
    state: &DemandRegistryState,
    commit: &worth_runtime_world::facade::CompositeCommitIdentity,
) -> bool {
    // The commit was the publication that installed this custody entry.
    !infallible(state.custody_prunable(commit, &mut PostCommitSweep::after_effect()))
}

/// A new committed publication replaces a logically absent slot. Consume that
/// slot's own debt before reusing its identity; unrelated notes are untouched.
pub(in super::super) fn prune_invisible_slot(
    state: &mut DemandRegistryState,
    commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    receipt: &WorthQueryApplicationCommitReceipt,
) {
    if state.source_custody.contains_key(commit) {
        state.note_custody(commit);
        infallible(
            state.prune_custody_entry(commit, &mut PostCommitSweep::after_source_commit(receipt)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::walk_bounds;
    use super::*;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

    #[test]
    fn registry_binding_reader_sweep_names_its_custody_bound() {
        let receipts = walk_bounds::receipt_batch();
        let mut contacts = Vec::new();
        for count in [1, 2, 4] {
            for neighbors in [1, 64] {
                let mut state = DemandRegistryState::default();
                walk_bounds::add_custody(&mut state, &receipts[..count], 1);
                for mut row in state.records.values_mut() {
                    row.interests = 0;
                    row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::NoEffect,
                        "completed reader row",
                    ));
                }
                for generation in 1..=neighbors {
                    state.records.insert(
                        super::super::super::tests::key("larger-neighbor", generation, 2),
                        super::super::super::tests::record(
                            receipts[0].product_branch().occurrence(),
                            DemandState::Admitted,
                            1,
                        ),
                    );
                }
                state.note_custody_changes(|_, _| true);
                let before = super::super::walk_charge::sweep_contacts();
                resolve_before_binding(&mut state, &receipts[0]);
                contacts.push(super::super::walk_charge::sweep_contacts() - before);
                assert_eq!(state.source_custody.len(), count.saturating_sub(2));
                assert_eq!(state.pending_custody_prune(), count > 2);
            }
        }
        // Each entry: custody entry + held row + three projected searches + retiring row.
        assert_eq!(contacts, [6, 6, 12, 12, 12, 12]);
    }
}
