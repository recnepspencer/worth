use crate::data::temporal::{
    TemporalRetiredWakeCompactionReport, TemporalWakeId, TemporalWakeOwner,
};

use super::TemporalRuntimeState;

impl TemporalRuntimeState {
    pub(in crate::logic::transaction::runtime) fn retired_wake_storage_width_for_test(
        &self,
    ) -> usize {
        self.retired_wakes.len()
    }

    pub(in crate::logic::transaction::runtime) fn retire_resource_retry_wake(
        &mut self,
        node: crate::data::resource::ResourceNodeId,
        wake: TemporalWakeId,
        telemetry: Option<&mut crate::data::telemetry::TemporalTelemetry>,
    ) -> Result<crate::data::temporal::RetiredTemporalWake, crate::data::error::SignalError> {
        let owner = TemporalWakeOwner::ResourceNode(node.node());
        if let Some(retired) = self
            .retired_wakes
            .get(&wake)
            .filter(|value| value.owner() == owner)
        {
            return Ok(retired.clone());
        }
        if self.active_wake_owner(wake) != Some(owner) {
            return Err(crate::data::error::SignalError::invalid_input(
                "retry wake is not owned by this resource",
            ));
        }
        self.retire_wake(
            wake,
            crate::data::temporal::TemporalWakeRetirementReason::Disposed,
            telemetry,
        )
    }

    pub(in crate::logic::transaction::runtime) fn can_release_resource_wake(
        &self,
        node: crate::data::resource::ResourceNodeId,
        wake: TemporalWakeId,
    ) -> bool {
        !self.scheduled_wakes.contains_key(&wake)
            && !self.ready_wakes.contains_key(&wake)
            && self.retired_wakes.get(&wake).is_none_or(|retired| {
                retired.owner() == TemporalWakeOwner::ResourceNode(node.node())
            })
    }

    pub(in crate::logic::transaction::runtime) fn release_resource_wake(
        &mut self,
        wake: TemporalWakeId,
    ) {
        self.expire_retired_wake(wake);
    }

    /// Managed conditional partitions return retirement evidence to their caller
    /// without retaining an unbounded reconstruction history in the ordinary lane.
    pub(crate) fn forget_retired_wake(&mut self, wake_id: TemporalWakeId) {
        self.retired_wakes.remove(&wake_id);
    }

    pub(crate) fn compact_retired_wake_tail(
        &mut self,
        retained_limit: u32,
        max_expired: u32,
    ) -> TemporalRetiredWakeCompactionReport {
        let mut expired_now = 0_u32;
        while self.retired_wakes.len() > retained_limit as usize && expired_now < max_expired {
            let Some(id) = self.retired_wakes.keys().next().copied() else {
                break;
            };
            if self.expire_retired_wake(id) {
                expired_now += 1;
            }
        }
        TemporalRetiredWakeCompactionReport::new(
            expired_now,
            self.retired_wakes.len().min(u32::MAX as usize) as u32,
            self.expired_retired_wakes.count(),
            self.expired_retired_wakes.digest(),
        )
    }

    fn expire_retired_wake(&mut self, id: TemporalWakeId) -> bool {
        let Some(receipt) = self.retired_wakes.remove(&id) else {
            return false;
        };
        self.expired_retired_wakes
            .absorb(b"temporal-retired-wake", id.get(), &receipt);
        true
    }
}
