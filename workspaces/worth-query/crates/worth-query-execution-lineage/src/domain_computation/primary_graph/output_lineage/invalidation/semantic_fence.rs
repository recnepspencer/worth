//! Whether any mark row a verification walk read changed meaning between its
//! reads and its fence (public #429).
//!
//! Each walk computes its own verdict. This module answers one question:
//! whether the rows a walk observed still mean what they meant when it read
//! them. The rule covers exactly the fields the walks read: terminal
//! resolution (`equal_links`), a row's fact binding, `historical`, its
//! upstream set, its output projection, and its `read_basis` through the
//! alignment predicate the walks evaluate. Fields neither walk reads (marks,
//! delivery bookkeeping, `verification_requirement`) may change freely.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::mark_state::MarkState;
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::verification::at_observation::reads_at_or_after;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

/// The settlement rows a walk read: the identity it asked for, the terminal
/// it resolved, and every row a projection walk passed through.
pub type ObservedRows = OrdSet<Arc<RecordedSettlementIdentity>>;

pub fn observe(observed: &mut ObservedRows, identity: &Arc<RecordedSettlementIdentity>) {
    observed.insert(Arc::clone(identity));
}

/// `true` when every row in `observed` means in `fresh` what it meant in
/// `old` for a reader at `selected`. A row absent from both images was walked
/// through, not read; a row present in only one changed.
pub fn same_observed_meaning(
    old: &MarkState,
    fresh: &MarkState,
    observed: &ObservedRows,
    selected: &PositionedRelationalSnapshot,
) -> bool {
    if !old.equal_links.ptr_eq(&fresh.equal_links) {
        return false;
    }
    observed.iter().all(|identity| {
        let (old, fresh) = match (old.settlements.get(identity), fresh.settlements.get(identity)) {
            (Some(old), Some(fresh)) => (old, fresh),
            (None, None) => return true,
            _ => return false,
        };
        if Arc::ptr_eq(old, fresh) {
            return true;
        }
        let same_output = match (&old.output_facts, &fresh.output_facts) {
            (Some(old), Some(fresh)) => Arc::ptr_eq(&old.facts, &fresh.facts),
            (None, None) => true,
            _ => false,
        };
        let same_historical = match (&old.historical, &fresh.historical) {
            (Some(old), Some(fresh)) => Arc::ptr_eq(old, fresh),
            (None, None) => true,
            _ => false,
        };
        // Both walks read `read_basis` only through this predicate; a
        // foreign runtime evaluates false in both.
        old.facts.same_binding(&fresh.facts)
            && old.consumed_upstream.ptr_eq(&fresh.consumed_upstream)
            && old.output_coverage == fresh.output_coverage
            && same_output
            && same_historical
            && reads_at_or_after(selected, &old.read_basis)
                == reads_at_or_after(selected, &fresh.read_basis)
    })
}
