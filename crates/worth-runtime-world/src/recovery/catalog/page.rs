use super::ProductUnpublishedRecoveryCatalog;
use super::RecoveryEntry;
use crate::inspection::RuntimeWorldRecoveryCursor;
use crate::inspection::{
    RuntimeWorldRecoveredOriginal, RuntimeWorldRecoveryPage, RuntimeWorldRecoveryRecordState,
    RuntimeWorldRecoveryRow,
};
use crate::lifecycle::RuntimeWorldInstant;
use crate::recovery::{ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial};
use std::num::NonZeroUsize;
impl ProductUnpublishedRecoveryCatalog {
    pub(crate) fn snapshot(&self) -> crate::inspection::RuntimeWorldRecoverySnapshot {
        let state = self.locked_state();
        crate::inspection::RuntimeWorldRecoverySnapshot {
            installed: state.slots.retained_len(),
            reserved: state.reserved_slots,
            abandoned: state.abandoned_slots,
            updating: state.updating_slots,
            retained_metadata_bytes: state.metadata_bytes,
            reserved_metadata_bytes: state.reserved_metadata_bytes,
            costs: state.costs,
        }
    }

    pub(crate) fn page(
        &self,
        after: Option<&RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
        now: RuntimeWorldInstant,
    ) -> Result<RuntimeWorldRecoveryPage, RuntimeWorldRecoveryDenial> {
        let state = self.locked_state();
        if let Some(cursor) = after {
            state
                .owner
                .admit_incarnation(cursor.owner)
                .map_err(|cause| match cause {
                    worth_proof::HomeIncarnationDenial::ForeignHome => {
                        RuntimeWorldRecoveryDenial::ForeignHandle
                    }
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        RuntimeWorldRecoveryDenial::StaleIncarnation
                    }
                })?;
        }
        if after.is_some_and(|cursor| {
            cursor.owner != state.owner || cursor.catalog_affinity != self.affinity()
        }) {
            return Err(RuntimeWorldRecoveryDenial::ForeignHandle);
        }
        let start = after
            .map_or(0, |cursor| cursor.next_slot)
            .min(state.slots.span());
        let end = start.saturating_add(maximum.get()).min(state.slots.span());
        let mut rows = Vec::new();
        for position in start..end {
            let Some((id, entry)) = state.slots.at(position) else {
                continue;
            };
            let recovered = match entry {
                RecoveryEntry::Recovered(_) => Some(RuntimeWorldRecoveredOriginal::Publication),
                RecoveryEntry::RecoveredCreation(_) => {
                    Some(RuntimeWorldRecoveredOriginal::BranchCreation)
                }
                RecoveryEntry::Active(_) | RecoveryEntry::Retained(_) | RecoveryEntry::Busy => None,
            };
            let (posture, time) = match entry {
                RecoveryEntry::Retained(record) => (
                    RuntimeWorldRecoveryRecordState::Retained,
                    Some((record.admitted_at(), record.deadline())),
                ),
                RecoveryEntry::Recovered(_) | RecoveryEntry::RecoveredCreation(_) => {
                    (RuntimeWorldRecoveryRecordState::Recovered, None)
                }
                RecoveryEntry::Active(record) => (
                    if record.is_abandoned() {
                        RuntimeWorldRecoveryRecordState::Abandoned
                    } else {
                        RuntimeWorldRecoveryRecordState::Active
                    },
                    Some((record.admitted_at(), record.deadline())),
                ),
                RecoveryEntry::Busy => (RuntimeWorldRecoveryRecordState::Busy, None),
            };
            rows.push(RuntimeWorldRecoveryRow {
                handle: ProductUnpublishedRecoveryHandle::new(id.clone(), self.affinity()),
                state: posture,
                recovered,
                admitted_at: time.map(|(t, _)| t),
                age_ticks: time.and_then(|(t, _)| now.elapsed_ticks_since(t)),
                deadline_expired: time
                    .map(|(_, deadline)| deadline.is_some_and(|d| now.deadline_elapsed(d))),
            });
        }
        let next_after = (end < state.slots.span()).then_some(RuntimeWorldRecoveryCursor {
            owner: state.owner,
            catalog_affinity: self.affinity(),
            next_slot: end,
        });
        Ok(RuntimeWorldRecoveryPage {
            rows,
            next_after,
            examined: end - start,
            observed_at: now,
        })
    }
}
