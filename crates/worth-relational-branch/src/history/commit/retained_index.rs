//! A small escaped index subject retains only its own actual metadata bill.
use super::{envelope_residency::JournalCommitReadmission, RelationalCommitIndexEntry};
use crate::history::data::{PositionedCanonicalCommit, RelationalCommitReceipt};
use crate::storage::substrate::allocation_witness::RelationalAllocationWitness;
use std::{alloc::Layout, ops::Deref, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational_storage::internal_port::storage::substrate::allocation_witness::RelationalAllocationWitnessOwnerPort as _;
#[derive(Debug)]
struct Backing {
    entry: RelationalCommitIndexEntry,
    journal_readmission: JournalCommitReadmission,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone)]
pub struct RetainedCanonicalIndex(Arc<Backing>);
impl RetainedCanonicalIndex {
    pub(super) fn readmit_journal(
        &self,
        source: &Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        protection: Option<&crate::durability::medium::RelationalDurableReadProtection>,
        request: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<
        PositionedCanonicalCommit,
        crate::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
        (self.0.journal_readmission)(&self.0.entry, source, protection, request, limit)
    }

    pub fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    /// Weak identity of this exact retained index owner, for custody census.
    pub fn allocation_witness(&self) -> RelationalAllocationWitness {
        RelationalAllocationWitness::of(&self.0)
    }
    fn reserve(
        receipt: &RelationalCommitReceipt,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<ExecutionMemoryReservation>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let bytes = Self::allocation_bytes(receipt).ok_or(
            crate::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow,
        )?;
        request.reserve_retained_memory(bytes).map_err(Into::into)
    }
    pub fn allocation_bytes(receipt: &RelationalCommitReceipt) -> Option<u64> {
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
    pub fn from_publication(
        commit: &PositionedCanonicalCommit,
        roots: &crate::branch::target::RelationalBranchRootDescriptor,
        digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
        journal_readmission: JournalCommitReadmission,
    ) -> Result<
        Self,
        crate::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial,
    >{
        use crate::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
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
            journal_readmission,
            _memory: memory,
        })))
    }
    pub fn copy_original(
        entry: &RelationalCommitIndexEntry,
        request: &ExecutionRequest<'_, '_>,
        journal_readmission: JournalCommitReadmission,
    ) -> Result<Self, crate::execution::denial_cause::RelationalExecutionDenialCause> {
        let memory = Self::reserve(&entry.commit, request)?;
        Ok(Self(Arc::new(Backing {
            entry: entry.clone(),
            journal_readmission,
            _memory: memory,
        })))
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

impl RetainedCanonicalIndex {
    #[doc(hidden)]
    pub fn readmit_journal_for_test(
        &self,
        source: &Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<
        PositionedCanonicalCommit,
        crate::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
        self.readmit_journal(source, None, request, limit)
    }
}
