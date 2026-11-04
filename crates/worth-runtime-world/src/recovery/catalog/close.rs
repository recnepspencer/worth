//! Durable recovered obligations survive close; live work still refuses it.
use super::{ProductUnpublishedRecoveryCatalog, RecoveryEntry};
use crate::lifecycle::{
    checkpoint_close_step as checkpoint, RecoveredCloseReceipt, RuntimeWorldCloseDenial as Denial,
    RuntimeWorldRecoveredCustodyReport,
};
use crate::recovery::recovered_publication::RecoveredNativeCustody;
use worth_execution::ExecutionRequest;

impl ProductUnpublishedRecoveryCatalog {
    pub(crate) fn try_close_recovered_count(
        &self,
        durable: bool,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<usize, Denial> {
        request.consult().map_err(Denial::Execution)?;
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        if state.reserved_slots != 0 || state.updating_slots != 0 || state.abandoned_slots != 0 {
            return Err(Denial::UnresolvedBusinessCustody);
        }
        for position in 0..state.slots.span() {
            checkpoint(request, 1)?;
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

    /// Fallibly copy every description before admission can seal. The single
    /// array reservation and each receipt's own funding travel with the output.
    pub(crate) fn prepare_close_reports(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Vec<RuntimeWorldRecoveredCustodyReport>,
            Option<worth_execution::ExecutionMemoryReservation>,
        ),
        Denial,
    > {
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        let count = state.slots.occupied_len();
        let bytes = count
            .checked_mul(std::mem::size_of::<RuntimeWorldRecoveredCustodyReport>())
            .ok_or(Denial::ReportAllocationFailed)?;
        let funding = request
            .reserve_retained_memory(
                u64::try_from(bytes).map_err(|_| Denial::ReportAllocationFailed)?,
            )
            .map_err(Denial::Execution)?;
        let mut rows = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| Denial::ReportAllocationFailed)?;
        if rows.capacity() != count {
            return Err(Denial::ReportAllocationFailed);
        }
        for position in 0..state.slots.span() {
            checkpoint(request, 1)?;
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
                    let receipt = native.custody().effect().subject().receipt();
                    Some(RecoveredCloseReceipt::copy_for_request(request, receipt)?)
                }
                RecoveredNativeCustody::Prepared(record) => {
                    Some(RecoveredCloseReceipt::copy_for_request(
                        request,
                        record.effect().subject().receipt(),
                    )?)
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
        Ok((rows, funding))
    }

    /// Recheck the exact prebuilt descriptions without allocating after seal.
    pub(crate) fn validate_close_reports(
        &self,
        rows: &[RuntimeWorldRecoveredCustodyReport],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        if rows.len() != state.slots.occupied_len() {
            return Err(Denial::PublicationFrontierChanged);
        }
        let mut rows = rows.iter();
        for position in 0..state.slots.span() {
            checkpoint(request, 1)?;
            let Some((_, entry)) = state.slots.at(position) else {
                continue;
            };
            let RecoveryEntry::Recovered(record) = entry else {
                return Err(Denial::PublicationFrontierChanged);
            };
            let row = rows.next().ok_or(Denial::PublicationFrontierChanged)?;
            if row.identity != record.identity
                || row.attempt != record.attempt
                || row.commit != record.commit
            {
                return Err(Denial::PublicationFrontierChanged);
            }
            let native = record.native.try_lock().map_err(|_| Denial::Busy)?;
            let receipt = match &*native {
                RecoveredNativeCustody::OpenOnly => None,
                RecoveredNativeCustody::Relational(native) => {
                    Some(native.custody().effect().subject().receipt())
                }
                RecoveredNativeCustody::Prepared(record) => {
                    Some(record.effect().subject().receipt())
                }
                RecoveredNativeCustody::Released => return Err(Denial::PublicationFrontierChanged),
            };
            let matches = match (&row.native_receipt, receipt) {
                (Some(row), Some(receipt)) => row.matches_for_request(request, receipt)?,
                (None, None) => true,
                (None, Some(_)) | (Some(_), None) => false,
            };
            if !matches {
                return Err(Denial::PublicationFrontierChanged);
            }
        }
        Ok(())
    }

    /// Move one exclusive original into the already funded close task.
    /// This neither releases pins nor publishes a durable release disposition.
    pub(crate) fn try_take_recovered_for_close(
        &self,
        position: &mut usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<
            std::sync::Arc<crate::recovery::recovered_publication::RecoveredWorldPublicationRecord>,
        >,
        Denial,
    > {
        loop {
            checkpoint(request, 1)?;
            let removed = {
                let mut state = self.state.try_lock().map_err(|_| Denial::Busy)?;
                if *position >= state.slots.span() {
                    return Ok(None);
                }
                let identity = state.slots.at(*position).map(|(id, _)| id.clone());
                let Some(identity) = identity else {
                    *position += 1;
                    continue;
                };
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
                *position += 1;
                record
            };
            return Ok(Some(removed));
        }
    }
}
