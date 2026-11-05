//! One streaming selector reader retains actual original-request allocations.
use super::{
    PositionNode, PublicationRoute, RequiredOutputPublicationPositions, RequiredOutputRoutePosition,
};
use crate::durability::canonical_commit_artifact::borrowed_payload::Cursor;
use crate::durability::checkpoint_admission::{
    native_stop, RelationalCheckpointAdmissionRefusal as Refusal,
};
use crate::history::data::CommitId;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
mod family;
mod node;

const MAXIMUM_HEIGHT: usize = 64;
struct Readmission<'r, 'scope, 'authority> {
    request: &'r ExecutionRequest<'scope, 'authority>,
    contains: &'r dyn Fn(CommitId) -> bool,
    previous: Option<Arc<PublicationRoute>>,
}
impl Readmission<'_, '_, '_> {
    fn charge(&self, bytes: u64) -> Result<(), Refusal> {
        crate::durability::artifact_payload_emission::request::account_work(self.request, bytes)
            .map_err(native_stop)
    }
    fn reserve<T>(&self, tail: usize) -> Result<Option<ExecutionMemoryReservation>, Refusal> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<T>())
            .map_err(|_| Refusal::CapacityOverflow)?
            .0
            .pad_to_align();
        let bytes = layout
            .size()
            .checked_add(tail)
            .ok_or(Refusal::CapacityOverflow)? as u64;
        self.charge(bytes)?;
        self.request
            .reserve_retained_memory(bytes)
            .map_err(|cause| Refusal::Request(cause.into()))
    }
}
impl RequiredOutputPublicationPositions {
    pub(crate) fn readmit_checkpoint(
        bytes: &[u8],
        contains: &impl Fn(CommitId) -> bool,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let mut context = Readmission {
            request,
            contains,
            previous: None,
        };
        let mut cursor = Cursor::new(bytes, request);
        require_count(cursor.map_length(), 2)?;
        key(&mut cursor, "unresolved_prefix")?;
        let unresolved_prefix = if cursor.take_nil().map_err(projected_stop)? {
            None
        } else {
            let commit = CommitId(cursor.unsigned().map_err(projected_stop)?);
            if !contains(commit) {
                return Err(Refusal::InvalidProjectedMeaning);
            }
            Some(commit)
        };
        key(&mut cursor, "root")?;
        let root = node::read(&mut cursor, &mut context, 0)?;
        cursor.finish().map_err(projected_stop)?;
        Ok(Self {
            root,
            unresolved_prefix,
        })
    }
}
fn key(cursor: &mut Cursor<'_, '_, '_>, expected: &str) -> Result<(), Refusal> {
    let raw = cursor.string_bytes().map_err(projected_stop)?;
    if raw.len() != expected.len() || raw != expected.as_bytes() {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    Ok(())
}
fn require_count(
    count: Result<
        usize,
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal,
    >,
    expected: usize,
) -> Result<(), Refusal> {
    if count.map_err(projected_stop)? != expected {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    Ok(())
}
fn projected_stop(
    cause: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal,
) -> Refusal {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal as Cause;
    match cause {
        Cause::Request(cause) => Refusal::Request(cause),
        Cause::Content(cause) => Refusal::Content(cause),
        Cause::Frame(_)
        | Cause::PayloadDecode { .. }
        | Cause::PositionDisagreement { .. }
        | Cause::CommitIdDisagreement { .. }
        | Cause::FieldText(_)
        | Cause::PositionAdmission(_)
        | Cause::NonCanonicalPayload => Refusal::InvalidProjectedMeaning,
    }
}
