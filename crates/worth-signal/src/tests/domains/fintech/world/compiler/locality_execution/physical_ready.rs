use std::collections::BTreeSet;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::progression::InvalidationWorkBindingAxes;
use crate::logic::planner::StageExecutionRecord;

/// Independent physical queue shape from performed bindings and executed slices.
/// The queue counters themselves are never inputs to this witness.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PhysicalReadyWitness {
    pub(super) nonempty_batches: u64,
    pub(super) maximum_items: u64,
}

pub(super) fn captured_bindings(graph: &SignalGraph) -> Vec<InvalidationWorkBindingAxes> {
    graph.invalidation_performed_work()
}

impl PhysicalReadyWitness {
    fn record_width(&mut self, width: usize) {
        if width > 0 {
            self.nonempty_batches += 1;
            self.maximum_items = self.maximum_items.max(width as u64);
        }
    }

    /// This fixture reads one input-free source per transaction. Reject any
    /// broader work shape rather than treating a logical epoch as a queue.
    pub(super) fn record_transaction(
        &mut self,
        source: NodeId,
        graph: &SignalGraph,
        before: &[InvalidationWorkBindingAxes],
        after: &[InvalidationWorkBindingAxes],
    ) -> Result<(), SignalError> {
        if !graph.dependencies_of(source)?.is_empty() {
            return Err(SignalError::internal(
                "physical source witness requires an input-free source",
            ));
        }
        let bindings = new_bindings(before, after)?;
        if bindings.len() > 1 || bindings.iter().any(|binding| binding.target != source) {
            return Err(SignalError::internal(
                "source transaction performed more than one source work item",
            ));
        }
        self.record_width(bindings.len());
        Ok(())
    }

    /// Each stage record is one actually admitted physical slice. A binding
    /// must belong to exactly one record in the request that performed it.
    pub(super) fn record_checked(
        &mut self,
        before: &[InvalidationWorkBindingAxes],
        after: &[InvalidationWorkBindingAxes],
        stages: &[StageExecutionRecord],
    ) -> Result<(), SignalError> {
        let bindings = new_bindings(before, after)?;
        let mut assigned = vec![false; bindings.len()];
        for stage in stages {
            let nodes = stage
                .task_records
                .iter()
                .map(|record| record.node)
                .collect::<BTreeSet<_>>();
            let mut width = 0;
            for (index, binding) in bindings.iter().enumerate() {
                if binding.stage_order.stage == stage.stage_index && nodes.contains(&binding.target)
                {
                    if assigned[index] {
                        return Err(SignalError::internal(
                            "performed ready work belongs to multiple physical slices",
                        ));
                    }
                    assigned[index] = true;
                    width += 1;
                }
            }
            self.record_width(width);
        }
        if assigned.iter().any(|assigned| !assigned) {
            return Err(SignalError::internal(
                "performed ready work lacks a physical stage record",
            ));
        }
        Ok(())
    }
}

fn new_bindings<'a>(
    before: &[InvalidationWorkBindingAxes],
    after: &'a [InvalidationWorkBindingAxes],
) -> Result<&'a [InvalidationWorkBindingAxes], SignalError> {
    if after.get(..before.len()) != Some(before) {
        return Err(SignalError::internal(
            "performed work capture changed during physical batch observation",
        ));
    }
    Ok(&after[before.len()..])
}
