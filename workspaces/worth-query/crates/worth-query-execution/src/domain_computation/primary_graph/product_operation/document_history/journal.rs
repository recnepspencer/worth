//! User annotations and completed source bindings; Native and World own canonical ancestry.
use super::{
    allocation::HistoryInsertionAllocation,
    observation::{Access, EnrollmentSink, WorthQueryHistoryEnrollmentObservationForTest},
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
#[path = "journal/group.rs"]
mod group;
#[path = "journal/navigation.rs"]
mod navigation;
#[path = "journal/paging.rs"]
mod paging;
#[path = "journal/preview.rs"]
mod preview;

pub(super) struct RecordedNode {
    pub(super) step: WorthQueryHistoryStep,
    pub(super) _source: Arc<WorthQueryApplicationReadObservation>,
    pub(super) root_posture:
        Option<crate::domain_computation::primary_graph::WorthQueryOutputSettlementPosture>,
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

#[derive(Clone, Copy)]
enum PendingRecord {
    Ordinary(WorthQueryHistoryNodeId),
    Group(WorthQueryHistoryNodeId),
}
impl PendingRecord {
    fn node(self) -> WorthQueryHistoryNodeId {
        match self {
            Self::Ordinary(node) | Self::Group(node) => node,
        }
    }
}

pub(in crate::domain_computation::primary_graph) struct HistoryJournal<'runtime, S> {
    incarnation: DocumentIncarnation,
    current: WorthQueryHistoryNodeId,
    next: u64,
    next_transition: u64,
    view_generation: u64,
    completed_end: u64,
    pending: Option<PendingRecord>,
    nodes: BTreeMap<u64, HistoryNode>,
    attempts: BTreeMap<[u8; 32], HistoryAttempt>,
    past_heads: BTreeMap<u64, PastHead<'runtime, S>>,
    // Rebuildable annotation index; its links confer no source or disclosure authority.
    children: BTreeMap<(u64, u64), ChildLink>,
    observation: Option<EnrollmentSink>,
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
    view_after: u64,
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
        let (allocation, _) =
            HistoryInsertionAllocation::reserve::<(u64, HistoryNode)>(execution, 0, 0)
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
                    root_posture: None,
                }),
                _allocation: allocation,
            },
        );
        Ok(Self {
            incarnation,
            current,
            next: 1,
            next_transition: 0,
            view_generation: 0,
            completed_end: 1,
            pending: None,
            nodes,
            attempts: BTreeMap::new(),
            past_heads: BTreeMap::new(),
            children: BTreeMap::new(),
            observation: None,
        })
    }

    pub(super) fn lookup(&self, meaning: WorthQueryApplicationIdempotencyBinding) -> AttemptLookup {
        self.access(Access::Attempt);
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
        self.access(Access::Selection);
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
        let view_after = self
            .view_generation
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        // Every real tree slot is inserted before any candidate creation or operation effect.
        // Failure while acquiring the grants drops only new grants, leaving the journal intact.
        let node_allocation = self
            .allocate::<(u64, HistoryNode)>(execution, self.nodes.len(), label.len())
            .map_err(Denial::Recording)?;
        let attempt_allocation = self
            .allocate::<([u8; 32], HistoryAttempt)>(execution, self.attempts.len(), 0)
            .map_err(Denial::Recording)?;
        let head_allocation = self
            .allocate::<(u64, PastHead<'runtime, S>)>(execution, self.past_heads.len(), 0)
            .map_err(Denial::Recording)?;
        let link_allocation = self
            .allocate::<((u64, u64), ChildLink)>(execution, self.children.len(), 0)
            .map_err(Denial::Recording)?;
        let label = Arc::from(label);
        self.access(Access::Attempt);
        assert!(
            !self.attempts.contains_key(&key),
            "only an unseen attempt reserves storage"
        );
        self.access(Access::Node);
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
        self.access(Access::Attempt);
        self.attempts.insert(
            key,
            HistoryAttempt {
                meaning,
                binding: AttemptBinding::Reserved(node),
                _allocation: attempt_allocation,
            },
        );
        self.access(Access::Custody);
        self.past_heads.insert(
            transition,
            PastHead {
                head: None,
                _allocation: head_allocation,
            },
        );
        self.access(Access::Child);
        self.children.insert(
            (self.current.ordinal, node.ordinal),
            ChildLink {
                _allocation: link_allocation,
            },
        );
        self.next = next;
        self.next_transition = next_transition;
        self.pending = Some(PendingRecord::Ordinary(node));
        Ok(ReservedHistoryRecord {
            node,
            parent: self.current,
            transition,
            key,
            view_after,
        })
    }

    pub(super) fn cancel(&mut self, reserved: ReservedHistoryRecord) {
        self.access(Access::Node);
        self.nodes.remove(&reserved.node.ordinal);
        self.access(Access::Attempt);
        self.attempts.remove(&reserved.key);
        self.access(Access::Child);
        self.children
            .remove(&(reserved.parent.ordinal, reserved.node.ordinal));
        self.access(Access::Custody);
        let head = self
            .past_heads
            .remove(&reserved.transition)
            .expect("reserved custody slot");
        assert!(head.head.is_none(), "an adopted head cannot be cancelled");
        self.access(Access::Selection);
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
        self.access(Access::Node);
        self.nodes
            .get_mut(&reserved.node.ordinal)
            .expect("reserved node")
            .binding = NodeBinding::Completed(completed);
        self.access(Access::Attempt);
        self.attempts
            .get_mut(&reserved.key)
            .expect("reserved attempt")
            .binding = AttemptBinding::Completed(reserved.node);
        self.access(Access::Custody);
        self.past_heads
            .get_mut(&reserved.transition)
            .expect("reserved head custody")
            .head = Some(previous);
        self.access(Access::Selection);
        self.current = reserved.node;
        self.completed_end = reserved.node.ordinal + 1; // Successor was admitted by reserve.
        self.view_generation = reserved.view_after;
        self.pending = None;
    }

    pub(super) fn step(&self, id: WorthQueryHistoryNodeId) -> Option<&WorthQueryHistoryStep> {
        self.access(Access::Selection);
        if id.document != self.incarnation {
            return None;
        }
        self.access(Access::Node);
        match &self.nodes.get(&id.ordinal)?.binding {
            NodeBinding::Completed(node) => Some(&node.step),
            NodeBinding::Reserved { .. } => None,
        }
    }

    pub(super) fn matches_reservation(&self, reserved: &ReservedHistoryRecord) -> bool {
        self.access(Access::Selection);
        self.pending
            .is_some_and(|pending| pending.node() == reserved.node)
            && self.current == reserved.parent
            && matches!({ self.access(Access::Attempt); self.attempts.get(&reserved.key) },
                Some(HistoryAttempt { binding: AttemptBinding::Reserved(node), .. }) if *node == reserved.node)
    }

    pub(super) fn prepared_step(
        &self,
        reserved: &ReservedHistoryRecord,
        endpoint: WorthQueryProductBranchReadIdentity,
    ) -> WorthQueryHistoryStep {
        self.access(Access::Node);
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

    pub(super) fn current_step(&self) -> &WorthQueryHistoryStep {
        self.access(Access::Selection);
        self.step(self.current)
            .expect("the selected user step completed")
    }

    pub(super) fn into_disposal(
        self,
    ) -> std::collections::btree_map::IntoValues<u64, PastHead<'runtime, S>> {
        drop(self.nodes);
        self.past_heads.into_values()
    }

    pub(super) fn observe(&mut self) -> WorthQueryHistoryEnrollmentObservationForTest {
        let (observer, sink) = WorthQueryHistoryEnrollmentObservationForTest::new();
        self.observation = Some(sink);
        observer
    }

    fn access(&self, access: Access) {
        if let Some(observation) = &self.observation {
            observation.access(access);
        }
    }

    fn allocate<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
        label_bytes: usize,
    ) -> Result<
        HistoryInsertionAllocation,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    > {
        let (allocation, requested) =
            HistoryInsertionAllocation::reserve::<T>(execution, len, label_bytes)?;
        if let Some(observation) = &self.observation {
            observation.backing(requested, allocation.ticket_bytes());
        }
        Ok(allocation)
    }
}
