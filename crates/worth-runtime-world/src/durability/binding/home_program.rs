//! One original Home binding handoff, without retaining the journal gate lock.
use super::*;
pub(crate) struct HomeProgramBindingReservation<'owner> {
    owner: &'owner WorldDurableMediumBinding,
}
impl WorldDurableMediumBinding {
    pub(crate) fn reserve_home_program_binding(
        &self,
    ) -> Result<HomeProgramBindingReservation<'_>, crate::lifecycle::WorldHomeProgramBindingDenial>
    {
        use crate::lifecycle::WorldHomeProgramBindingDenial as Denial;
        let mut gates = self.gates.try_lock().map_err(|_| Denial::Busy)?;
        if self.current_home_binding.get().is_some() {
            return Err(Denial::AlreadyBound);
        }
        if gates.home_binding_reserved || gates.capture_reserved {
            return Err(Denial::Busy);
        }
        for flight in [&gates.branches, &gates.custody, &gates.occurrences] {
            if flight.in_caller || flight.subject.is_some() || flight.barrier.is_some() {
                return Err(Denial::Busy);
            }
        }
        gates.home_binding_reserved = true;
        Ok(HomeProgramBindingReservation { owner: self })
    }
}
impl Drop for HomeProgramBindingReservation<'_> {
    fn drop(&mut self) {
        self.owner
            .gates
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
            .home_binding_reserved = false;
    }
}
