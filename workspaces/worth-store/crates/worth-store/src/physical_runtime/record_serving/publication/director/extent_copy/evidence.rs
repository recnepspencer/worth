use crate::physical_runtime::record_serving::residency::candidate_frame_residency::{
    CandidateFrameCoordinate, CandidateFramePhysicalWrite,
};
use crate::physical_runtime::{
    CompletedPhysicalPublicationEffect, PhysicalPublicationEffect, PhysicalWorkEffectFate,
    PhysicalWorkIdentity, PhysicalWorkRecoveryDisposition,
};
use sha2::{Digest, Sha256};
use worth_store_physical_format::{PhysicalExtentCopyIntent, RecordFrameCoordinate};

/// A bounded fold of completed executor receipts, never a declaration of work.
pub(in crate::physical_runtime) struct ExtentCopyWriteEvidence {
    frames: u32,
    bytes: u64,
    digest: [u8; 32],
    last_work: PhysicalWorkIdentity,
}

impl ExtentCopyWriteEvidence {
    pub(in crate::physical_runtime) const fn frames(&self) -> u32 {
        self.frames
    }
    pub(in crate::physical_runtime) const fn bytes(&self) -> u64 {
        self.bytes
    }
    pub(in crate::physical_runtime) const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(in crate::physical_runtime) const fn last_work(&self) -> PhysicalWorkIdentity {
        self.last_work
    }
}

pub(super) struct CopyWriteAccumulator {
    frames: u32,
    bytes: u64,
    digest: Sha256,
    last_work: Option<PhysicalWorkIdentity>,
}

impl CopyWriteAccumulator {
    pub(super) fn new(intent: PhysicalExtentCopyIntent, lsn: u64) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"store.extent-copy.completed-writes.v1");
        digest.update(intent.operation());
        digest.update(lsn.to_le_bytes());
        Self {
            frames: 0,
            bytes: 0,
            digest,
            last_work: None,
        }
    }

    pub(super) fn observe(
        &mut self,
        store: worth_store_physical_format::store_namespace::StableStoreIdentity,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        physical: CandidateFramePhysicalWrite,
    ) -> Result<(), ()> {
        let settlement = physical
            .settle_residency(
                store,
                CandidateFrameCoordinate::new(coordinate.artifact(), coordinate.offset()),
                bytes,
            )
            .map_err(|_| ())?
            .settlement();
        let effect = settlement.effect().ok_or(())?;
        if settlement.effect_fate() != PhysicalWorkEffectFate::PublicationCompleted
            || settlement.recovery() != PhysicalWorkRecoveryDisposition::ContinueSettlement
            || effect.work() != settlement.identity()
        {
            return Err(());
        }
        if let Some(prior) = self.last_work {
            if prior.runtime() != settlement.identity().runtime()
                || prior.generation() != settlement.identity().generation()
            {
                return Err(());
            }
        }
        self.digest.update(self.frames.to_le_bytes());
        self.digest.update(coordinate.offset().to_le_bytes());
        self.digest.update(coordinate.length().to_le_bytes());
        self.digest.update(Sha256::digest(bytes));
        self.digest
            .update(settlement.identity().runtime().get().to_le_bytes());
        self.digest
            .update(settlement.identity().operation().get().to_le_bytes());
        self.digest
            .update(effect.backend_operation().value().to_le_bytes());
        self.frames = self.frames.checked_add(1).ok_or(())?;
        self.bytes = self.bytes.checked_add(bytes.len() as u64).ok_or(())?;
        self.last_work = Some(settlement.identity());
        Ok(())
    }

    pub(super) fn finish(self, expected: u32) -> Result<ExtentCopyWriteEvidence, ()> {
        if self.frames != expected {
            return Err(());
        }
        Ok(ExtentCopyWriteEvidence {
            frames: self.frames,
            bytes: self.bytes,
            digest: self.digest.finalize().into(),
            last_work: self.last_work.ok_or(())?,
        })
    }
}

/// Actual file and namespace synchronization, bound to this destination arena.
pub(in crate::physical_runtime) struct ExtentCopySynchronization {
    file: CompletedPhysicalPublicationEffect,
    parent: CompletedPhysicalPublicationEffect,
    file_work: PhysicalWorkIdentity,
    parent_work: PhysicalWorkIdentity,
}

impl ExtentCopySynchronization {
    pub(super) fn new(
        intent: PhysicalExtentCopyIntent,
        file: CompletedPhysicalPublicationEffect,
        parent: CompletedPhysicalPublicationEffect,
        file_work: PhysicalWorkIdentity,
        parent_work: PhysicalWorkIdentity,
    ) -> Result<Self, ()> {
        let artifact = worth_store_physical_format::RecordArtifactFile::ExtentArena {
            arena: intent.destination().arena_range().arena().get(),
        };
        if file.artifact() != artifact
            || parent.artifact() != artifact
            || file.effect() != PhysicalPublicationEffect::SynchronizeArtifact
            || parent.effect() != PhysicalPublicationEffect::SynchronizeArtifactParent
            || file_work.store() != parent_work.store()
            || file_work.runtime() != parent_work.runtime()
            || file_work.generation() != parent_work.generation()
        {
            return Err(());
        }
        Ok(Self {
            file,
            parent,
            file_work,
            parent_work,
        })
    }
    pub(in crate::physical_runtime) fn file(&self) -> &CompletedPhysicalPublicationEffect {
        &self.file
    }
    pub(in crate::physical_runtime) fn parent(&self) -> &CompletedPhysicalPublicationEffect {
        &self.parent
    }
    pub(in crate::physical_runtime) fn matches_writes(
        &self,
        writes: &ExtentCopyWriteEvidence,
    ) -> bool {
        let work = writes.last_work();
        self.file.physical().store() == work.store()
            && self.parent.physical().store() == work.store()
            && self.file.physical().owner() == self.parent.physical().owner()
            && self.file_work.store() == work.store()
            && self.file_work.runtime() == work.runtime()
            && self.file_work.generation() == work.generation()
            && self.file_work.operation().get() > work.operation().get()
            && self.parent_work.operation().get() > self.file_work.operation().get()
    }
}
