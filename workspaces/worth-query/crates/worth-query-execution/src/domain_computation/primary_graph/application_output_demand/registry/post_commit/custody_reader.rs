//! Binding has a World receipt but no admission with which to drain custody.
use super::super::refreshed_rejoin::infallible;
use super::walk_charge::PostCommitSweep;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

pub(in super::super) fn resolve_before_binding(
    state: &mut DemandRegistryState,
    receipt: &WorthQueryApplicationCommitReceipt,
) {
    if state.pending_custody_prune() {
        infallible(
            state.prune_completed_custody(&mut PostCommitSweep::after_source_commit(receipt)),
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
            let mut state = DemandRegistryState::default();
            walk_bounds::add_custody(&mut state, &receipts[..count], 1);
            for row in state.records.values_mut() {
                row.interests = 0;
                row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::NoEffect,
                    "completed reader row",
                ));
            }
            state.note_custody_changes(|_, _| true);
            let before = super::super::walk_charge::sweep_contacts();
            resolve_before_binding(&mut state, &receipts[0]);
            contacts.push(super::super::walk_charge::sweep_contacts() - before);
            assert!(state.source_custody.is_empty());
            assert!(!state.pending_custody_prune());
        }
        // Each entry: custody entry + held row + stale-owner row + retiring row.
        assert_eq!(contacts, [4, 8, 16]);
    }
}
