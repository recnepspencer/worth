//! Closed-row Drop debt: a quoted claimant pass, optional for foreign work.
use super::super::refreshed_rejoin::{occurrence_rows, OccurrenceSearchCharge};
use super::*;

pub(in super::super) const FOREIGN_CLOSED_DRAIN_QUOTA: usize = 1;
type RetiredClosedRows = (Vec<DemandRecord>, Vec<Arc<WorthQueryOutputDemandKey>>);

impl DemandRegistryState {
    pub(in super::super) fn take_closed_cleanup_step(
        &mut self,
        note: super::closed_notes::QueuedClosedRow,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RetiredClosedRows, WorthQueryOutputDemandDenial> {
        let key = note.key();
        if !note.names(&self.records) {
            admission.inspect_row()?;
            return Ok(self
                .take_closed_note(note)
                .prune(|_| (Vec::new(), Vec::new())));
        }
        // The candidates are one occurrence, and their cloned keys are admitted
        // before they are built. The cascade plan admits its own whole-map scratch.
        let count = self.records.len();
        let mut producer = 0usize;
        let mut rows = 0usize;
        for (candidate, _) in occurrence_rows(&self.records, &key, admission)? {
            admission.inspect_row()?;
            producer = producer.max(candidate.producer.len());
            rows = rows.checked_add(1).ok_or_else(work_denial)?;
        }
        // One root buffer for the cascade and one for its post-cascade members.
        let slot = std::mem::size_of::<WorthQueryOutputDemandKey>()
            .checked_add(producer)
            .ok_or_else(work_denial)?;
        let scratch = rows
            .checked_mul(2)
            .and_then(|rows| rows.checked_mul(slot))
            .ok_or_else(work_denial)?;
        admission
            .admit_read_scratch(u64::try_from(scratch).map_err(|_| work_denial())?)
            .map_err(|_| empty_capacity_denial())?;
        let mut roots = Vec::with_capacity(rows);
        for (candidate, _) in occurrence_rows(&self.records, &key, admission)? {
            admission.inspect_row()?;
            roots.push(candidate.clone());
        }
        debug_assert!(roots.len() <= count);
        Ok(
            super::super::lifecycle::cleanup_price::price_closed_cascade(self, admission, &roots)?
                .complete(|priced| Ok(priced.prune_closed_note(note, roots))),
        )
    }

    /// One foreign attempt after the request's own edit is fully admitted.
    /// A pause has a holder: the head, unchanged because its quote refused.
    pub(in super::super) fn take_optional_closed_cleanup(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Option<RetiredClosedRows> {
        debug_assert_eq!(FOREIGN_CLOSED_DRAIN_QUOTA, 1);
        let note = self.closed_cleanup.front()?;
        let retired = admission
            .optional_cleanup_quote(|admission| self.take_closed_cleanup_step(note, admission));
        if retired.is_none() {
            self.closed_cleanup.rotate_front();
        }
        retired
    }
}

impl WorthQueryOutputDemandRegistry {
    /// The own edit is complete before optional foreign maintenance starts.
    pub(in super::super) fn drain_foreign_cleanup(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) {
        let retired = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let retired = state.take_optional_closed_cleanup(admission);
            state.drain_foreign_custody_quota(admission);
            retired
        };
        drop(retired);
    }

    #[cfg(test)]
    pub(in super::super) fn drain_closed_cleanup_admitted(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let visits = self.state.lock().unwrap().closed_cleanup.len();
        for _ in 0..visits {
            let retired = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let Some(note) = state.closed_cleanup.front() else {
                    break;
                };
                state.take_closed_cleanup_step(note, admission)?
            };
            drop(retired);
        }
        Ok(())
    }
}
