//! Store's real immutable native cut; the reader retires before Serving.
use super::{Cause, StoreColdSource};
use std::{
    any::Any,
    sync::{Arc, Weak},
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::durability::{
    RelationalDurableOwnedReadRefusal as Refusal, RelationalDurableReadBounds as Bounds,
    RelationalDurableReadPayload,
};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, publication::PatchStreamPosition,
};
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordProtectedMembership,
    PhysicalCheckpointReadProtectionDenial as Capture, PhysicalRecordReader,
    ServingPhysicalRuntime,
};
pub(super) struct StoreCheckpointReadCut {
    pub(super) reader: PhysicalRecordReader,
    pub(super) membership: Option<AnchoredRecordProtectedMembership>,
    issuer: Weak<StoreColdSource>,
    pub(super) bounds: Bounds,
    pub(super) serving: Arc<ServingPhysicalRuntime>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl StoreColdSource {
    pub(super) fn capture_native_read(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(Bounds, RelationalDurableReadPayload), Refusal> {
        let memory =
            request.reserve_retained_memory(std::mem::size_of::<StoreCheckpointReadCut>() as u64)?;
        let reader =
            self.serving
                .try_records_for_checkpoint(request)
                .map_err(|cause| match cause {
                    Capture::Busy => Refusal::Busy,
                    Capture::Request(cause) => Refusal::Request(cause.into()),
                    Capture::Protection(_) => Refusal::Storage(Cause::SourceUnavailable),
                })?;
        let canonical = self
            .serving
            .canonical_commits()
            .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?
            .head_position_from(&reader)
            .map_err(|failure| super::owned_read::journal_failure(failure, 0))?;
        let transitions = self
            .serving
            .branch_transitions()
            .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?
            .head_position_from(&reader)
            .map_err(|failure| super::owned_read::journal_failure(failure, 0))?;
        let membership = self
            .serving
            .anchored_records()
            .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?
            .protected_membership_from(
                &reader,
                request,
                AnchoredRecordName::new("home.checkpoint").expect("static anchor"),
            )
            .map_err(|failure| match failure {
                worth_store::physical_runtime::AnchoredRecordReadFailure::Execution(cause) => {
                    Refusal::from(cause)
                }
                worth_store::physical_runtime::AnchoredRecordReadFailure::Allocation(cause) => {
                    Refusal::Buffer(worth_execution::ExecutionRequestArrayDenial::Allocation(
                        cause,
                    ))
                }
                worth_store::physical_runtime::AnchoredRecordReadFailure::ProtectionBusy => {
                    Refusal::Busy
                }
                worth_store::physical_runtime::AnchoredRecordReadFailure::ForeignReader => {
                    Refusal::ForeignProtection
                }
                _ => Refusal::Storage(Cause::SourceUnavailable),
            })?;
        let bounds = Bounds {
            canonical: PatchStreamPosition(canonical),
            transitions: RelationalBranchTransitionSequence(transitions),
        };
        Ok((
            bounds,
            Box::new(StoreCheckpointReadCut {
                reader,
                membership,
                issuer: self.weak_self.clone(),
                bounds,
                serving: self.serving.clone(),
                _memory: memory,
            }),
        ))
    }
    pub(super) fn validate_native_read<'a>(
        &self,
        original: &'a (dyn Any + Send + Sync),
    ) -> Result<&'a StoreCheckpointReadCut, Refusal> {
        let cut = original
            .downcast_ref::<StoreCheckpointReadCut>()
            .ok_or(Refusal::ForeignProtection)?;
        if cut.issuer.as_ptr() != self as *const Self || !Arc::ptr_eq(&cut.serving, &self.serving) {
            return Err(Refusal::ForeignSource);
        }
        Ok(cut)
    }
}

#[cfg(test)]
mod tests;
