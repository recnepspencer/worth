//! Durable recovered obligations survive close; live work still refuses it.
use super::{ProductUnpublishedRecoveryCatalog, RecoveryEntry};
use crate::lifecycle::{RuntimeWorldCloseDenial as Denial, RuntimeWorldRecoveredCustodyReport};
use crate::recovery::recovered_publication::RecoveredNativeCustody;
use worth_execution::ExecutionRequest;

impl ProductUnpublishedRecoveryCatalog {
    pub(crate) fn try_close_recovered_count(&self, durable: bool) -> Result<usize, Denial> {
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        if state.reserved_slots != 0 || state.updating_slots != 0 || state.abandoned_slots != 0 {
            return Err(Denial::UnresolvedBusinessCustody);
        }
        for position in 0..state.slots.span() {
            let Some((_, entry)) = state.slots.at(position) else {
                continue;
            };
            match entry {
                RecoveryEntry::Recovered(record) if durable => {
                    if *record.claimed.try_lock().map_err(|_| Denial::Busy)? {
                        return Err(Denial::UnresolvedBusinessCustody);
                    }
                }
                RecoveryEntry::Recovered(_)
                | RecoveryEntry::Active(_)
                | RecoveryEntry::Retained(_)
                | RecoveryEntry::Busy => {
                    return Err(Denial::UnresolvedBusinessCustody);
                }
            }
        }
        Ok(state.slots.occupied_len())
    }

    /// The caller has already sealed admission and funded the entire output.
    pub(crate) fn append_close_reports(
        &self,
        rows: &mut Vec<RuntimeWorldRecoveredCustodyReport>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        if rows.capacity() < state.slots.occupied_len() {
            return Err(Denial::PublicationFrontierChanged);
        }
        // Retry rebuilds only descriptive inline identities, never capabilities.
        rows.clear();
        for position in 0..state.slots.span() {
            request.consult().map_err(Denial::Execution)?;
            let Some((_, entry)) = state.slots.at(position) else {
                continue;
            };
            let RecoveryEntry::Recovered(record) = entry else {
                return Err(Denial::CloseProgressInvariant);
            };
            let native = record.native.try_lock().map_err(|_| Denial::Busy)?;
            let receipt = match &*native {
                RecoveredNativeCustody::OpenOnly => None,
                RecoveredNativeCustody::Relational(native) => {
                    Some(native.custody().effect().subject().receipt().clone())
                }
                RecoveredNativeCustody::Released => return Err(Denial::CloseProgressInvariant),
            };
            rows.push(RuntimeWorldRecoveredCustodyReport {
                identity: record.identity.clone(),
                attempt: record.attempt.clone(),
                commit: record.commit.clone(),
                native_receipt: receipt,
            });
        }
        Ok(())
    }

    /// Process-local pins retire after the Home checkpoint attempt. This does
    /// not publish a release disposition: the original durable open survives.
    pub(crate) fn retire_closed_recovered(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let mut position = 0;
        loop {
            request.consult().map_err(Denial::Execution)?;
            let removed = {
                let mut state = self.state.try_lock().map_err(|_| Denial::Busy)?;
                if position >= state.slots.span() {
                    return Ok(());
                }
                let identity = state.slots.at(position).map(|(id, _)| id.clone());
                position += 1;
                let Some(identity) = identity else { continue };
                match state.slots.get(&identity) {
                    Some(RecoveryEntry::Recovered(record))
                        if std::sync::Arc::strong_count(record) == 1 => {}
                    Some(RecoveryEntry::Recovered(_)) => {
                        return Err(Denial::UnresolvedBusinessCustody)
                    }
                    Some(
                        RecoveryEntry::Active(_) | RecoveryEntry::Retained(_) | RecoveryEntry::Busy,
                    )
                    | None => return Err(Denial::CloseProgressInvariant),
                }
                let Some(RecoveryEntry::Recovered(record)) = state.slots.remove(&identity) else {
                    unreachable!("checked exact recovered owner")
                };
                state.metadata_bytes -= record.metadata_bytes;
                record
            };
            // Native capability and expected-head destruction happen outside
            // every World catalog/lifecycle lock, through this explicit path.
            drop(removed);
        }
    }
}
