//! Selected payload copies at exclusive output publication. Container mutation
//! and retained-byte admission are separate obligations.
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::evaluation::EvaluationWork;

impl SignalGraph {
    pub(crate) fn admit_effect_node_copy_work(
        &self,
        node: NodeId,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        self.admit_node_operational_copy_work(node, work)?;
        if self.arena.cold.exclusive_capacity().is_none() {
            work.reserve(Some(self.arena.cold.lookup_steps()))?;
            if let Some(cold) = self.cold_ref(node)? {
                cold.admit_clone_work(work)?;
            }
        }
        Ok(())
    }

    pub(crate) fn admit_node_operational_copy_work(
        &self,
        node: NodeId,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        self.validate_handle(node)?;
        // No storage fork intervenes between final packet validation and
        // publication. Each selected retained payload becomes unique at most
        // once, even when multiple setters subsequently visit that payload.
        if self.arena.hot.exclusive_capacity().is_none() {
            work.reserve(Some(
                std::mem::size_of::<crate::data::node::NodeHotData>() + 32,
            ))?;
        }
        if self.arena.warm.exclusive_capacity().is_none() {
            work.reserve(Some(self.arena.warm.lookup_steps()))?;
            self.warm_ref(node)?.admit_clone_work(work)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
