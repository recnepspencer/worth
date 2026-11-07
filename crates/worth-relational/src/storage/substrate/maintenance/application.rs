//! Ordinary dispatch preserves untouched state and refuses a missing predecessor.
use super::{ChangeWork, CommitChange, Maintained, MaintenanceRefusal};
use std::sync::Arc;

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
        let levels = owner
            .routes()
            .len()
            .checked_ilog2()
            .map_or(1, |bits| u64::from(bits) + 1);
        work.prepare(11 * levels + 1, 0)?;
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
