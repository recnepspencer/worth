#[cfg(test)]
use super::{CheckpointRootBasis, CheckpointWalSourceRange, PhysicalCheckpointIdentity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistedCompactionProductRole {
    OperationBindingIndex,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedCompactionCutoverRecord {
    checkpoint: PhysicalCheckpointIdentity,
    root: CheckpointRootBasis,
    checkpoint_wal: CheckpointWalSourceRange,
    product_role: PersistedCompactionProductRole,
    product_generation: u64,
    wal_cutoff_lsn_exclusive: u64,
}

#[cfg(test)]
impl PersistedCompactionCutoverRecord {
    pub(super) const fn admitted_from_verified_checkpoint(
        checkpoint: PhysicalCheckpointIdentity,
        root: CheckpointRootBasis,
        checkpoint_wal: CheckpointWalSourceRange,
        product_generation: u64,
        wal_cutoff_lsn_exclusive: u64,
    ) -> Self {
        Self {
            checkpoint,
            root,
            checkpoint_wal,
            product_role: PersistedCompactionProductRole::OperationBindingIndex,
            product_generation,
            wal_cutoff_lsn_exclusive,
        }
    }

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
        self.product_role
    }

    pub const fn product_generation(self) -> u64 {
        self.product_generation
    }

    pub const fn wal_cutoff_lsn_exclusive(self) -> u64 {
        self.wal_cutoff_lsn_exclusive
    }
}
