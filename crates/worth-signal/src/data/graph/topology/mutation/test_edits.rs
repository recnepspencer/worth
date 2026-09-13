use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;

use crate::data::graph::signal_graph::SignalGraph;

impl SignalGraph {
    pub(crate) fn append_simple_dependency_edge(
        &mut self,
        node: NodeId,
        upstream: NodeId,
        aspect: Aspect,
    ) -> Result<bool, SignalError> {
        let dependency = self.build_dependency_edge(upstream, aspect, None);
        let mut desired = self.raw_dependencies_of(node)?.to_vec();
        desired.push(dependency);
        Ok(self.reconcile_dependencies(node, &desired)?.added != 0)
    }

    pub(crate) fn drop_simple_dependency_edge(
        &mut self,
        node: NodeId,
        upstream: NodeId,
        aspect: Aspect,
    ) -> Result<bool, SignalError> {
        let desired = self
            .raw_dependencies_of(node)?
            .iter()
            .filter(|edge| !(edge.source() == upstream && edge.aspect() == aspect))
            .cloned()
            .collect::<Vec<_>>();
        Ok(self.reconcile_dependencies(node, &desired)?.removed != 0)
    }

    pub(crate) fn rewire_simple_dependency_edge(
        &mut self,
        node: NodeId,
        old_upstream: NodeId,
        new_upstream: NodeId,
        aspect: Aspect,
    ) -> Result<bool, SignalError> {
        if old_upstream == new_upstream {
            return Ok(false);
        }
        let new_dependency = self.build_dependency_edge(new_upstream, aspect, None);
        let mut replaced = false;
        let mut desired = self
            .raw_dependencies_of(node)?
            .iter()
            .filter_map(|edge| {
                if edge.source() == old_upstream && edge.aspect() == aspect {
                    replaced = true;
                    None
                } else {
                    Some(edge.clone())
                }
            })
            .collect::<Vec<_>>();
        if !replaced {
            return Ok(false);
        }
        desired.push(new_dependency);
        self.reconcile_dependencies(node, &desired)?;
        Ok(true)
    }
}
