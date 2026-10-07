//! Read-only affected waiter closure before checked evaluator dispatch.
use std::collections::BTreeSet;

use worth_execution::MapKernelContext;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::btree_structure_charge;

impl SignalGraph {
    /// Only buckets reachable from this producer are observed. A later
    /// producer can reuse `admitted` without revisiting an admitted closure.
    pub(crate) fn epoch_waiter_closure(
        &self,
        producer: NodeId,
        admitted: &BTreeSet<NodeId>,
        mut request: Option<&mut MapKernelContext<'_, '_>>,
        budget: &mut SignalPreparationBudget,
    ) -> Result<BTreeSet<NodeId>, SignalError> {
        let mut discovered = BTreeSet::new();
        let mut frontier = BTreeSet::new();
        if !admitted.contains(&producer) {
            claim_node(budget)?;
            discovered.insert(producer);
            frontier.insert(producer);
        }
        while let Some(node) = frontier.pop_first() {
            checkpoint(
                request.as_deref_mut(),
                self.topology.pending_revalidation_waiters.lookup_steps(),
            )?;
            let Some(bucket) = self.topology.pending_revalidation_waiters.get(&node) else {
                continue;
            };
            for &consumer in bucket {
                checkpoint(request.as_deref_mut(), 1)?;
                if admitted.contains(&consumer)
                    || discovered.contains(&consumer)
                    || !self.is_alive(consumer)
                {
                    continue;
                }
                claim_node(budget)?;
                discovered.insert(consumer);
                frontier.insert(consumer);
            }
        }
        Ok(discovered)
    }
}

fn claim_node(budget: &mut SignalPreparationBudget) -> Result<(), SignalError> {
    let one = btree_structure_charge::<NodeId, ()>(1)
        .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
    let both = one
        .checked_mul(2)
        .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
    budget.claim(both.bytes())
}

fn checkpoint(
    request: Option<&mut MapKernelContext<'_, '_>>,
    visits: usize,
) -> Result<(), SignalError> {
    if let Some(request) = request {
        request
            .checkpoint(visits as u64)
            .map_err(SignalError::execution_checkpoint_stopped)?;
    }
    Ok(())
}
