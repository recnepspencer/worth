//! Retain each original recovered row through post-checkpoint retirement.
use super::{
    checkpoint_close_step, drain::CloseProgress, recovered_native::RecoveredNativeRetirement,
    RuntimeWorldCloseDenial as Denial,
};
use crate::recovery::{RecoveredNativeCustody, RecoveredWorldPublicationRecord};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) struct RecoveredCloseRetirement {
    record: Arc<RecoveredWorldPublicationRecord>,
    native: Option<RecoveredNativeRetirement>,
    initialized: bool,
    expected_basis: Option<crate::history::CompositeCommitRetirement>,
}

impl RecoveredCloseRetirement {
    fn new(record: Arc<RecoveredWorldPublicationRecord>) -> Self {
        Self {
            record,
            native: None,
            initialized: false,
            expected_basis: None,
        }
    }

    fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native_owner: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<bool, Denial> {
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
        if self.expected_basis.is_none() {
            record
                .expected
                .try_retire_for_close()
                .map_err(|_| Denial::Busy)?;
            self.expected_basis = Some(
                record
                    .expected
                    .begin_close_basis_retirement()
                    .map_err(|_| Denial::Busy)?,
            );
        }
        use worth_relational::facade::branch::RelationalBranchBasisRetirementDenial as NativeDenial;
        self.expected_basis
            .as_mut()
            .expect("same original expected basis")
            .advance(request, native_owner)
            .map_err(|cause| match cause {
                NativeDenial::Busy => Denial::Busy,
                NativeDenial::Execution(cause) => Denial::Execution(cause),
                NativeDenial::ForeignOwner => Denial::CloseProgressInvariant,
            })
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
    native_owner: &worth_relational::facade::runtime::RelationalRuntime,
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
            .advance(request, native_owner)?
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

impl std::fmt::Debug for RecoveredCloseRetirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveredCloseRetirement")
            .field("identity", &self.record.identity)
            .field("initialized", &self.initialized)
            .field("expected_basis_started", &self.expected_basis.is_some())
            .finish_non_exhaustive()
    }
}
