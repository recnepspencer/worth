//! Immutable observer bytes and independently mutable decoded fold backing.
use super::super::super::certificate_capacity::CheckpointCustodyDenial as Denial;
use super::super::{backing::LiveReleaseAllocation, heads::SelectedReleaseHeadRoster};
use crate::physical_runtime::durability::FundedCheckpointCommandBuffer;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use worth_store_physical_format::{
    encode_checkpoint_certificate_in_reserved, CheckpointCertificateKind, ReleaseCheckpointBatchV1,
};

pub(in crate::physical_runtime) struct FundedCheckpointBufferPreparation {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) bytes: Vec<u8>,
    /// Keeps the funded allocation alive for the buffer's lifetime.
    pub(in crate::physical_runtime::durability::publication::current_root_owner) _custody:
        Arc<LiveReleaseAllocation>,
}
impl FundedCheckpointBufferPreparation {
    pub(in crate::physical_runtime) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub(in crate::physical_runtime) fn bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
}

pub(in crate::physical_runtime::durability::publication::current_root_owner) struct CertificateRange
{
    pub(in crate::physical_runtime::durability::publication::current_root_owner) kind:
        CheckpointCertificateKind,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) record:
        Range<usize>,
}
pub(in crate::physical_runtime::durability::publication::current_root_owner) struct SnapshotBytes {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) bytes: Vec<u8>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) records:
        Vec<CertificateRange>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) scratch: Vec<u8>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) frame_scratch:
        Vec<u8>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) custody:
        Option<Arc<LiveReleaseAllocation>>,
}
impl SnapshotBytes {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn push(
        &mut self,
        kind: CheckpointCertificateKind,
    ) -> Result<(), Denial> {
        if self.records.len() == self.records.capacity() {
            return Err(Denial::ReleaseCertificateUnavailable);
        }
        encode_checkpoint_certificate_in_reserved(kind, &self.scratch, &mut self.frame_scratch)
            .map_err(|_| Denial::ReleaseCertificateUnavailable)?;
        if self.bytes.capacity() - self.bytes.len() < self.frame_scratch.len() {
            return Err(Denial::ReleaseCertificateUnavailable);
        }
        let start = self.bytes.len();
        self.bytes.extend_from_slice(&self.frame_scratch);
        self.records.push(CertificateRange {
            kind,
            record: start..self.bytes.len(),
        });
        Ok(())
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn clear(
        &mut self,
    ) {
        self.bytes.clear();
        self.records.clear();
        self.scratch.clear();
        self.frame_scratch.clear();
    }
}

pub(in crate::physical_runtime::durability::publication::current_root_owner) struct FoldWorkspace {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) heads:
        SelectedReleaseHeadRoster,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) batches:
        Vec<ReleaseCheckpointBatchV1>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) scratch: Vec<u8>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) custody:
        Option<Arc<LiveReleaseAllocation>>,
}
pub(in crate::physical_runtime::durability::publication::current_root_owner) struct CheckpointPreparation
{
    pub(in crate::physical_runtime::durability::publication::current_root_owner) observer:
        SnapshotBytes,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fold:
        Option<FoldWorkspace>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) command:
        Option<FundedCheckpointCommandBuffer>,
}
pub(in crate::physical_runtime::durability::publication::current_root_owner) struct ReusableCheckpointSlot
{
    pub(in crate::physical_runtime::durability::publication::current_root_owner) available:
        Mutex<Option<Arc<SealedCheckpointStorage>>>,
}

pub(in crate::physical_runtime) struct SealedCheckpointStorage {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) observer:
        Option<SnapshotBytes>,
    fold: Mutex<Option<FoldWorkspace>>,
    command: Mutex<Option<FundedCheckpointCommandBuffer>>,
}
impl SealedCheckpointStorage {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn from_prepared(
        preparation: CheckpointPreparation,
    ) -> Self {
        Self {
            observer: Some(preparation.observer),
            fold: Mutex::new(preparation.fold),
            command: Mutex::new(preparation.command),
        }
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn take_preparation(
        &mut self,
    ) -> CheckpointPreparation {
        CheckpointPreparation {
            observer: self.observer.take().expect("unique prepared observer"),
            fold: self
                .fold
                .get_mut()
                .unwrap_or_else(|p| p.into_inner())
                .take(),
            command: self
                .command
                .get_mut()
                .unwrap_or_else(|p| p.into_inner())
                .take(),
        }
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn put_preparation(
        &mut self,
        preparation: CheckpointPreparation,
    ) {
        self.observer = Some(preparation.observer);
        *self.fold.get_mut().unwrap_or_else(|p| p.into_inner()) = preparation.fold;
        *self.command.get_mut().unwrap_or_else(|p| p.into_inner()) = preparation.command;
    }
    /// Capsule uniqueness does not imply returned command-byte uniqueness:
    /// a settled command or retry observer can retain that independent Arc.
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn returned_command_is_exclusive(
        &self,
    ) -> bool {
        self.command
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_mut()
            .is_some_and(|buffer| buffer.bytes_mut().is_some())
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn take_fold(
        &self,
    ) -> Option<FoldWorkspace> {
        self.fold.lock().unwrap_or_else(|p| p.into_inner()).take()
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn return_fold(
        &self,
        fold: FoldWorkspace,
    ) {
        *self.fold.lock().unwrap_or_else(|p| p.into_inner()) = Some(fold);
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn lease_fold(
        self: &Arc<Self>,
    ) -> Option<FoldLease> {
        self.take_fold().map(|workspace| FoldLease {
            owner: Arc::clone(self),
            workspace: Some(workspace),
        })
    }
    pub(in crate::physical_runtime) fn take_command_buffer(
        self: &Arc<Self>,
    ) -> Option<FundedCheckpointCommandBufferLease> {
        self.command
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
            .map(|buffer| FundedCheckpointCommandBufferLease {
                owner: Arc::clone(self),
                buffer: Some(buffer),
            })
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn views(
        self: &Arc<Self>,
    ) -> CheckpointCertificateViews<'_> {
        CheckpointCertificateViews { owner: self }
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn frame(
        self: &Arc<Self>,
        index: usize,
    ) -> Option<FundedCheckpointFrame> {
        self.observer.as_ref()?.records.get(index)?;
        Some(FundedCheckpointFrame {
            owner: Arc::clone(self),
            index,
        })
    }
}

pub(in crate::physical_runtime::durability::publication::current_root_owner) struct FoldLease {
    owner: Arc<SealedCheckpointStorage>,
    workspace: Option<FoldWorkspace>,
}
impl FoldLease {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn workspace_mut(
        &mut self,
    ) -> &mut FoldWorkspace {
        self.workspace.as_mut().unwrap()
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn into_workspace(
        mut self,
    ) -> FoldWorkspace {
        self.workspace.take().unwrap()
    }
}
impl Drop for FoldLease {
    fn drop(&mut self) {
        if let Some(workspace) = self.workspace.take() {
            self.owner.return_fold(workspace);
        }
    }
}
pub(in crate::physical_runtime) struct FundedCheckpointCommandBufferLease {
    owner: Arc<SealedCheckpointStorage>,
    buffer: Option<FundedCheckpointCommandBuffer>,
}
impl FundedCheckpointCommandBufferLease {
    pub(in crate::physical_runtime) fn buffer_mut(&mut self) -> &mut FundedCheckpointCommandBuffer {
        self.buffer.as_mut().expect("lease owns command buffer")
    }
}
impl Drop for FundedCheckpointCommandBufferLease {
    fn drop(&mut self) {
        *self.owner.command.lock().unwrap_or_else(|p| p.into_inner()) = self.buffer.take();
    }
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct FundedCheckpointFrame {
    owner: Arc<SealedCheckpointStorage>,
    index: usize,
}
impl FundedCheckpointFrame {
    pub(in crate::physical_runtime) fn bytes(&self) -> &[u8] {
        let observer = self.owner.observer.as_ref().expect("sealed observer bytes");
        &observer.bytes[observer.records[self.index].record.clone()]
    }
}
#[derive(Clone)]
pub(in crate::physical_runtime) struct SelectedCheckpointCertificate {
    frame: FundedCheckpointFrame,
}
impl SelectedCheckpointCertificate {
    pub(in crate::physical_runtime) fn kind(&self) -> CheckpointCertificateKind {
        self.frame.owner.observer.as_ref().unwrap().records[self.frame.index].kind
    }
    pub(in crate::physical_runtime) fn payload(&self) -> &[u8] {
        worth_store_physical_format::decode_checkpoint_certificate(self.frame.bytes())
            .expect("sealed canonical framing")
            .1
    }
}
pub(in crate::physical_runtime) struct CheckpointCertificateViews<'a> {
    owner: &'a Arc<SealedCheckpointStorage>,
}
impl CheckpointCertificateViews<'_> {
    pub(in crate::physical_runtime) fn len(&self) -> usize {
        self.owner.observer.as_ref().unwrap().records.len()
    }
    pub(in crate::physical_runtime) fn iter(
        &self,
    ) -> impl Iterator<Item = SelectedCheckpointCertificate> + '_ {
        (0..self.len()).map(|index| SelectedCheckpointCertificate {
            frame: self.owner.frame(index).unwrap(),
        })
    }
    #[cfg(test)]
    pub(in crate::physical_runtime) fn get(
        &self,
        index: usize,
    ) -> Option<SelectedCheckpointCertificate> {
        self.owner
            .frame(index)
            .map(|frame| SelectedCheckpointCertificate { frame })
    }
}
