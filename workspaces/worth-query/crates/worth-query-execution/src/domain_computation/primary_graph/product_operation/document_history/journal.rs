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
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};
use worth_execution::ExecutionRequest;
#[path = "journal/group.rs"]
mod group;
#[path = "journal/heads.rs"]
mod heads;
#[path = "journal/navigation.rs"]
mod navigation;
#[path = "journal/paging.rs"]
mod paging;
#[path = "journal/preview.rs"]
mod preview;
#[path = "journal/recording.rs"]
mod recording;
#[path = "journal/retention.rs"]
mod retention;
#[path = "journal/retention_work.rs"]
mod retention_work;

pub(super) struct RecordedNode {
    pub(super) step: WorthQueryHistoryStep,
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
    head_slot: Option<u64>,
    _allocation: HistoryInsertionAllocation,
}

pub(super) struct RetainedHistorySource {
    source: Option<Arc<WorthQueryApplicationReadObservation>>,
    bookmarked: bool,
    _allocation: HistoryInsertionAllocation,
    released_child: Option<ChildLink>,
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
    pub(super) custody: Arc<super::head_custody::HistoryHeadSlot<'runtime, S>>,
    queued: bool,
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
    retained: BTreeMap<u64, RetainedHistorySource>,
    retained_completed: usize,
    bookmarked: usize,
    window: worth_query_declaration::facade::application_history::ApplicationHistoryRecentSteps,
    pending_window:
        Option<worth_query_declaration::facade::application_history::ApplicationHistoryRecentSteps>,
    retention_resources:
        worth_query_declaration::facade::application_history::ApplicationHistoryRetentionResources,
    releases: VecDeque<u64>,
    _release_memory: Option<worth_execution::ExecutionMemoryReservation>,
    navigation_sources: Arc<super::navigation::PendingNavigationCapacity>,
    closed_heads: usize,
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
    pub(super) fn document_identity(&self) -> super::WorthQueryDocumentHistoryIdentity {
        super::WorthQueryDocumentHistoryIdentity(self.incarnation)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn attach(
        execution: &ExecutionRequest<'_, '_>,
        runtime: u64,
        endpoint: WorthQueryProductBranchReadIdentity,
        source: Arc<WorthQueryApplicationReadObservation>,
        retention_resources: worth_query_declaration::facade::application_history::ApplicationHistoryRetentionResources,
        navigation_sources: Arc<super::navigation::PendingNavigationCapacity>,
    ) -> Result<Self, Denial> {
        let incarnation =
            DocumentIncarnation::mint(runtime).ok_or(Denial::RecordingIdentityExhausted)?;
        let current = incarnation.node(0);
        let (allocation, _) =
            HistoryInsertionAllocation::reserve::<(u64, HistoryNode)>(execution, 0, 0)
                .map_err(Denial::Recording)?;
        let mut nodes = BTreeMap::new();
        let (source_allocation, _) =
            HistoryInsertionAllocation::reserve::<(u64, RetainedHistorySource)>(execution, 0, 0)
                .map_err(Denial::Recording)?;
        let release_capacity = retention_resources.maximum_pending_releases.get();
        let release_bytes = release_capacity
            .checked_next_power_of_two()
            .and_then(|capacity| capacity.checked_mul(size_of::<u64>()))
            .ok_or(Denial::RecordingIdentityExhausted)?;
        let release_memory = super::allocation::reserve_buffer(execution, release_bytes)
            .map_err(Denial::Recording)?;
        let mut releases = VecDeque::new();
        releases
            .try_reserve_exact(release_capacity)
            .map_err(Denial::Allocation)?;
        let retained = BTreeMap::from([(
            0,
            RetainedHistorySource {
                source: Some(source),
                bookmarked: false,
                _allocation: source_allocation,
                released_child: None,
            },
        )]);
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
                    root_posture: None,
                }),
                head_slot: None,
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
            retained,
            retained_completed: 1,
            bookmarked: 0,
            window: worth_query_declaration::facade::application_history::ApplicationHistoryRecentSteps::DEFAULT,
            pending_window: None,
            retention_resources,
            releases,
            _release_memory: release_memory,
            navigation_sources,
            closed_heads: 0,
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

    pub(in crate::domain_computation::primary_graph) fn view_generation(&self) -> u64 {
        self.view_generation
    }

    pub(super) fn into_disposal(
        self,
    ) -> (
        usize,
        std::collections::btree_map::IntoValues<u64, PastHead<'runtime, S>>,
    ) {
        drop(self.nodes);
        drop(self.retained);
        (self.closed_heads, self.past_heads.into_values())
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
