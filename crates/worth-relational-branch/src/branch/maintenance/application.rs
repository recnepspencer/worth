//! Ordinary dispatch preserves untouched state and refuses a missing predecessor.
use super::{CommitChange, Maintained};
use crate::storage::substrate::maintenance::{ChangeWork, MaintenanceRefusal};
use std::sync::Arc;
use worth_relational_contracts::internal_port::storage::substrate::maintenance::work::ChangeWorkOwnerPort as _;

/// Cost: change routes times installed-route lookup, then the owner's delta.
/// The install-time dispatcher offers only owners reached by those routes.
pub fn apply_change<M: Maintained>(
    owner: &mut M,
    previous: Option<&Arc<M::State>>,
    change: &CommitChange,
    work: &mut ChangeWork<'_>,
) -> Result<Arc<M::State>, M::Error> {
    let previous = previous.ok_or(MaintenanceRefusal::MissingPredecessor)?;
    let mut relevant = false;
    for route in change.routes() {
        work.prepare_route_lookup(owner.routes().len())?;
        if owner.routes().contains(route) {
            relevant = true;
            break;
        }
    }
    if relevant {
        owner.apply(previous, change, work)
    } else {
        Ok(Arc::clone(previous))
    }
}
