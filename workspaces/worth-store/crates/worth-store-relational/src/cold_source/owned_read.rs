//! Exact protected stream output is admitted before growth and sealed at EOF.
use super::{Cause, StoreColdSource};
use std::any::Any;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence,
    durability::{RelationalDurableMemberLocator, RelationalDurableOwnedReadRefusal as Refusal},
    publication::PatchStreamPosition,
};
use worth_store::physical_runtime::{
    OrderedJournalPayloadStream, OrderedJournalReadFailure as Failure,
};
impl StoreColdSource {
    pub(super) fn owned_commit(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        let cut = self.validate_native_read(original)?;
        if position.0 == 0 || position > cut.bounds.canonical {
            return Err(Refusal::Bounds);
        }
        self.commit_reads.fetch_add(1, super::Ordering::Relaxed);
        let journal = self
            .serving
            .canonical_commits()
            .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?;
        let stream = journal
            .open_payload_from(&cut.reader, request, position.0, limit)
            .map_err(|cause| journal_failure(cause, limit))?;
        let bytes = copy_journal(request, stream, limit)?;
        #[cfg(feature = "certification-test-authority")]
        if let Some(observer) = &self.commit_read_observer {
            observer(position, self.counters());
        }
        Ok(bytes)
    }
    pub(super) fn owned_transition(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        sequence: RelationalBranchTransitionSequence,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        let cut = self.validate_native_read(original)?;
        if sequence.0 == 0 || sequence > cut.bounds.transitions {
            return Err(Refusal::Bounds);
        }
        let journal = self
            .serving
            .branch_transitions()
            .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?;
        let stream = journal
            .open_payload_from(&cut.reader, request, sequence.0, limit)
            .map_err(|cause| journal_failure(cause, limit))?;
        copy_journal(request, stream, limit)
    }
    pub(super) fn owned_member(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        if locator.length > limit {
            return Err(Refusal::Storage(Cause::LimitExceeded {
                length: locator.length,
                limit,
            }));
        }
        let mut stream = if let Some(stream) =
            self.open_selected_owned_member(request, original, locator, limit)?
        {
            stream
        } else {
            let cut = self.validate_native_read(original)?;
            self.member_reads.fetch_add(1, super::Ordering::Relaxed);
            let membership = cut
                .membership
                .as_ref()
                .ok_or(Refusal::Storage(Cause::MissingMember))?;
            let member = membership
                .member(locator.sha256, locator.length)
                .ok_or(Refusal::Storage(Cause::MissingMember))?;
            self.serving
                .anchored_records()
                .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?
                .open_protected_member(&cut.reader, membership, member, limit)
                .map_err(|failure| {
                    Refusal::Storage(super::member_read::member_failure(failure, limit))
                })?
        };
        let length = usize::try_from(locator.length)
            .map_err(|_| Refusal::Storage(Cause::MemoryExhausted))?;
        let mut builder = ExecutionRequestArrayBuilder::<u8>::allocate(*request, length)
            .map_err(Refusal::Buffer)?;
        let mut scratch = [0; 16 * 1024];
        loop {
            request
                .check_live()
                .map_err(|cause| Refusal::Request(cause.into()))?;
            let count = stream.read_next(&mut scratch).map_err(|failure| {
                Refusal::Storage(super::member_read::member_failure(failure, limit))
            })?;
            if count == 0 {
                break;
            }
            if count > length - builder.len() {
                return Err(Refusal::Storage(Cause::Damaged));
            }
            builder
                .extend_from_slice(&scratch[..count])
                .map_err(Refusal::Buffer)?;
        }
        if builder.len() != length {
            return Err(Refusal::Storage(Cause::Damaged));
        }
        Ok(builder
            .seal()
            .map_err(Refusal::Buffer)?
            .into_immutable_bytes())
    }
}
fn copy_journal(
    request: &ExecutionRequest<'_, '_>,
    mut stream: OrderedJournalPayloadStream<'_, '_>,
    limit: u64,
) -> Result<ExecutionImmutableBytes, Refusal> {
    let length = stream.entry().payload_bytes();
    if length > limit {
        return Err(Refusal::Storage(Cause::LimitExceeded { length, limit }));
    }
    let length = usize::try_from(length).map_err(|_| Refusal::Storage(Cause::MemoryExhausted))?;
    let mut builder =
        ExecutionRequestArrayBuilder::<u8>::allocate(*request, length).map_err(Refusal::Buffer)?;
    let mut scratch = [0; 16 * 1024];
    loop {
        let count = stream
            .read_next(&mut scratch)
            .map_err(|cause| journal_failure(cause, limit))?;
        if count == 0 {
            break;
        }
        if count > length - builder.len() {
            return Err(Refusal::Storage(Cause::Damaged));
        }
        builder
            .extend_from_slice(&scratch[..count])
            .map_err(Refusal::Buffer)?;
    }
    if builder.len() != length {
        return Err(Refusal::Storage(Cause::Damaged));
    }
    Ok(builder
        .seal()
        .map_err(Refusal::Buffer)?
        .into_immutable_bytes())
}
pub(super) fn journal_failure(failure: Failure, limit: u64) -> Refusal {
    match failure {
        Failure::Execution(cause) => Refusal::from(cause),
        Failure::ForeignReader => Refusal::ForeignProtection,
        Failure::NotFound { .. } => Refusal::Storage(Cause::MissingMember),
        Failure::LimitExceeded { payload_bytes } => Refusal::Storage(Cause::LimitExceeded {
            length: payload_bytes,
            limit,
        }),
        Failure::Damaged => Refusal::Storage(Cause::Damaged),
        _ => Refusal::Storage(Cause::SourceUnavailable),
    }
}
