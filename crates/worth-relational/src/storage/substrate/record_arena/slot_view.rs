use std::marker::PhantomData;

use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::storage::data::RecordLifecycleState;

use super::{RecordArena, RecordKind};

pub(crate) struct SlotView<'a, K: RecordKind> {
    arena: &'a RecordArena<K>,
    physical: usize,
    _marker: PhantomData<K>,
}

impl<'a, K: RecordKind> SlotView<'a, K> {
    pub(crate) fn new(arena: &'a RecordArena<K>, physical: usize) -> Self {
        Self {
            arena,
            physical,
            _marker: PhantomData,
        }
    }

    pub(crate) fn generation(&self) -> u32 {
        self.arena.generations[self.physical]
    }
    pub(crate) fn partition_id(&self) -> PartitionId {
        self.arena.partition_ids[self.physical]
    }
    pub(crate) fn lifecycle(&self) -> RecordLifecycleState {
        self.arena.lifecycle[self.physical]
    }
    pub(crate) fn is_live(&self) -> bool {
        self.lifecycle() == RecordLifecycleState::Live
    }
    pub(crate) fn is_materialization_unavailable(&self) -> bool {
        self.lifecycle() == RecordLifecycleState::MaterializationUnavailable
    }
    pub(crate) fn kind_id(&self) -> Option<KindId> {
        self.arena.kind_ids[self.physical]
    }
    pub(crate) fn retired_at(&self) -> Option<VersionId> {
        self.arena.retired_at[self.physical]
    }
    pub(crate) fn extra(&self) -> &K::Extra {
        &self.arena.extra[self.physical]
    }
}
