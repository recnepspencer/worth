//! Immutable complete selected-root aggregate truth and its original pure-read payer.
use super::*;
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionMemoryReservation};
use worth_foundational::facade::{AggregateDefinitionIdentity, AggregateIdentity};

pub(super) struct Value {
    pub(super) definition: AggregateDefinitionIdentity,
    pub(super) identity: AggregateIdentity,
    pub(super) members: BTreeSet<NativeMemberKey>,
    pub(super) lineage: BTreeSet<super::super::AggregateLineageEvent>,
}
impl ChargedBytes for Value {
    fn additional_charged_bytes(&self) -> u64 {
        (self.members.len() as u64)
            .saturating_mul(extent::tree_entry::<NativeMemberKey, ()>())
            .saturating_add(
                (self.lineage.len() as u64)
                    .saturating_mul(
                        extent::tree_entry::<super::super::AggregateLineageEvent, ()>(),
                    ),
            )
            .saturating_add(controls())
    }
}

struct Data {
    value: Value,
    reads: AggregateReadObservations,
    work: u64,
    _retention: Option<ExecutionMemoryReservation>,
}

/// Complete canonical membership and stored incident lineage from one actual
/// pinned read. Membership is identity-sorted, not successor order. Clones keep
/// the same original grant; no runtime/request or private graph IDs are exposed.
#[derive(Clone)]
pub struct PinnedAggregateRead<Projection = EntityId>(
    Arc<Data>,
    std::marker::PhantomData<fn() -> Projection>,
);

pub(super) fn controls() -> u64 {
    extent::arc_backing::<Data>()
}

impl<Projection> PinnedAggregateRead<Projection> {
    pub(super) fn finish(
        value: Value,
        reads: AggregateReadObservations,
        work: u64,
        retention: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self(
            Arc::new(Data {
                value,
                reads,
                work,
                _retention: retention,
            }),
            std::marker::PhantomData,
        )
    }
    pub fn definition(&self) -> AggregateDefinitionIdentity {
        self.0.value.definition
    }
    pub fn identity(&self) -> AggregateIdentity {
        self.0.value.identity
    }
    pub fn len(&self) -> usize {
        self.0.value.members.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    /// All actual stored events incident to this aggregate, in canonical tuple
    /// order. This is neither publication order nor a latest-identity answer.
    pub fn lineage(
        &self,
    ) -> impl ExactSizeIterator<Item = super::super::AggregateReadLineageEvent<'_>> {
        self.0
            .value
            .lineage
            .iter()
            .map(super::super::AggregateReadLineageEvent::new)
    }
    pub fn reads(&self) -> AggregateReadObservations {
        self.0.reads
    }
    /// Logical read-checkpoint units reached; not an execution billing report.
    pub fn work_units(&self) -> u64 {
        self.0.work
    }
}
impl<Projection> std::fmt::Debug for PinnedAggregateRead<Projection> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedAggregateRead")
            .field("definition", &self.definition())
            .field("identity", &self.identity())
            .field("anchor", &self.0.value.members.first())
            .field("members", &self.len())
            .field("lineage_events", &self.0.value.lineage.len())
            .field("reads", &self.0.reads)
            .finish()
    }
}

impl PinnedAggregateRead<EntityId> {
    pub fn anchor(&self) -> EntityId {
        self.0
            .value
            .members
            .first()
            .and_then(NativeMemberKey::entity)
            .expect("physical pin scope validates entity source")
    }
    pub fn members(&self) -> impl ExactSizeIterator<Item = EntityId> + '_ {
        self.0.value.members.iter().map(|member| {
            member
                .entity()
                .expect("physical pin scope validates entity source")
        })
    }
    pub fn contains(&self, member: EntityId) -> bool {
        self.0
            .value
            .members
            .contains(&NativeMemberKey::Entity(member))
    }
}
impl PinnedAggregateRead<NativeMemberKey> {
    pub fn anchor(&self) -> &NativeMemberKey {
        self.0
            .value
            .members
            .first()
            .expect("complete projected cohort")
    }
    pub fn members(&self) -> impl ExactSizeIterator<Item = &NativeMemberKey> {
        self.0.value.members.iter()
    }
    pub fn contains(&self, member: &NativeMemberKey) -> bool {
        self.0.value.members.contains(member)
    }
}
