//! Closed-row Drop debt: a quoted claimant pass, optional for foreign work.
use super::super::refreshed_rejoin::{occurrence_rows, OccurrenceSearchCharge};
use super::*;

pub(in super::super) const FOREIGN_CLOSED_DRAIN_QUOTA: usize = 1;
type RetiredClosedRows = (Vec<DemandRecord>, Vec<Arc<WorthQueryOutputDemandKey>>);

impl DemandRegistryState {
    pub(in super::super) fn take_closed_cleanup_step(
        &mut self,
        key: Arc<WorthQueryOutputDemandKey>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RetiredClosedRows, WorthQueryOutputDemandDenial> {
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
        let scratch = rows
            .checked_mul(
                std::mem::size_of::<WorthQueryOutputDemandKey>()
                    .checked_add(producer)
                    .ok_or_else(work_denial)?,
            )
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
                .complete(|priced| Ok(priced.prune_closed_note(key, roots))),
        )
    }

    /// One foreign attempt after the request's own edit is fully admitted.
    /// A pause has a holder: the head, unchanged because its quote refused.
    pub(in super::super) fn take_optional_closed_cleanup(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Option<RetiredClosedRows> {
        debug_assert_eq!(FOREIGN_CLOSED_DRAIN_QUOTA, 1);
        let key = Arc::clone(self.closed_cleanup.head.as_ref()?);
        match self.take_closed_cleanup_step(key, admission) {
            Ok(retired) => Some(retired),
            Err(denial) => {
                debug_assert!(matches!(
                    denial.kind(),
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                        | WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                ));
                None
            }
        }
    }
}

impl WorthQueryOutputDemandRegistry {
    /// A reader needing the closed row pays for its claimant pass before use.
    pub(in super::super) fn drain_closed_for_occurrence(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let retired = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.closed_cleanup.len == 0 {
                return Ok(());
            }
            let mut found = None;
            for (candidate, row) in occurrence_rows(&state.records, key, admission)? {
                admission.inspect_row()?;
                if row.closed_cleanup.queued() {
                    found = Some(Arc::clone(
                        row.work_membership
                            .as_ref()
                            .map(|member| member.key_arc())
                            .or_else(|| state.required_keys.get(candidate))
                            .unwrap(),
                    ));
                    break;
                }
            }
            match found {
                Some(key) => Some(state.take_closed_cleanup_step(key, admission)?),
                None => None,
            }
        };
        drop(retired);
        Ok(())
    }

    pub(in super::super) fn drain_closed_cleanup_admitted(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let visits = self.state.lock().unwrap().closed_cleanup.len;
        for _ in 0..visits {
            let retired = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let Some(key) = state.closed_cleanup.head.as_ref().map(Arc::clone) else {
                    break;
                };
                state.take_closed_cleanup_step(key, admission)?
            };
            drop(retired);
        }
        Ok(())
    }
}
