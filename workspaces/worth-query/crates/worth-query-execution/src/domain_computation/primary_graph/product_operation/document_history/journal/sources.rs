//! Correction sources: which recorded step a commit belongs to, and whether
//! that step is on the selected line, at logarithmic cost (plan §10.1a and
//! §11.1; cost law 2). A source commit maps to its step through an index the
//! adoption fills, and each node keeps its depth and a skew-binary jump
//! pointer (Myers' jump pointers), so the level-ancestor query that decides
//! "on the selected line" takes O(log depth) node probes, never a walk of the
//! line.
use super::*;

/// Where a node stands in the history tree.
#[derive(Clone, Copy, Debug)]
pub(in crate::domain_computation::primary_graph::product_operation::document_history) struct LinePosition {
    parent: Option<u64>,
    depth: u64,
    jump: u64,
}

impl LinePosition {
    pub(super) const ROOT: Self = Self {
        parent: None,
        depth: 0,
        jump: 0,
    };
}

impl<S> HistoryJournal<'_, S> {
    /// The position of a new child of `parent`: one deeper, and a jump that
    /// keeps every level-ancestor query logarithmic.
    pub(super) fn child_position(&self, parent: u64) -> LinePosition {
        let at = |ordinal: u64| {
            self.access(Access::Node);
            self.nodes
                .get(&ordinal)
                .map(|node| node.line)
                .expect("a parent and its jumps are retained nodes")
        };
        let above = at(parent);
        let jumped = at(above.jump);
        let jump = if above.depth - jumped.depth == jumped.depth - at(jumped.jump).depth {
            jumped.jump
        } else {
            parent
        };
        LinePosition {
            parent: Some(parent),
            depth: above.depth + 1,
            jump,
        }
    }

    /// Index the commit an envelope's stages made as `node`'s, while the
    /// record is prepared; a cancelled record releases it again.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn index_source(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        node: WorthQueryHistoryNodeId,
    ) -> Result<(), worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        let allocation = self.allocate::<(
            worth_runtime_world::facade::CompositeCommitIdentity,
            (u64, HistoryInsertionAllocation),
        )>(execution, self.sources.len(), 0)?;
        self.access(Access::Node);
        self.sources
            .insert(commit.clone(), (node.ordinal(), allocation));
        self.pending_source = Some(commit.clone());
        Ok(())
    }

    /// A cancelled record's indexed source goes with it.
    pub(super) fn release_pending_source(&mut self, node: u64) {
        if let Some(commit) = self.pending_source.take()
            && self.sources.get(&commit).is_some_and(|(indexed, _)| *indexed == node)
        {
            self.sources.remove(&commit);
        }
    }

    /// The recorded step on the selected line (the current step or one of
    /// its ancestors, never the root) whose stages made `commit`.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn selected_line_source(
        &self,
        commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) -> Option<WorthQueryHistoryNodeId> {
        self.access(Access::Selection);
        self.access(Access::Node);
        let ordinal = self.sources.get(commit)?.0;
        if ordinal == 0 {
            return None;
        }
        let depth = self.position(ordinal)?.depth;
        let mut at = self.current.ordinal();
        loop {
            let position = self.position(at)?;
            if position.depth == depth {
                return (at == ordinal).then(|| self.incarnation.node(ordinal));
            }
            if position.depth < depth {
                return None;
            }
            at = if self.position(position.jump)?.depth >= depth {
                position.jump
            } else {
                position.parent?
            };
        }
    }

    fn position(&self, ordinal: u64) -> Option<LinePosition> {
        self.access(Access::Node);
        Some(self.nodes.get(&ordinal)?.line)
    }
}
