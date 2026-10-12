//! Closing a claimed row creates its debt at the row's sole mutation boundary.
use super::*;
pub(super) fn publish(row: &DemandRecord, node: &Arc<ClosedNoteBacking>) {
    let closed = row.interests == 0 && row.required_interests == 0;
    let debt = (closed && row.has_cached_ready() && !row.prerequisites.is_empty())
        || (row.terminal()
            && (row.framework_required_count != 0
                || (closed
                    && (!row.prerequisites.is_empty()
                        || row.prepared_prerequisite_claims != 0
                        || !row.settlements.is_empty()
                        || !row.performed_obligations.is_empty()))));
    if row.pending_cleanup_queued || !debt || node.closed_note.lock().unwrap().visiting {
        return;
    }
    if let Some(order) = node.order.lock().unwrap().upgrade() {
        order.lock().unwrap().enqueue(Arc::clone(node));
    }
}
