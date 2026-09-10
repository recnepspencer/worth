use super::{BridgeManagedClockLane, BridgeManagedTemporalDenial, BridgeManagedTemporalDenialKind};
use std::sync::{Mutex, MutexGuard};

pub(in crate::conditional_execution) fn lock_lane(
    cell: &Mutex<BridgeManagedClockLane>,
) -> Result<MutexGuard<'_, BridgeManagedClockLane>, BridgeManagedTemporalDenial> {
    let lane = match cell.lock() {
        Ok(lane) => lane,
        Err(poison) => {
            poison.into_inner().quarantined = true;
            return Err(denied());
        }
    };
    lane.require_usable()?;
    Ok(lane)
}

impl BridgeManagedClockLane {
    pub(super) fn require_usable(&self) -> Result<(), BridgeManagedTemporalDenial> {
        if self.quarantined {
            Err(denied())
        } else {
            Ok(())
        }
    }

    pub(super) fn begin_effect(&mut self) -> Result<(), BridgeManagedTemporalDenial> {
        self.require_usable()?;
        // Remains set on unwind, internal divergence, and any incomplete join.
        self.quarantined = true;
        Ok(())
    }

    pub(super) fn admit_signal_result<T>(
        &mut self,
        result: Result<T, worth_signal::facade::branch::SignalConditionalTemporalPartitionDenial>,
    ) -> Result<T, BridgeManagedTemporalDenial> {
        match result {
            Ok(value) => Ok(value),
            Err(worth_signal::facade::branch::SignalConditionalTemporalPartitionDenial::PartitionQuarantined) => Err(denied()),
            Err(error) => {
                // Signal's other denials prove that no effect was published.
                self.quarantined = false;
                Err(super::clock_lane::signal_denial(error))
            }
        }
    }

    pub(super) fn finish_effect(&mut self) {
        self.quarantined = false;
    }

    pub(super) fn after_signal_before_publication(
        &mut self,
    ) -> Result<(), BridgeManagedTemporalDenial> {
        #[cfg(test)]
        match self.fault.take() {
            Some(Fault::Divergence) => return Err(denied()),
            Some(Fault::Unwind) => panic!("injected Bridge fault after Signal movement"),
            None => {}
        }
        Ok(())
    }
}

pub(super) fn denied() -> BridgeManagedTemporalDenial {
    BridgeManagedTemporalDenial::new(
        BridgeManagedTemporalDenialKind::LaneQuarantined,
        "managed clock lane is permanently quarantined after incomplete cross-owner publication",
    )
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum Fault {
    Divergence,
    Unwind,
}
