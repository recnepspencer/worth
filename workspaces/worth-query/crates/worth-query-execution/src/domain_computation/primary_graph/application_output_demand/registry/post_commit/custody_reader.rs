//! Binding has a World receipt but no admission with which to drain custody.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

pub fn resolve_before_binding(
    state: &mut DemandRegistryState,
    receipt: &WorthQueryApplicationCommitReceipt,
) {
    let commit = receipt.committed_product_publication().composite_commit();
    state.drain_custody_entry(commit);
}

/// This reads a publication already committed into source custody; there is
/// no request admission. All readers project the same eager entry eligibility.
pub fn visible(
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
