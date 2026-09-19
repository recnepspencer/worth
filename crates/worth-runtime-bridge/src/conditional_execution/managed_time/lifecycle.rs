use std::sync::{Arc, Mutex};

use super::{
    BridgeManagedClockBinding, BridgeManagedClockClosure, BridgeManagedClockLane,
    BridgeManagedTemporalDenial, BridgeManagedTemporalDenialKind,
};
use crate::conditional_execution::BridgeOwnedSignalRuntime;

pub(super) fn require_clock_lane(
    lane: &BridgeManagedClockLane,
    binding: &BridgeManagedClockBinding,
) -> Result<(), BridgeManagedTemporalDenial> {
    require_clock_affinity(lane, binding)?;
    lane.require_usable()
}

fn require_clock_affinity(
    lane: &BridgeManagedClockLane,
    binding: &BridgeManagedClockBinding,
) -> Result<(), BridgeManagedTemporalDenial> {
    if !Arc::ptr_eq(&lane.lease, &binding.lease) {
        return Err(foreign_binding_denial());
    }
    if !binding.is_live() {
        return Err(closed_binding_denial());
    }
    Ok(())
}

impl BridgeOwnedSignalRuntime {
    pub(in crate::conditional_execution) fn admit_managed_clock_lane(
        &self,
        binding: &BridgeManagedClockBinding,
    ) -> Result<Arc<Mutex<BridgeManagedClockLane>>, BridgeManagedTemporalDenial> {
        if binding.bridge_runtime_key != self.bridge.signal_runtime_key {
            return Err(foreign_binding_denial());
        }
        if !binding.is_live() {
            return Err(closed_binding_denial());
        }
        self.managed_clock_lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&binding.binding_identity)
            .cloned()
            .ok_or_else(foreign_binding_denial)
    }

    pub fn close_managed_clock(
        &self,
        binding: BridgeManagedClockBinding,
    ) -> Result<BridgeManagedClockClosure, BridgeManagedTemporalDenial> {
        let retained = self.admit_managed_clock_lane(&binding)?;
        let lane = retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        require_clock_affinity(&lane, &binding)?;
        let (active_intents, scheduled_wakes, ready_wakes) = lane.closure_counts()?;
        lane.lease.revoke();
        drop(lane);
        let mut lanes = self
            .managed_clock_lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if lanes
            .get(&binding.binding_identity)
            .is_some_and(|installed| Arc::ptr_eq(installed, &retained))
        {
            lanes.remove(&binding.binding_identity);
        }
        let empty_backing = if lanes.is_empty() {
            Some(std::mem::take(&mut *lanes))
        } else {
            None
        };
        drop(lanes);
        drop(empty_backing);
        Ok(BridgeManagedClockClosure::new(
            active_intents,
            scheduled_wakes,
            ready_wakes,
        ))
    }

    pub(in crate::conditional_execution) fn revoke_managed_clock_liveness(&self) {
        let lanes = self
            .managed_clock_lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for lane in lanes {
            lane.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .lease
                .revoke();
        }
    }

    pub fn managed_clock_count(&self) -> usize {
        self.managed_clock_lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}

fn foreign_binding_denial() -> BridgeManagedTemporalDenial {
    BridgeManagedTemporalDenial::new(
        BridgeManagedTemporalDenialKind::ForeignClockBinding,
        "managed clock binding belongs to another Bridge runtime or installation",
    )
}
fn closed_binding_denial() -> BridgeManagedTemporalDenial {
    BridgeManagedTemporalDenial::new(
        BridgeManagedTemporalDenialKind::ClosedClockBinding,
        "managed clock binding is no longer installed",
    )
}
