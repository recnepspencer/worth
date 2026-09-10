use crate::data::temporal::{
    PreviousValueRevision, ReadyTemporalWake, RetiredTemporalWake, RuntimeClockBasis,
    ScheduledTemporalWake, TemporalWakeId, TemporalWakeOwner, WakeOrdinal,
};

mod admission;
mod clock;
#[cfg(test)]
mod fork_cost_tests;
mod frontier;
mod lifecycle;
mod observation;
mod previous_value;
mod receipt_retention;
mod restoration;
mod retirement;

/// Runtime-owned temporal state for authoritative clock basis semantics and wake lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemporalRuntimeState {
    pub(super) clock_basis: RuntimeClockBasis,
    pub(super) previous_value_capability_epoch: u64,
    pub(super) next_wake_id: TemporalWakeId,
    pub(super) next_wake_ordinal: WakeOrdinal,
    pub(super) next_previous_value_revision: PreviousValueRevision,
    pub(super) scheduled_wakes:
        crate::data::persistent_ord_map::PersistentOrdMap<TemporalWakeId, ScheduledTemporalWake>,
    pub(super) scheduled_frontier: crate::data::persistent_ord_map::PersistentOrdMap<
        crate::data::temporal::ClockTick,
        im::OrdMap<WakeOrdinal, TemporalWakeId>,
    >,
    pub(super) ready_wakes:
        crate::data::persistent_ord_map::PersistentOrdMap<TemporalWakeId, ReadyTemporalWake>,
    pub(super) ready_frontier:
        crate::data::persistent_ord_map::PersistentOrdMap<WakeOrdinal, TemporalWakeId>,
    pub(super) owner_frontier: crate::data::persistent_ord_map::PersistentOrdMap<
        TemporalWakeOwner,
        im::OrdMap<WakeOrdinal, TemporalWakeId>,
    >,
    pub(super) retired_wakes:
        crate::data::persistent_ord_map::PersistentOrdMap<TemporalWakeId, RetiredTemporalWake>,
}

impl Default for TemporalRuntimeState {
    fn default() -> Self {
        Self {
            clock_basis: RuntimeClockBasis::default(),
            previous_value_capability_epoch: 0,
            next_wake_id: TemporalWakeId::new(0),
            next_wake_ordinal: WakeOrdinal::ZERO,
            next_previous_value_revision: PreviousValueRevision::ZERO,
            scheduled_wakes: Default::default(),
            scheduled_frontier: Default::default(),
            ready_wakes: Default::default(),
            ready_frontier: Default::default(),
            owner_frontier: Default::default(),
            retired_wakes: Default::default(),
        }
    }
}

impl TemporalRuntimeState {
    pub(in crate::logic::transaction::runtime) fn fork_persistent(&mut self) -> Self {
        Self {
            clock_basis: self.clock_basis,
            previous_value_capability_epoch: self.previous_value_capability_epoch,
            next_wake_id: self.next_wake_id,
            next_wake_ordinal: self.next_wake_ordinal,
            next_previous_value_revision: self.next_previous_value_revision,
            scheduled_wakes: self.scheduled_wakes.fork_persistent(),
            scheduled_frontier: self.scheduled_frontier.fork_persistent(),
            ready_wakes: self.ready_wakes.fork_persistent(),
            ready_frontier: self.ready_frontier.fork_persistent(),
            owner_frontier: self.owner_frontier.fork_persistent(),
            retired_wakes: self.retired_wakes.fork_persistent(),
        }
    }
}

#[cfg(test)]
impl TemporalRuntimeState {
    pub(super) fn fork_storage_identity(&self) -> Self {
        Self {
            clock_basis: self.clock_basis,
            previous_value_capability_epoch: self.previous_value_capability_epoch,
            next_wake_id: self.next_wake_id,
            next_wake_ordinal: self.next_wake_ordinal,
            next_previous_value_revision: self.next_previous_value_revision,
            scheduled_wakes: self.scheduled_wakes.fork_storage_identity(),
            scheduled_frontier: self.scheduled_frontier.fork_storage_identity(),
            ready_wakes: self.ready_wakes.fork_storage_identity(),
            ready_frontier: self.ready_frontier.fork_storage_identity(),
            owner_frontier: self.owner_frontier.fork_storage_identity(),
            retired_wakes: self.retired_wakes.fork_storage_identity(),
        }
    }

    pub(super) fn shares_storage_with(&self, other: &Self) -> bool {
        self.scheduled_wakes.ptr_eq(&other.scheduled_wakes)
            && self.scheduled_frontier.ptr_eq(&other.scheduled_frontier)
            && self.ready_wakes.ptr_eq(&other.ready_wakes)
            && self.ready_frontier.ptr_eq(&other.ready_frontier)
            && self.owner_frontier.ptr_eq(&other.owner_frontier)
            && self.retired_wakes.ptr_eq(&other.retired_wakes)
    }
}
