//! Retain each original recovered row through post-checkpoint retirement.
use super::{
    checkpoint_close_step, drain::CloseProgress, recovered_native::RecoveredNativeRetirement,
    RuntimeWorldCloseDenial as Denial,
};
use crate::recovery::recovered_publication::{
    RecoveredNativeCustody, RecoveredWorldPublicationRecord,
};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
pub(super) struct RecoveredCloseRetirement {
    record: Arc<RecoveredWorldPublicationRecord>,
    native: Option<RecoveredNativeRetirement>,
    initialized: bool,
}

impl RecoveredCloseRetirement {
    fn new(record: Arc<RecoveredWorldPublicationRecord>) -> Self {
        Self {
            record,
            native: None,
            initialized: false,
        }
    }

    fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<bool, Denial> {
        checkpoint_close_step(request, 1)?;
        let record = Arc::get_mut(&mut self.record).ok_or(Denial::UnresolvedBusinessCustody)?;
        if !self.initialized {
            let original = std::mem::replace(
                record
                    .native
                    .get_mut()
                    .unwrap_or_else(|cause| cause.into_inner()),
                RecoveredNativeCustody::Released,
            );
            self.native = match original {
                RecoveredNativeCustody::OpenOnly => None,
                RecoveredNativeCustody::Relational(original) => {
                    Some(original.consume(RecoveredNativeRetirement::new))
                }
                RecoveredNativeCustody::Released => return Err(Denial::CloseProgressInvariant),
            };
            self.initialized = true;
        }
        if let Some(native) = self.native.take() {
            let (native, progress) = native.advance(request);
            self.native = Some(native);
            if !progress? {
                return Ok(false);
            }
        }
        record
            .expected
            .try_retire_for_close()
            .map_err(|_| Denial::Busy)?;
        Ok(true)
    }
}

pub(super) fn prepare(
    request: &ExecutionRequest<'_, '_>,
    count: usize,
) -> Result<
    (
        Vec<RecoveredCloseRetirement>,
        Option<ExecutionMemoryReservation>,
    ),
    Denial,
> {
    let bytes = count
        .checked_mul(std::mem::size_of::<RecoveredCloseRetirement>())
        .ok_or(Denial::ReportAllocationFailed)?;
    let funding = request
        .reserve_retained_memory(u64::try_from(bytes).map_err(|_| Denial::ReportAllocationFailed)?)
        .map_err(Denial::Execution)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| Denial::ReportAllocationFailed)?;
    if rows.capacity() != count {
        return Err(Denial::ReportAllocationFailed);
    }
    Ok((rows, funding))
}

pub(in crate::lifecycle) fn retire(
    recovery: &crate::recovery::RecoveryCatalog,
    task: &Arc<Mutex<CloseProgress>>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    // This try-only task guard serializes its own progress. No World catalog,
    // lifecycle or branch lock spans the concrete nonwaiting Native protocol.
    let mut progress = task.try_lock().map_err(|_| Denial::Busy)?;
    loop {
        checkpoint_close_step(request, 1)?;
        if progress.pending_recovered.is_none() {
            let next =
                recovery.try_take_recovered_for_close(&mut progress.recovered_position, request)?;
            let Some(record) = next else {
                return Ok(());
            };
            progress.pending_recovered = Some(RecoveredCloseRetirement::new(record));
        }
        if !progress
            .pending_recovered
            .as_mut()
            .expect("retained original recovered row")
            .advance(request)?
        {
            continue;
        }
        if progress.retired_recovered.len() == progress.retired_recovered.capacity() {
            return Err(Denial::CloseProgressInvariant);
        }
        let retired = progress
            .pending_recovered
            .take()
            .expect("completed original recovered row");
        progress.retired_recovered.push(retired);
    }
}
