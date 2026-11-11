//! A small escaped index subject retains only its own actual metadata bill.
use super::RelationalCommitIndexEntry;
use crate::history::data::{PositionedCanonicalCommit, RelationalCommitReceipt};
use std::{alloc::Layout, ops::Deref, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug)]
struct Backing {
    entry: RelationalCommitIndexEntry,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone)]
pub(crate) struct RetainedCanonicalIndex(Arc<Backing>);
impl RetainedCanonicalIndex {
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    fn reserve(
        receipt: &RelationalCommitReceipt,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<ExecutionMemoryReservation>, crate::execution::RelationalExecutionDenialCause>
    {
        let bytes = Self::allocation_bytes(receipt)
            .ok_or(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)?;
        request.reserve_retained_memory(bytes).map_err(Into::into)
    }
    pub(crate) fn allocation_bytes(receipt: &RelationalCommitReceipt) -> Option<u64> {
        let control = Layout::new::<(usize, usize)>()
            .extend(Layout::new::<Backing>())
            .ok()?
            .0
            .pad_to_align()
            .size();
        let parents = Layout::array::<crate::history::data::CommitId>(receipt.parents.len())
            .ok()?
            .size();
        u64::try_from(
            control
                .checked_add(parents)?
                .checked_add(receipt.branch_id.0.len())?,
        )
        .ok()
    }
    pub(crate) fn from_publication(
        commit: &PositionedCanonicalCommit,
        roots: &crate::branch::RelationalBranchRootDescriptor,
        digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        Self,
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
    > {
        use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
        let bytes = Self::allocation_bytes(&commit.commit).ok_or(Denial::FrameSizeOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let entry = RelationalCommitIndexEntry {
            association_discovery: commit.envelope().association_discovery(),
            commit: commit.commit.clone(),
            position: commit.position(),
            root: roots.clone(),
            payload_digest: digest,
            schema_version: commit.schema_version,
        };
        Ok(Self(Arc::new(Backing {
            entry,
            _memory: memory,
        })))
    }
    pub(crate) fn copy_original(
        entry: &RelationalCommitIndexEntry,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        let memory = Self::reserve(&entry.commit, request)?;
        Ok(Self(Arc::new(Backing {
            entry: entry.clone(),
            _memory: memory,
        })))
    }
    /// Existing ordinary value construction is explicitly outside controlled copy claims.
    #[cfg(test)]
    pub(crate) fn from_ordinary_owned(entry: RelationalCommitIndexEntry) -> Self {
        Self(Arc::new(Backing {
            entry,
            _memory: None,
        }))
    }
}
impl Deref for RetainedCanonicalIndex {
    type Target = RelationalCommitIndexEntry;
    fn deref(&self) -> &Self::Target {
        &self.0.entry
    }
}
impl AsRef<RelationalCommitIndexEntry> for RetainedCanonicalIndex {
    fn as_ref(&self) -> &RelationalCommitIndexEntry {
        self
    }
}

#[cfg(test)]
mod tests;
