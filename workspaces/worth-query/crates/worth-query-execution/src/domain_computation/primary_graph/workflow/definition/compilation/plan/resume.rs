//! The graph questions migration asks of a target definition.

use std::collections::BTreeSet;

use worth_relational::facade::identity::EntityId;

use super::{CompiledWorkflowConnectionKind, CompiledWorkflowDefinition, CompiledWorkflowNode};

impl CompiledWorkflowDefinition {
    /// The same definition read from `path` as its first node. Progress,
    /// replay and history of a migration successor all begin there.
    pub(in crate::domain_computation::primary_graph) fn resumed_at(
        &self,
        path: &str,
    ) -> Option<Self> {
        let [node] = self.nodes_with_path(path) else {
            return None;
        };
        let ordinal = self.node_ordinal(node.entity())?;
        Some(Self {
            resume: Some(ordinal),
            ..self.clone()
        })
    }

    /// The node a migration successor began at, when it did not begin at the
    /// definition's start.
    pub(in crate::domain_computation::primary_graph) fn resumed_path(&self) -> Option<&str> {
        self.resume
            .map(|ordinal| self.publication.nodes[ordinal].path())
    }

    /// The node a fresh instance of this definition begins at, whatever
    /// node this compilation resumes at.
    pub(in crate::domain_computation::primary_graph) fn definition_start(&self) -> EntityId {
        self.publication.nodes[self.publication.start].entity()
    }

    /// Every node control or retry can run from `start`, including `start`,
    /// without ever running `avoiding`. Avoiding `start` itself reaches
    /// nothing.
    pub(in crate::domain_computation::primary_graph) fn reachable_from(
        &self,
        start: EntityId,
        avoiding: Option<EntityId>,
    ) -> BTreeSet<EntityId> {
        if avoiding == Some(start) {
            return BTreeSet::new();
        }
        let mut reached = BTreeSet::from([start]);
        let mut pending = vec![start];
        while let Some(source) = pending.pop() {
            for edge in self.outgoing(source) {
                let runs = matches!(
                    self.publication.connections[edge.connection].kind.as_ref(),
                    CompiledWorkflowConnectionKind::Control(_)
                        | CompiledWorkflowConnectionKind::Retry { .. }
                );
                let peer = self.publication.nodes[edge.peer].entity();
                if runs && avoiding != Some(peer) && reached.insert(peer) {
                    pending.push(peer);
                }
            }
        }
        reached
    }

    /// Every node whose result `target` consumes, whatever the data flow.
    pub(in crate::domain_computation::primary_graph) fn consumed_sources(
        &self,
        target: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.incoming(target).filter_map(move |edge| {
            match self.publication.connections[edge.connection].kind.as_ref() {
                CompiledWorkflowConnectionKind::Data(_) => Some(&self.publication.nodes[edge.peer]),
                _ => None,
            }
        })
    }
}
