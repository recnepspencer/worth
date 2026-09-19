//! Reserved cumulative node roots prepared before native output publication.
use super::node_publication::{visit_output_node_changes, PreparedProducerNodeChanges};
use super::{OutputCommitStorage, SignalGraph};
use crate::data::error::SignalError;
use crate::data::graph::runtime::graph::{
    map_node_edit as map_edit, map_node_edit_accounting as map_accounting,
    map_node_edit_retention as map_retention,
};
use crate::data::graph::runtime::graph::{PreparedRetainedNodeEdit, RetainedNodeEditPreparation};
use crate::data::graph::storage::NodeEvaluationMutation;
use crate::data::graph::{
    PreparedPendingRevalidationIndex, PreparedRetainedPendingRevalidationIndex,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
};
use crate::logic::evaluation::EvaluationWork;

type RawNodeResult = (
    super::super::EffectStateMutation,
    Option<PreparedPendingRevalidationIndex>,
);

impl SignalGraph {
    pub(super) fn prepare_retained_output_nodes(
        &mut self,
        storage: &mut OutputCommitStorage,
        state: &mut Option<super::super::PreparedEffectNodeState>,
        allowance: &mut EvaluationWork<'_>,
    ) -> Result<Option<super::retained_publication::PreparedRetainedOutputPublication>, SignalError>
    {
        if self.arena.retained_node_ledger.is_none() {
            return Ok(None);
        }
        match allowance {
            EvaluationWork::Conditional(work) => self
                .prepare_reserved_output_nodes(storage, state, work)
                .map(Some),
            EvaluationWork::Ordinary => {
                let maximum = self
                    .installed_runtime_policy()
                    .conditional_evaluation_budget()
                    .maximum_attempt_visits;
                self.prepare_reserved_output_nodes(storage, state, &mut Work::new(maximum))
                    .map(Some)
            }
        }
    }

    fn prepare_reserved_output_nodes(
        &mut self,
        storage: &mut OutputCommitStorage,
        state: &mut Option<super::super::PreparedEffectNodeState>,
        work: &mut Work,
    ) -> Result<super::retained_publication::PreparedRetainedOutputPublication, SignalError> {
        let ledger = self
            .arena
            .retained_node_ledger
            .as_ref()
            .expect("retained evaluation context")
            .clone();
        let node = storage.apply.effect.operational.node;
        let suppressed_downstream = storage
            .direct_causes
            .as_ref()
            .map_or(0, |p| p.suppressed_downstream_count());
        let (direct, stores) = match storage.direct_causes.take() {
            Some(prepared) => {
                let (nodes, stores) = prepared.split_node_changes();
                (Some(nodes), Some(stores))
            }
            None => (None, None),
        };
        let count = direct
            .as_ref()
            .map_or(1, |nodes| nodes.selected_nodes().len());
        work.reserve_visits(count).map_err(map_accounting)?;
        let selection_charge = Charge::capacity::<usize>(count).map_err(map_accounting)?;
        let growth = self.arena.warm[node.index() as usize]
            .aspect_version_overrides
            .evaluation_growth_charge(storage.apply.effect.changed_regions(), work)
            .map_err(map_accounting)?;
        let extra = selection_charge
            .checked_add(growth)
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<crate::data::node::NodeColdData>(1)?)
            })
            .map_err(map_accounting)?;
        // This covers allocation by this node preparation only. Earlier output
        // materialization remains the enclosing attempt's admission obligation.
        let _payload_custody = ledger.reserve(0, extra).map_err(map_retention)?;
        let indices = match direct.as_ref() {
            Some(nodes) => nodes
                .selected_nodes()
                .map(|node| node.index() as usize)
                .collect::<Vec<_>>(),
            None => vec![node.index() as usize],
        };
        let state = state.take().expect("prepared producer state");
        let release = state.release_causes;
        let write = std::mem::take(&mut storage.artifact_write);
        let runtime_write = write.runtime.is_some();
        let causality = storage.apply.effect.take_causality();
        let causality_changed = causality.is_some();
        let producer = PreparedProducerNodeChanges {
            state,
            write,
            causality,
            snapshot: storage
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.node_snapshot_id()),
        };
        let maximum = self
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_retained_bytes;
        let maximum = usize::try_from(maximum)
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
        let maximum_charge = Charge::capacity::<u8>(maximum).map_err(map_accounting)?;
        let roots = self
            .arena
            .prepare_retained_node_edits(&ledger, &indices, maximum_charge, work, |payloads| {
                let mut payloads = indices.iter().zip(payloads.iter_mut());
                visit_output_node_changes(node, producer, direct, |selected, changes| {
                    let (&index, payload) = payloads.next().expect("one selected payload per node");
                    assert_eq!(index, selected.index() as usize, "prepared node order");
                    Ok(changes.apply(
                        &mut NodeEvaluationMutation::draft(
                            &mut payload.hot,
                            &mut payload.warm,
                            &mut payload.cold,
                        ),
                        storage.apply.effect.changed_regions(),
                    ))
                })
                .expect("validated payload visitation is infallible")
            })
            .map_err(map_edit)?;
        let roots = match roots {
            RetainedNodeEditPreparation::Ready(roots) => roots,
            RetainedNodeEditPreparation::Rejected { denial, .. } => return Err(map_edit(denial)),
        };
        let (roots, waiters) =
            self.prepare_retained_waiter_root(roots, &ledger, maximum_charge, work)?;
        let stores = stores
            .map(|stores| stores.prepare_retained(self, &ledger, maximum_charge, work))
            .transpose()?;
        Ok(
            super::retained_publication::PreparedRetainedOutputPublication {
                roots,
                waiters,
                stores,
                snapshot: storage.snapshot.take(),
                node,
                release,
                causality_changed,
                runtime_write,
                suppressed_downstream,
            },
        )
    }

    #[inline(never)]
    fn prepare_retained_waiter_root(
        &mut self,
        roots: PreparedRetainedNodeEdit<RawNodeResult>,
        ledger: &std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionLedger>,
        maximum: Charge,
        work: &mut Work,
    ) -> Result<
        (
            PreparedRetainedNodeEdit<super::super::EffectStateMutation>,
            Option<PreparedRetainedPendingRevalidationIndex>,
        ),
        SignalError,
    > {
        roots.try_split_output(|(mutation, index)| {
            let index = index
                .map(|index| index.prepare_retained(self, ledger, maximum, work))
                .transpose()?;
            Ok((mutation, index))
        })
    }
}

#[cfg(test)]
mod tests;
