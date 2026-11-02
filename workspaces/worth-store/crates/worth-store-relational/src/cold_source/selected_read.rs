//! Original installed selected read and persistent placements, never a latest-reader alias.
use super::{selection::MemberMap, Cause, StoreColdSource};
use std::{
    any::Any,
    sync::{Arc, TryLockError, Weak},
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::durability::{
    RelationalDurableMemberLocator, RelationalDurableOwnedReadRefusal as Refusal,
    RelationalDurableReadPayload,
};
use worth_store::physical_runtime::{
    AnchoredRecordMemberStream, AnchoredRecordReadMembership, ServingPhysicalRuntime,
};

mod retirement;

pub(super) struct StoreSelectedRead {
    read: Option<AnchoredRecordReadMembership>,
    retirement: Option<worth_store::physical_runtime::AnchoredRecordReadRetirement>,
    _retired: Option<worth_store::physical_runtime::RetiredAnchoredRecordReadMembership>,
    complete: bool,
    entries: MemberMap,
    issuer: Weak<StoreColdSource>,
    // Last operational owner: read/table obligations disappear before Serving.
    serving: Option<Arc<ServingPhysicalRuntime>>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl StoreColdSource {
    pub(crate) fn member_locators_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_execution::ExecutionArray<RelationalDurableMemberLocator>, Refusal> {
        request.consult()?;
        let entries = {
            let current = match self.members.try_read() {
                Ok(current) => current,
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
                Err(TryLockError::WouldBlock) => return Err(Refusal::Busy),
            };
            current.entries.clone()
        };
        let mut roster =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, entries.len())
                .map_err(Refusal::Buffer)?;
        for key in entries.keys() {
            roster.push(*key).map_err(Refusal::Buffer)?;
        }
        roster.seal().map_err(Refusal::Buffer)
    }
    pub(super) fn capture_selected_read(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, Refusal> {
        let memory =
            request.reserve_retained_memory(std::mem::size_of::<StoreSelectedRead>() as u64)?;
        request.consult()?;
        // Allocate all control storage before borrowing the installed read.
        // On any refusal the payload still has NO read share to destroy.
        let mut payload = Box::new(StoreSelectedRead {
            read: None,
            retirement: None,
            _retired: None,
            complete: false,
            entries: MemberMap::default(),
            issuer: self.weak_self.clone(),
            serving: Some(Arc::clone(self.serving())),
            _memory: memory,
        });
        {
            let current = match self.members.try_read() {
                Ok(current) => current,
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
                Err(TryLockError::WouldBlock) => return Err(Refusal::Busy),
            };
            let read = current
                .read
                .as_ref()
                .ok_or(Refusal::Storage(Cause::SourceUnavailable))?;
            self.serving()
                .anchored_records()
                .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?
                .validate_read_membership(read)
                .map_err(read_failure)?;
            // No I/O or fallible operation follows these nonallocating shares.
            payload.read = Some(read.clone());
            payload.entries = current.entries.clone();
        }
        Ok(payload)
    }
    pub(super) fn open_selected_owned_member(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Option<AnchoredRecordMemberStream>, Refusal> {
        let Some(selected) = original.downcast_ref::<StoreSelectedRead>() else {
            return Ok(None);
        };
        if selected.issuer.as_ptr() != self as *const Self
            || !selected
                .serving
                .as_ref()
                .is_some_and(|serving| Arc::ptr_eq(serving, self.serving()))
        {
            return Err(Refusal::ForeignSource);
        }
        request.consult()?;
        self.member_reads.fetch_add(1, super::Ordering::Relaxed);
        let placement = selected
            .entries
            .get(&locator)
            .ok_or(Refusal::Storage(Cause::MissingMember))?;
        let stream = self
            .serving()
            .anchored_records()
            .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?
            .open_read_member(
                selected
                    .read
                    .as_ref()
                    .ok_or(Refusal::Storage(Cause::SourceUnavailable))?,
                placement.index,
                limit,
            )
            .map_err(|failure| {
                Refusal::Storage(super::member_read::member_failure(failure, limit))
            })?;
        Ok(Some(stream))
    }
}
fn read_failure(failure: worth_store::physical_runtime::AnchoredRecordReadFailure) -> Refusal {
    use worth_store::physical_runtime::AnchoredRecordReadFailure as Failure;
    match failure {
        Failure::Execution(cause) => Refusal::from(cause),
        Failure::ProtectionBusy => Refusal::Busy,
        Failure::ForeignReader => Refusal::ForeignProtection,
        _ => Refusal::Storage(Cause::SourceUnavailable),
    }
}
