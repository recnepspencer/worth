//! Intrinsic storage for the fixed canonical wire index, separate from cell custody.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::history::{
    data::{RelationalAssociationDiscovery, RelationalCommitReceipt},
    RelationalCommitIndexEntry,
};
use serde::{Deserialize, Serialize};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
struct Backing {
    entries: Vec<RelationalCommitIndexEntry>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct CheckpointCommitIndex(Option<Arc<Backing>>);

pub(crate) struct CanonicalIndexSource<'a> {
    pub(crate) receipt: &'a RelationalCommitReceipt,
    pub(crate) root: &'a crate::branch::RelationalBranchRootDescriptor,
    pub(crate) position: crate::publication::patch::data::PatchStreamPosition,
    pub(crate) digest: [u8; 32],
    pub(crate) schema_version: crate::schema::data::SchemaVersionId,
    pub(crate) association: RelationalAssociationDiscovery,
}
impl CheckpointCommitIndex {
    pub(crate) fn from_ordinary(entries: Vec<RelationalCommitIndexEntry>) -> Self {
        if entries.is_empty() {
            return Self::default();
        }
        Self(Some(Arc::new(Backing {
            entries,
            _memory: None,
        })))
    }
    pub(crate) fn copy_for_request<'a>(
        count: usize,
        at: impl Fn(usize) -> Result<CanonicalIndexSource<'a>, Cause>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        if count == 0 {
            return Ok(Self::default());
        }
        let control = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Backing>())
            .ok()
            .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let mut bytes = control
            .checked_add(array_bytes::<RelationalCommitIndexEntry>(count)?)
            .ok_or(Cause::ChargedBytesOverflow)?;
        for index in 0..count {
            let source = at(index)?;
            bytes = bytes
                .checked_add(array_bytes::<crate::history::data::CommitId>(
                    source.receipt.parents.len(),
                )?)
                .and_then(|bytes| bytes.checked_add(source.receipt.branch_id.0.len() as u64))
                .ok_or(Cause::ChargedBytesOverflow)?;
        }
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        let mut entries = Vec::with_capacity(count);
        for index in 0..count {
            crate::execution::check_request(request)?;
            let source = at(index)?;
            entries.push(RelationalCommitIndexEntry {
                association_discovery: source.association,
                commit: source.receipt.clone(),
                position: source.position,
                root: source.root.clone(),
                payload_digest: source.digest,
                schema_version: source.schema_version,
            });
        }
        Ok(Self(Some(Arc::new(Backing {
            entries,
            _memory: memory,
        }))))
    }
}
impl std::ops::Deref for CheckpointCommitIndex {
    type Target = [RelationalCommitIndexEntry];
    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .map_or(&[], |owner| owner.entries.as_slice())
    }
}
impl PartialEq for CheckpointCommitIndex {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl Eq for CheckpointCommitIndex {}
impl AsRef<[RelationalCommitIndexEntry]> for CheckpointCommitIndex {
    fn as_ref(&self) -> &[RelationalCommitIndexEntry] {
        self
    }
}
impl<'a> IntoIterator for &'a CheckpointCommitIndex {
    type Item = &'a RelationalCommitIndexEntry;
    type IntoIter = std::slice::Iter<'a, RelationalCommitIndexEntry>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for CheckpointCommitIndex {
    type Item = RelationalCommitIndexEntry;
    type IntoIter = std::vec::IntoIter<RelationalCommitIndexEntry>;
    fn into_iter(self) -> Self::IntoIter {
        // Existing ordinary consuming decode/rebuild callers. New capture only borrows.
        self.0
            .map_or_else(Vec::new, |owner| match Arc::try_unwrap(owner) {
                Ok(owner) => owner.entries,
                Err(owner) => owner.entries.clone(),
            })
            .into_iter()
    }
}
impl Serialize for CheckpointCommitIndex {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_ref().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CheckpointCommitIndex {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<RelationalCommitIndexEntry>::deserialize(deserializer).map(Self::from_ordinary)
    }
}
fn array_bytes<T>(count: usize) -> Result<u64, Cause> {
    Layout::array::<T>(count)
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}
