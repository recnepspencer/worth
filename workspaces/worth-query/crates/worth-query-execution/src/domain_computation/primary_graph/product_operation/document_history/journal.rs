//! User annotations and completed source bindings; Native and World own canonical ancestry.
use super::{
    allocation::HistoryInsertionAllocation,
    step::{DocumentIncarnation, WorthQueryHistoryNodeId, WorthQueryHistoryStep},
    WorthQueryDocumentHistoryDenial as Denial,
};
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationReadObservation,
    WorthQueryOwnedProductHead,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;

pub(super) struct RecordedNode {
    pub(super) step: WorthQueryHistoryStep,
    pub(super) _source: Arc<WorthQueryApplicationReadObservation>,
}

pub(super) enum NodeBinding {
    Reserved {
        parent: WorthQueryHistoryNodeId,
        label: Arc<str>,
    },
    Completed(RecordedNode),
}

pub(super) struct HistoryNode {
    pub(super) binding: NodeBinding,
    _allocation: HistoryInsertionAllocation,
}

pub(super) enum AttemptBinding {
    Reserved(WorthQueryHistoryNodeId),
    Completed(WorthQueryHistoryNodeId),
}

pub(super) struct HistoryAttempt {
    meaning: WorthQueryApplicationIdempotencyBinding,
    pub(super) binding: AttemptBinding,
    _allocation: HistoryInsertionAllocation,
}

pub(super) struct PastHead<'runtime, S> {
    pub(super) head: Option<WorthQueryOwnedProductHead<'runtime, S>>,
    _allocation: HistoryInsertionAllocation,
}

pub(super) struct ChildLink {
    _allocation: HistoryInsertionAllocation,
}

pub(in crate::domain_computation::primary_graph) struct HistoryJournal<'runtime, S> {
    incarnation: DocumentIncarnation,
    pub(super) current: WorthQueryHistoryNodeId,
    next: u64,
    next_transition: u64,
    pending: Option<WorthQueryHistoryNodeId>,
    pub(super) nodes: BTreeMap<u64, HistoryNode>,
    pub(super) attempts: BTreeMap<[u8; 32], HistoryAttempt>,
    pub(super) past_heads: BTreeMap<u64, PastHead<'runtime, S>>,
    // Rebuildable annotation index; its links confer no source or disclosure authority.
    pub(super) children: BTreeMap<(u64, u64), ChildLink>,
}

pub(super) enum AttemptLookup {
    Unseen,
    Reserved,
    Completed(WorthQueryHistoryNodeId),
    Drift,
}

pub(super) struct ReservedHistoryRecord {
    pub(super) node: WorthQueryHistoryNodeId,
    pub(super) parent: WorthQueryHistoryNodeId,
    pub(super) transition: u64,
    pub(super) key: [u8; 32],
}

impl<'runtime, S> HistoryJournal<'runtime, S> {
    pub(in crate::domain_computation::primary_graph) fn attach(
        execution: &ExecutionRequest<'_, '_>,
        runtime: u64,
        endpoint: WorthQueryProductBranchReadIdentity,
        source: Arc<WorthQueryApplicationReadObservation>,
    ) -> Result<Self, Denial> {
        let incarnation =
            DocumentIncarnation::mint(runtime).ok_or(Denial::RecordingIdentityExhausted)?;
        let current = incarnation.node(0);
        let allocation = HistoryInsertionAllocation::reserve::<(u64, HistoryNode)>(execution, 0, 0)
            .map_err(Denial::Recording)?;
        let mut nodes = BTreeMap::new();
        nodes.insert(
            0,
            HistoryNode {
                binding: NodeBinding::Completed(RecordedNode {
                    step: WorthQueryHistoryStep {
                        id: current,
                        parent: None,
                        label: None,
                        endpoint,
                    },
                    _source: source,
                }),
                _allocation: allocation,
            },
        );
        Ok(Self {
            incarnation,
            current,
            next: 1,
            next_transition: 0,
            pending: None,
            nodes,
            attempts: BTreeMap::new(),
            past_heads: BTreeMap::new(),
            children: BTreeMap::new(),
        })
    }

    pub(super) fn lookup(&self, meaning: WorthQueryApplicationIdempotencyBinding) -> AttemptLookup {
        match self.attempts.get(&meaning.canonical_key()) {
            None => AttemptLookup::Unseen,
            Some(attempt) if !attempt.meaning.has_same_intent(meaning) => AttemptLookup::Drift,
            Some(HistoryAttempt {
                binding: AttemptBinding::Reserved(_),
                ..
            }) => AttemptLookup::Reserved,
            Some(HistoryAttempt {
                binding: AttemptBinding::Completed(id),
                ..
            }) => AttemptLookup::Completed(*id),
        }
    }

    pub(super) fn reserve(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        meaning: WorthQueryApplicationIdempotencyBinding,
        label: &str,
    ) -> Result<ReservedHistoryRecord, Denial> {
        if self.pending.is_some() {
            return Err(Denial::AttemptInFlight);
        }
        let node = self.incarnation.node(self.next);
        let next = self
            .next
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        let transition = self.next_transition;
        let next_transition = transition
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        let key = meaning.canonical_key();
        // Every real tree slot is inserted before any candidate creation or operation effect.
        // Failure while acquiring the grants drops only new grants, leaving the journal intact.
        let node_allocation = HistoryInsertionAllocation::reserve::<(u64, HistoryNode)>(
            execution,
            self.nodes.len(),
            label.len(),
        )
        .map_err(Denial::Recording)?;
        let attempt_allocation = HistoryInsertionAllocation::reserve::<([u8; 32], HistoryAttempt)>(
            execution,
            self.attempts.len(),
            0,
        )
        .map_err(Denial::Recording)?;
        let head_allocation = HistoryInsertionAllocation::reserve::<(u64, PastHead<'runtime, S>)>(
            execution,
            self.past_heads.len(),
            0,
        )
        .map_err(Denial::Recording)?;
        let link_allocation = HistoryInsertionAllocation::reserve::<((u64, u64), ChildLink)>(
            execution,
            self.children.len(),
            0,
        )
        .map_err(Denial::Recording)?;
        let label = Arc::from(label);
        assert!(
            !self.attempts.contains_key(&key),
            "only an unseen attempt reserves storage"
        );
        self.nodes.insert(
            node.ordinal,
            HistoryNode {
                binding: NodeBinding::Reserved {
                    parent: self.current,
                    label,
                },
                _allocation: node_allocation,
            },
        );
        self.attempts.insert(
            key,
            HistoryAttempt {
                meaning,
                binding: AttemptBinding::Reserved(node),
                _allocation: attempt_allocation,
            },
        );
        self.past_heads.insert(
            transition,
            PastHead {
                head: None,
                _allocation: head_allocation,
            },
        );
        self.children.insert(
            (self.current.ordinal, node.ordinal),
            ChildLink {
                _allocation: link_allocation,
            },
        );
        self.next = next;
        self.next_transition = next_transition;
        self.pending = Some(node);
        Ok(ReservedHistoryRecord {
            node,
            parent: self.current,
            transition,
            key,
        })
    }

    pub(super) fn cancel(&mut self, reserved: ReservedHistoryRecord) {
        self.nodes.remove(&reserved.node.ordinal);
        self.attempts.remove(&reserved.key);
        self.children
            .remove(&(reserved.parent.ordinal, reserved.node.ordinal));
        let head = self
            .past_heads
            .remove(&reserved.transition)
            .expect("reserved custody slot");
        assert!(head.head.is_none(), "an adopted head cannot be cancelled");
        self.pending = None;
    }

    /// All lookup slots already exist. Invoke only while the same exclusive
    /// document owner performs its genuine head adoption; this cannot allocate.
    pub(super) fn complete(
        &mut self,
        reserved: ReservedHistoryRecord,
        completed: RecordedNode,
        previous: WorthQueryOwnedProductHead<'runtime, S>,
    ) {
        self.nodes
            .get_mut(&reserved.node.ordinal)
            .expect("reserved node")
            .binding = NodeBinding::Completed(completed);
        self.attempts
            .get_mut(&reserved.key)
            .expect("reserved attempt")
            .binding = AttemptBinding::Completed(reserved.node);
        self.past_heads
            .get_mut(&reserved.transition)
            .expect("reserved head custody")
            .head = Some(previous);
        self.current = reserved.node;
        self.pending = None;
    }

    pub(super) fn step(&self, id: WorthQueryHistoryNodeId) -> Option<&WorthQueryHistoryStep> {
        if id.document != self.incarnation {
            return None;
        }
        match &self.nodes.get(&id.ordinal)?.binding {
            NodeBinding::Completed(node) => Some(&node.step),
            NodeBinding::Reserved { .. } => None,
        }
    }

    pub(super) fn matches_reservation(&self, reserved: &ReservedHistoryRecord) -> bool {
        self.pending == Some(reserved.node)
            && self.current == reserved.parent
            && matches!(self.attempts.get(&reserved.key),
                Some(HistoryAttempt { binding: AttemptBinding::Reserved(node), .. }) if *node == reserved.node)
    }

    pub(super) fn prepared_step(
        &self,
        reserved: &ReservedHistoryRecord,
        endpoint: WorthQueryProductBranchReadIdentity,
    ) -> WorthQueryHistoryStep {
        let NodeBinding::Reserved { parent, label } = &self
            .nodes
            .get(&reserved.node.ordinal)
            .expect("reserved node")
            .binding
        else {
            unreachable!("only the original pending node can complete")
        };
        WorthQueryHistoryStep {
            id: reserved.node,
            parent: Some(*parent),
            label: Some(Arc::clone(label)),
            endpoint,
        }
    }
}
