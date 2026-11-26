//! Reversal facts follow navigation (public #56, spec-owner ruling (u); plan
//! §10.3 and §11.2.2). A recorded correction step keeps the aftermath-causality
//! fact its commit posted. A move from the selected step to another retires
//! the facts of the steps it leaves, `Line(current) \ Line(target)`, and
//! restores at their original IDs the facts of the steps it enters,
//! `Line(target) \ Line(current)`, so a fact is live exactly where its step is
//! on the selected line (`FactsFollowLine` in `TreeBasedUndo.tla`). The walk
//! climbs both ends to their common ancestor: O(path length) node probes,
//! and at most one fact per step.
use super::*;
use worth_relational::facade::branch::RelationalOwnedGraphCompanions;

impl HistoryJournal {
    /// The facts a move from the selected step to `target` retires and
    /// restores. `None` when a step on the path is not recorded here.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn facts_between(
        &self,
        target: WorthQueryHistoryNodeId,
    ) -> Option<RelationalOwnedGraphCompanions> {
        let mut companions = RelationalOwnedGraphCompanions::default();
        let (mut leaving, mut entering) = (self.current.ordinal(), target.ordinal());
        while leaving != entering {
            self.access(Access::Node);
            let (from, to) = (self.nodes.get(&leaving)?, self.nodes.get(&entering)?);
            if from.line.depth() >= to.line.depth() {
                companions.retire.extend(from.posted_fact());
                leaving = from.line.parent()?;
            } else {
                companions.restore.extend(to.posted_fact());
                entering = to.line.parent()?;
            }
        }
        Some(companions)
    }
}

impl HistoryJournal {
    /// Each recorded correction step, its posted fact and its commit.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn posted_facts(
        &self,
    ) -> Vec<(
        WorthQueryHistoryNodeId,
        worth_relational::facade::identity::EntityId,
        worth_runtime_world::facade::CompositeCommitIdentity,
    )> {
        self.nodes
            .iter()
            .filter_map(|(&ordinal, node)| match &node.binding {
                NodeBinding::Completed(RecordedNode {
                    posted_fact: Some(fact),
                    source_commit: Some(commit),
                    ..
                }) => Some((self.incarnation.node(ordinal), *fact, commit.clone())),
                _ => None,
            })
            .collect()
    }

    /// Each recorded source commit and its step.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn recorded_sources(
        &self,
    ) -> Vec<(worth_runtime_world::facade::CompositeCommitIdentity, WorthQueryHistoryNodeId)> {
        self.sources
            .iter()
            .map(|(commit, (ordinal, _))| (commit.clone(), self.incarnation.node(*ordinal)))
            .collect()
    }
}

impl HistoryNode {
    /// The reversal fact this step's commit posted, if it is a recorded
    /// correction.
    fn posted_fact(&self) -> Option<worth_relational::facade::identity::EntityId> {
        match &self.binding {
            NodeBinding::Completed(recorded) => recorded.posted_fact,
            NodeBinding::Reserved { .. } => None,
        }
    }
}
