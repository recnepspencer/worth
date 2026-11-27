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

impl HistoryJournal {
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

    /// Index the commit an envelope's stages made, and the step's retained
    /// basis with the commit its candidate forked from (`after`), as `node`'s
    /// at adoption; a cancelled record releases both again. An entry that
    /// already names another step is kept, never overwritten: each step's
    /// commits are its own fresh ones, so a collision should not happen, and
    /// if one did, keeping the earliest means a release removes only what this
    /// record inserted. An adoption retry indexes the same basis again, which
    /// the guard skips.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn index_source<'c>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        commits: impl IntoIterator<Item = &'c worth_runtime_world::facade::CompositeCommitIdentity>,
        after: Option<worth_relational::facade::history::CommitId>,
        endpoint: Option<worth_relational::facade::history::CommitId>,
        node: WorthQueryHistoryNodeId,
    ) -> Result<(), worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        // A gesture's step names each member's commit.
        for commit in commits {
            if self.sources.contains_key(commit) {
                continue;
            }
            let allocation = self.allocate::<(
                worth_runtime_world::facade::CompositeCommitIdentity,
                (u64, HistoryInsertionAllocation),
            )>(execution, self.sources.len(), 0)?;
            self.access(Access::Node);
            self.sources.insert(commit.clone(), (node.ordinal(), allocation));
            self.pending_sources.push(commit.clone());
        }
        if let Some(endpoint) = endpoint.filter(|endpoint| !self.endpoints.contains_key(endpoint)) {
            // The index is written once, at adoption: a pending record never
            // indexes a second, different basis.
            debug_assert!(self.pending_endpoint.is_none(), "a pending record indexes one basis");
            let allocation = self.allocate::<(
                worth_relational::facade::history::CommitId,
                (Option<worth_relational::facade::history::CommitId>, u64, HistoryInsertionAllocation),
            )>(execution, self.endpoints.len(), 0)?;
            self.access(Access::Node);
            self.endpoints.insert(endpoint, (after, node.ordinal(), allocation));
            self.pending_endpoint = Some(endpoint);
        }
        Ok(())
    }

    /// A cancelled record's indexed source and basis go with it.
    pub(super) fn release_pending_source(&mut self, node: u64) {
        for commit in std::mem::take(&mut self.pending_sources) {
            if self.sources.get(&commit).is_some_and(|(indexed, _)| *indexed == node) {
                self.sources.remove(&commit);
            }
        }
        if let Some(endpoint) = self.pending_endpoint.take()
            && self.endpoints.get(&endpoint).is_some_and(|(_, indexed, _)| *indexed == node)
        {
            self.endpoints.remove(&endpoint);
        }
    }

    /// For a correction's restored support (S3 ruling B): whether the step
    /// whose retained basis is `selected` is on the selected line and
    /// descends from the step whose stages made `source`.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn restores_descendant_of(
        &self,
        selected: worth_relational::facade::history::CommitId,
        source: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) -> bool {
        self.access(Access::Selection);
        self.access(Access::Node);
        let (Some(basis), Some(step)) = (
            self.endpoints.get(&selected).map(|(_, ordinal, _)| *ordinal),
            self.sources.get(source).map(|(ordinal, _)| *ordinal),
        ) else {
            return false;
        };
        step != 0
            && self.is_ancestor_or_self(step, basis)
            && self.is_ancestor_or_self(basis, self.current.ordinal())
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
        (ordinal != 0 && self.is_ancestor_or_self(ordinal, self.current.ordinal()))
            .then(|| self.incarnation.node(ordinal))
    }

    /// Whether `ancestor` is `node` or one of its ancestors: a level-ancestor
    /// lift of `node` to the depth of `ancestor`, O(log depth) node probes.
    fn is_ancestor_or_self(&self, ancestor: u64, node: u64) -> bool {
        let Some(depth) = self.position(ancestor).map(|position| position.depth) else {
            return false;
        };
        let mut at = node;
        loop {
            let Some(position) = self.position(at) else {
                return false;
            };
            if position.depth == depth {
                return at == ancestor;
            }
            if position.depth < depth {
                return false;
            }
            at = match self.position(position.jump) {
                Some(jumped) if jumped.depth >= depth => position.jump,
                _ => match position.parent {
                    Some(parent) => parent,
                    None => return false,
                },
            };
        }
    }

    fn position(&self, ordinal: u64) -> Option<LinePosition> {
        self.access(Access::Node);
        Some(self.nodes.get(&ordinal)?.line)
    }

    /// For a correction's restored support (public #453): whether the
    /// relational commit `commit` is a write of a recorded step on the path
    /// from the step whose stages made `source` to the step whose retained
    /// basis is `selected`. Such a line write is kept with its witness. A
    /// step's commits lie in `(after, basis]`, and no other step's basis falls
    /// between them, so the first basis at or after `commit` names its
    /// writer; a commit at or before that step's fork is none of its writes.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn wrote_on_line_through(
        &self,
        commit: worth_relational::facade::history::CommitId,
        selected: worth_relational::facade::history::CommitId,
        source: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) -> bool {
        self.access(Access::Selection);
        self.access(Access::Node);
        let (Some(basis), Some(step), Some((_, (after, writer, _)))) = (
            self.endpoints.get(&selected).map(|(_, ordinal, _)| *ordinal),
            self.sources.get(source).map(|(ordinal, _)| *ordinal),
            self.endpoints.range(commit..).next(),
        ) else {
            return false;
        };
        !after.is_some_and(|after| commit <= after)
            && self.is_ancestor_or_self(step, *writer)
            && self.is_ancestor_or_self(*writer, basis)
    }
}

impl HistoryJournal {
    /// The number of indexed source commits and recorded bases, for courts.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn source_index_sizes(
        &self,
    ) -> (usize, usize) {
        (self.sources.len(), self.endpoints.len())
    }

    /// The first recorded basis and the first indexed source commit, for courts.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn first_indexed_source(
        &self,
    ) -> Option<(worth_relational::facade::history::CommitId, worth_runtime_world::facade::CompositeCommitIdentity)> {
        Some((*self.endpoints.keys().next()?, self.sources.keys().next()?.clone()))
    }
}

impl LinePosition {
    /// The parent step, `None` at the root.
    pub(super) fn parent(&self) -> Option<u64> {
        self.parent
    }
    /// The number of steps between this node and the root.
    pub(super) fn depth(&self) -> u64 {
        self.depth
    }
}
