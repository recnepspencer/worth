//! Reserve only the actual durable flights while acquiring the physical W root.
use super::*;

pub(crate) struct WorldCaptureReservation {
    binding: Arc<WorldDurableMediumBinding>,
}
impl WorldDurableMediumBinding {
    pub(crate) fn reserve_checkpoint_capture(
        self: &Arc<Self>,
    ) -> Result<WorldCaptureReservation, WorldDurableMediumDenial> {
        let mut gates = self
            .gates
            .try_lock()
            .map_err(|_| WorldDurableMediumDenial::PublicationBusy)?;
        if gates.capture_reserved {
            return Err(WorldDurableMediumDenial::PublicationBusy);
        }
        for flight in [&gates.branches, &gates.custody, &gates.occurrences] {
            if flight.in_caller {
                return Err(WorldDurableMediumDenial::PublicationBusy);
            }
            if flight.subject.is_some() || flight.barrier.is_some() {
                return Err(WorldDurableMediumDenial::PublicationUnresolved);
            }
        }
        match self.medium.publication_task_posture() {
            super::super::WorldPublicationTaskPosture::Busy => {
                return Err(WorldDurableMediumDenial::PublicationBusy)
            }
            super::super::WorldPublicationTaskPosture::Unsettled => {
                return Err(WorldDurableMediumDenial::PublicationUnresolved)
            }
            super::super::WorldPublicationTaskPosture::Terminal => {}
        }
        gates.capture_reserved = true;
        Ok(WorldCaptureReservation {
            binding: self.clone(),
        })
    }
}
impl Drop for WorldCaptureReservation {
    fn drop(&mut self) {
        self.binding
            .gates
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .capture_reserved = false;
    }
}
