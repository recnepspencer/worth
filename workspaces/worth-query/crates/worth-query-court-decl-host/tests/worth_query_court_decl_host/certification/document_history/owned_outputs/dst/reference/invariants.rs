//! The model's safety invariants, checked on every state the oracle keeps.
//! A broken one is a fault in the oracle's translation, not in the system.
use super::*;

impl State {
    pub(super) fn broken_invariant(&self) -> Option<&'static str> {
        [
            ("SnapshotExactness", self.snapshot_exactness()),
            ("OwnershipAndOutputsConsistent", self.ownership_consistent()),
            ("TreeStructureValid", self.tree_structure_valid()),
            ("IdempotencyValid", self.idempotency_valid()),
            ("GenerationalCustody", self.generational_custody()),
            ("StoppedCustody", self.stopped_custody()),
            ("QueuedHeads", !self.closed || self.heads == 0),
            ("NoWedge", self.no_wedge()),
            ("FactsFollowLine", self.facts_follow_line()),
            ("FactsAgreeWithSnapshots", self.facts_agree_with_snapshots()),
        ]
        .into_iter()
        .find_map(|(name, holds)| (!holds).then_some(name))
    }

    fn snapshot_exactness(&self) -> bool {
        self.head_diverged || self.branch == self.snapshots[self.current as usize]
    }

    fn ownership_consistent(&self) -> bool {
        let closed = |s: &Snapshot| !s.live[E2] || s.live[E1];
        let exact = |s: &Snapshot| {
            ENTITIES.iter().all(|&e| {
                if s.live[e] {
                    s.generation[e] > 0 && s.value[e] > 0
                } else {
                    s.generation[e] == 0 && s.value[e] == 0
                }
            })
        };
        closed(&self.branch) && exact(&self.branch) && self.snapshots.iter().all(closed)
    }

    fn tree_structure_valid(&self) -> bool {
        self.parent[0].is_none()
            && (1..self.nodes()).all(|n| matches!(self.parent[n as usize], Some(p) if p < n))
    }

    /// One key per recorded step in each namespace, no step under two keys,
    /// and a writer records only edits.
    fn idempotency_valid(&self) -> bool {
        let nodes: BTreeSet<u32> = self.journal.values().map(|entry| entry.node).collect();
        let writers_edit = self.journal.iter().all(|((namespace, _), entry)| {
            *namespace == Namespace::Envelope || matches!(entry.intent, Intent::Edit(_))
        });
        nodes.len() == self.journal.len() && nodes == (1..self.nodes()).collect() && writers_edit
    }

    fn generational_custody(&self) -> bool {
        let owner_of = |generation: u32| {
            self.snapshots
                .iter()
                .flat_map(|s| ENTITIES.map(|e| (s.generation[e], e)))
                .filter(move |(g, _)| *g == generation && *g > 0)
                .map(|(_, e)| e)
        };
        let one_entity = self.snapshots.iter().all(|s| {
            ENTITIES
                .iter()
                .all(|&e| owner_of(s.generation[e]).all(|owner| owner == e))
        });
        let below_next = self
            .snapshots
            .iter()
            .all(|s| s.generation.iter().all(|g| *g < self.next_generation));
        one_entity && below_next && (1..self.nodes()).all(|n| self.step_matches_its_ops(n))
    }

    fn step_matches_its_ops(&self, node: u32) -> bool {
        let n = node as usize;
        let before = &self.snapshots[self.parent[n].unwrap() as usize];
        let after = &self.snapshots[n];
        ENTITIES.iter().all(|&e| match self.ops[n][e] {
            StepOp::None => {
                before.live[e] == after.live[e]
                    && before.generation[e] == after.generation[e]
                    && before.value[e] == after.value[e]
            }
            StepOp::Set => {
                after.live[e] && before.live[e] && before.generation[e] == after.generation[e]
            }
            StepOp::Retired => before.live[e] && !after.live[e] && after.generation[e] == 0,
            StepOp::Born | StepOp::Restored => {
                !before.live[e]
                    && after.live[e]
                    && self.snapshots[..n]
                        .iter()
                        .all(|s| s.generation.iter().all(|g| *g < after.generation[e]))
            }
        })
    }

    /// `NoWedge` as the oracle can state it without `ENABLED`: an open writer
    /// holds no trees bound elsewhere, so it can always discard, and commit
    /// when the head is current and a key is free.
    fn no_wedge(&self) -> bool {
        self.writer.is_none() || (self.forest_origin == 0 && !self.forest_disposing)
    }

    /// Ruling (u): a reversal's fact is active exactly on the lines where its
    /// step is an ancestor of the selected step, or the selected step itself.
    /// Recomputed from the journal, so it checks `facts_after_move`.
    fn facts_follow_line(&self) -> bool {
        let line = self.line(self.current);
        self.facts == self.posted_facts().filter(|f| line.contains(&f.step)).collect()
    }

    /// Where a fact is active the snapshots agree its source is already
    /// reversed: the fact never refuses what the line allows.
    fn facts_agree_with_snapshots(&self) -> bool {
        self.facts
            .iter()
            .all(|f| self.meaning_derivable(f.source, f.entity) && !self.source_reversible(f.source, f.entity))
    }

    fn stopped_custody(&self) -> bool {
        self.parked.iter().all(|c| *c < self.next_candidate)
            && (self.forest_origin == 0 || self.parked.contains(&self.forest_origin))
            && (!self.forest_disposing || self.forest_origin != 0)
            && (!self.closed
                || (self.parked.is_empty() && self.forest_origin == 0 && self.writer.is_none()))
    }
}
