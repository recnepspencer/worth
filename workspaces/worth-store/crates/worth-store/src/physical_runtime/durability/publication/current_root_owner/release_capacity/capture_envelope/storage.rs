//! Immutable observer bytes and the independently mutable decoded fold, both
//! inside one admitted capture envelope.
use super::super::super::certificate_capacity::CheckpointCustodyDenial as Denial;
use super::super::{backing::LiveReleaseAllocation, heads::SelectedReleaseHeadRoster};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use worth_store_physical_format::{
    encode_checkpoint_certificate_in_reserved, CheckpointCertificateKind, ReleaseCheckpointBatchV1,
};

pub(in crate::physical_runtime::durability::publication::current_root_owner) struct CertificateRange
{
    pub(in crate::physical_runtime::durability::publication::current_root_owner) kind:
        CheckpointCertificateKind,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) record:
        Range<usize>,
}
#[derive(Default)]
pub(in crate::physical_runtime::durability::publication::current_root_owner) struct SnapshotBytes {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) bytes: Vec<u8>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) records:
        Vec<CertificateRange>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) scratch: Vec<u8>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) frame_scratch:
        Vec<u8>,
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
}

pub(in crate::physical_runtime::durability::publication::current_root_owner) struct FoldWorkspace {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) heads:
        SelectedReleaseHeadRoster,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) batches:
        Vec<ReleaseCheckpointBatchV1>,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) scratch: Vec<u8>,
}
pub(in crate::physical_runtime::durability::publication::current_root_owner) struct CheckpointPreparation
{
    pub(in crate::physical_runtime::durability::publication::current_root_owner) observer:
        SnapshotBytes,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fold:
        FoldWorkspace,
    pub(in crate::physical_runtime::durability::publication::current_root_owner) custody:
        Arc<LiveReleaseAllocation>,
}

/// Declared buffers first: they are freed before the reservation they use.
pub(in crate::physical_runtime) struct SealedCheckpointStorage {
    observer: SnapshotBytes,
    fold: Mutex<Option<FoldWorkspace>>,
    _custody: Arc<LiveReleaseAllocation>,
}
impl SealedCheckpointStorage {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn from_prepared(
        preparation: CheckpointPreparation,
    ) -> Self {
        Self {
            observer: preparation.observer,
            fold: Mutex::new(Some(preparation.fold)),
            _custody: preparation.custody,
        }
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn lease_fold(
        self: &Arc<Self>,
    ) -> Option<FoldLease> {
        let workspace = self.fold.lock().unwrap_or_else(|p| p.into_inner()).take()?;
        Some(FoldLease {
            owner: Arc::clone(self),
            workspace: Some(workspace),
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
    ) -> Option<CheckpointCertificateFrame> {
        self.observer.records.get(index)?;
        Some(CheckpointCertificateFrame {
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
            *self.owner.fold.lock().unwrap_or_else(|p| p.into_inner()) = Some(workspace);
        }
    }
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct CheckpointCertificateFrame {
    owner: Arc<SealedCheckpointStorage>,
    index: usize,
}
impl CheckpointCertificateFrame {
    pub(in crate::physical_runtime) fn bytes(&self) -> &[u8] {
        let observer = &self.owner.observer;
        &observer.bytes[observer.records[self.index].record.clone()]
    }
}
#[derive(Clone)]
pub(in crate::physical_runtime) struct SelectedCheckpointCertificate {
    frame: CheckpointCertificateFrame,
}
impl SelectedCheckpointCertificate {
    pub(in crate::physical_runtime) fn kind(&self) -> CheckpointCertificateKind {
        self.frame.owner.observer.records[self.frame.index].kind
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
        self.owner.observer.records.len()
    }
    pub(in crate::physical_runtime) fn iter(
        &self,
    ) -> impl Iterator<Item = SelectedCheckpointCertificate> + '_ {
        (0..self.len()).map(|index| SelectedCheckpointCertificate {
            frame: self.owner.frame(index).unwrap(),
        })
    }
}
