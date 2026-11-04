//! Retain each original recovered row through post-checkpoint retirement.
use super::{
    checkpoint_close_step, drain::CloseProgress, recovered_native::RecoveredNativeRetirement,
    RuntimeWorldCloseDenial as Denial,
};
use crate::recovery::{RecoveredNativeCustody, RecoveredWorldPublicationRecord};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) enum RecoveredCloseRetirement {
    Pending(Option<Arc<RecoveredWorldPublicationRecord>>),
    Retiring(Option<OriginalRecoveredClose>),
    Retired(OriginalRecoveredClose),
}

pub(super) struct OriginalRecoveredClose {
    _identity: crate::identity::ProductUnpublishedOwnerEffectsIdentity,
    _attempt: crate::identity::CompositePublicationAttemptIdentity,
    _commit: crate::identity::CompositeCommitIdentity,
    _open: crate::durability::AdmittedWorldOpenAttempt,
    _release: Mutex<Option<crate::durability::WorldDurableArtifact>>,
    _committed_release:
        Mutex<Option<worth_proof::DurablyCommitted<crate::durability::ActualWorldPublication>>>,
    _claimed: Mutex<bool>,
    expected: crate::branch::ProductBranchObservation,
    native: Option<RecoveredNativeRetirement>,
    expected_basis: Option<crate::history::CompositeCommitRetirement>,
}

impl RecoveredCloseRetirement {
    fn new(record: Arc<RecoveredWorldPublicationRecord>) -> Self {
        Self::Pending(Some(record))
    }

    fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native_owner: &worth_relational::facade::runtime::RelationalRuntime,
        original_binding: &Arc<crate::durability::WorldDurableMediumBinding>,
    ) -> Result<bool, Denial> {
        checkpoint_close_step(request, 1)?;
        match self {
            Self::Pending(slot) => {
                let record = slot.as_ref().expect("retained original recovered record");
                if !Arc::ptr_eq(&record.binding, original_binding) {
                    return Err(Denial::CloseProgressInvariant);
                }
                let record = slot.take().expect("same original recovered record");
                match Arc::try_unwrap(record) {
                    Ok(original) => {
                        *self = Self::Retiring(Some(OriginalRecoveredClose::new(original)));
                        Ok(false)
                    }
                    Err(record) => {
                        *slot = Some(record);
                        Err(Denial::UnresolvedBusinessCustody)
                    }
                }
            }
            Self::Retiring(slot) => {
                if !slot
                    .as_mut()
                    .expect("original retirement is retained")
                    .advance(request, native_owner)?
                {
                    return Ok(false);
                }
                let original = slot.take().expect("completed original retirement");
                *self = Self::Retired(original);
                Ok(true)
            }
            Self::Retired(_) => Ok(true),
        }
    }
}

impl OriginalRecoveredClose {
    fn new(original: RecoveredWorldPublicationRecord) -> Self {
        let RecoveredWorldPublicationRecord {
            identity,
            attempt,
            commit,
            open,
            continued: _,
            native_identity: _,
            expected,
            binding,
            native,
            claimed,
            release,
            committed_release,
            continuation,
            metadata_bytes: _,
        } = original;
        let native = match native
            .into_inner()
            .unwrap_or_else(|cause| cause.into_inner())
        {
            RecoveredNativeCustody::OpenOnly => None,
            RecoveredNativeCustody::Relational(original) => Some(
                Arc::try_unwrap(original)
                    .expect("exclusive original Native custody")
                    .consume(RecoveredNativeRetirement::new),
            ),
            RecoveredNativeCustody::Prepared(view) => {
                drop(view);
                let prepared = continuation
                    .into_inner()
                    .unwrap_or_else(|e| e.into_inner())
                    .expect("same prepared continuation");
                Some(RecoveredNativeRetirement::Publication(
                    (*prepared).into_home_retirement(),
                ))
            }
            RecoveredNativeCustody::Released => {
                unreachable!("the exclusive recovered record passed final sealed validation")
            }
        };
        // The caller still borrows the pointer-equal binding in the live World
        // owner. This is a nonfinal local share, so no medium/provider Drop can
        // run here. Retired data must not retain this Serving capability.
        drop(binding);
        Self {
            _identity: identity,
            _attempt: attempt,
            _commit: commit,
            _open: open,
            _release: release,
            _committed_release: committed_release,
            _claimed: claimed,
            expected,
            native,
            expected_basis: None,
        }
    }

    fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native_owner: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<bool, Denial> {
        if let Some(native) = self.native.take() {
            let (native, progress) = native.advance(request, native_owner);
            self.native = Some(native);
            if !progress? {
                return Ok(false);
            }
        }
        if self.expected_basis.is_none() {
            self.expected
                .try_retire_for_close()
                .map_err(|_| Denial::Busy)?;
            self.expected_basis = Some(
                self.expected
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
    original_binding: &Arc<crate::durability::WorldDurableMediumBinding>,
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
            .advance(request, native_owner, original_binding)?
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
        let (phase, identity) = match self {
            Self::Pending(original) => (
                "pending",
                &original.as_ref().expect("original record").identity,
            ),
            Self::Retiring(original) => (
                "retiring",
                &original.as_ref().expect("original retirement")._identity,
            ),
            Self::Retired(original) => ("retired", &original._identity),
        };
        f.debug_struct("RecoveredCloseRetirement")
            .field("phase", &phase)
            .field("identity", identity)
            .finish()
    }
}
