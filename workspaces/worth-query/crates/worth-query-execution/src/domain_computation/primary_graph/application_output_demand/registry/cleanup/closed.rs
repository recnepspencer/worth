//! Closed-row Drop debt: a quoted claimant pass, optional for foreign work.
use super::super::refreshed_rejoin::occurrence_rows;
use super::*;

pub(in super::super) struct RetiredForeignCleanup {
    rows: RetiredClosedRows,
    terminal: Option<super::RetiredTerminalCleanup>,
}
impl RetiredForeignCleanup {
    fn closed(rows: RetiredClosedRows) -> Self {
        Self {
            rows,
            terminal: None,
        }
    }
    fn terminal(terminal: super::RetiredTerminalCleanup) -> Self {
        Self {
            rows: (Vec::new(), Vec::new()),
            terminal: Some(terminal),
        }
    }
    fn destroy(self) {
        let Self { rows, terminal } = self;
        drop(rows);
        drop(terminal);
    }
}
type RetiredClosedRows = (Vec<DemandRecord>, Vec<Arc<WorthQueryOutputDemandKey>>);

impl DemandRegistryState {
    pub(in super::super) fn take_closed_cleanup_step(
        &mut self,
        note: super::closed_notes::QueuedClosedRow,
    ) -> Result<RetiredClosedRows, WorthQueryOutputDemandDenial> {
        let key = note.key();
        if !note.names(&self.records) {
            return Ok(self
                .take_closed_note(note)
                .prune(|_| (Vec::new(), Vec::new())));
        }
        // The candidates are one occurrence, and their cloned keys are admitted
        // before they are built. The cascade plan admits its own whole-map scratch.
        let count = self.records.len();
        let mut producer = 0usize;
        let mut rows = 0usize;
        for (candidate, _) in occurrence_rows(&self.records, &key) {
            producer = producer.max(candidate.producer.len());
            rows = rows.checked_add(1).ok_or_else(work_denial)?;
        }
        // One root buffer for the cascade and one for its post-cascade members.

        let mut roots = Vec::with_capacity(rows);
        for (candidate, _) in occurrence_rows(&self.records, &key) {
            roots.push(candidate.clone());
        }
        debug_assert!(roots.len() <= count);
        Ok({
            let priced: &mut DemandRegistryState = &mut *self;
            {
                priced.prune_closed_note(note, roots)
            }
        })
    }

    /// One foreign attempt after the request's own edit is fully admitted.
    /// A pause has a holder: the head, unchanged because its quote refused.
    pub(in super::super) fn take_optional_closed_cleanup(
        &mut self,
    ) -> Option<RetiredForeignCleanup> {
        if self.pending_cleanup_head.is_some() {
            return self
                .take_terminal_cleanup_step()
                .ok()
                .flatten()
                .map(RetiredForeignCleanup::terminal);
        }
        let note = self.closed_cleanup.front()?;
        let retired = self.take_closed_cleanup_step(note).ok();
        if retired.is_none() {
            self.closed_cleanup.rotate_front();
        }
        retired.map(RetiredForeignCleanup::closed)
    }
}

impl WorthQueryOutputDemandRegistry {
    /// The own edit is complete before optional foreign maintenance starts.
    pub(in super::super) fn drain_foreign_cleanup(&self) {
        let retired = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let retired = state.take_optional_closed_cleanup();
            state.drain_next_foreign_custody();
            retired
        };
        self.finish_foreign_cleanup(retired);
    }

    pub(in super::super) fn finish_foreign_cleanup(&self, retired: Option<RetiredForeignCleanup>) {
        if let Some(retired) = retired {
            retired.destroy();
        }
    }

    #[cfg(test)]
    pub(in super::super) fn drain_closed_cleanup_admitted(
        &self,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let visits = self.state.lock().unwrap().closed_cleanup.len();
        for pass in 0..=visits {
            loop {
                let retired = {
                    let mut state = self.state.lock().unwrap();
                    if state.pending_cleanup_head.is_some() {
                        state.take_terminal_cleanup_step()?
                    } else {
                        None
                    }
                };
                let Some(retired) = retired else {
                    break;
                };
                self.finish_foreign_cleanup(Some(RetiredForeignCleanup::terminal(retired)));
            }
            if pass == visits {
                break;
            }
            let retired = {
                let mut state = self.state.lock().unwrap();
                let Some(note) = state.closed_cleanup.front() else {
                    break;
                };
                state.take_closed_cleanup_step(note)?
            };
            self.finish_foreign_cleanup(Some(RetiredForeignCleanup::closed(retired)));
        }
        Ok(())
    }
}
