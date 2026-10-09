//! Inline checkpoint observations; copying them retains no certificate backing.

use worth_store_physical_format::{
    CheckpointRootBasis, CheckpointStreamFooter, CheckpointWalSourceRange,
    PersistedCompactionProductRole, PhysicalCheckpointIdentity, PhysicalCheckpointSource,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedCheckpointFacts {
    pub(super) source: PhysicalCheckpointSource,
    pub(super) footer: CheckpointStreamFooter,
    pub(super) encoded_bytes: u64,
    pub(super) encoded_digest: [u8; 32],
    pub(super) compaction_cutover: VerifiedCheckpointCompactionCutover,
}

impl VerifiedCheckpointFacts {
    pub const fn source(self) -> PhysicalCheckpointSource {
        self.source
    }
    pub const fn footer(self) -> CheckpointStreamFooter {
        self.footer
    }
    pub const fn encoded_bytes(self) -> u64 {
        self.encoded_bytes
    }
    pub const fn encoded_digest(self) -> [u8; 32] {
        self.encoded_digest
    }
    pub const fn compaction_cutover(self) -> VerifiedCheckpointCompactionCutover {
        self.compaction_cutover
    }
    pub const fn certificate_record_count(self) -> u64 {
        self.footer.certificate_record_count()
    }
    pub const fn certificate_record_bytes(self) -> u64 {
        self.footer.certificate_record_bytes()
    }
    pub const fn certificate_records_digest(self) -> [u8; 32] {
        self.footer.certificate_records_digest()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedCheckpointCompactionCutover {
    pub(super) checkpoint: PhysicalCheckpointIdentity,
    pub(super) root: CheckpointRootBasis,
    pub(super) checkpoint_wal: CheckpointWalSourceRange,
    pub(super) product_generation: u64,
    pub(super) wal_cutoff_lsn_exclusive: u64,
}

impl VerifiedCheckpointCompactionCutover {
    pub const fn checkpoint(self) -> PhysicalCheckpointIdentity {
        self.checkpoint
    }
    pub const fn root(self) -> CheckpointRootBasis {
        self.root
    }
    pub const fn checkpoint_wal(self) -> CheckpointWalSourceRange {
        self.checkpoint_wal
    }
    pub const fn product_role(self) -> PersistedCompactionProductRole {
        PersistedCompactionProductRole::OperationBindingIndex
    }
    pub const fn product_generation(self) -> u64 {
        self.product_generation
    }
    pub const fn wal_cutoff_lsn_exclusive(self) -> u64 {
        self.wal_cutoff_lsn_exclusive
    }
}
