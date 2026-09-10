//! Work admission for the selected node payloads and waiter bucket writes.
use super::{PreparedPendingRevalidationResolution, SignalGraph};
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::logic::evaluation::EvaluationWork;
impl SignalGraph {
    pub(crate) fn admit_pending_resolution_publication_work(
        &self,
        prepared: &PreparedPendingRevalidationResolution,
        producer: NodeId,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(prepared.nodes.len().checked_add(prepared.buckets.len()))?;
        for &node in prepared.nodes.keys() {
            // Producer copy work is already admitted by final packet validation.
            if node != producer {
                self.admit_downstream_node_mutation_work(work)?;
                self.admit_node_operational_copy_work(node, work)?;
            }
        }
        let map = &self.topology.pending_revalidation_waiters;
        let growth = crate::data::retained_storage::ordered_lookup_steps(prepared.buckets.len());
        let mutation = map
            .lookup_steps()
            .checked_add(growth.checked_mul(4).unwrap_or(usize::MAX))
            .and_then(|n| n.checked_mul(32));
        work.reserve(mutation.and_then(|n| n.checked_mul(prepared.buckets.len())))?;
        for (node, next) in &prepared.buckets {
            // OrdSet cloning shares its tree. Replacing a uniquely owned set
            // may destroy its fixed NodeId entries, so admit the old length.
            work.reserve(Some(map.lookup_steps()))?;
            let previous = map.get(node).map_or(0, im::OrdSet::len);
            work.reserve(
                previous
                    .checked_add(next.len())
                    .and_then(|n| n.checked_mul(32)),
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
