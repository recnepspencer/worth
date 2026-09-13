mod backup_artifact;
mod golden;
mod hostile;
mod roundtrip;

use std::num::NonZeroU64;

use crate::store_namespace::{ProposedStoreIdentity, StableStoreIdentity};

use super::super::{
    CheckpointRootBasis, CheckpointWalSourceRange, PhysicalCheckpointIdentity,
    PhysicalCheckpointSource,
};

fn identity(sequence: u64) -> PhysicalCheckpointIdentity {
    let proposed = ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap();
    PhysicalCheckpointIdentity::new(
        StableStoreIdentity::from_published_record(proposed),
        NonZeroU64::new(sequence).unwrap(),
    )
}

fn source(sequence: u64) -> PhysicalCheckpointSource {
    PhysicalCheckpointSource::concurrent(
        identity(sequence),
        CheckpointWalSourceRange::new(11, 29).unwrap(),
        CheckpointRootBasis::new(5, 71),
        43,
    )
}

fn secured_source(sequence: u64) -> PhysicalCheckpointSource {
    PhysicalCheckpointSource::secured_concurrent(
        identity(sequence),
        CheckpointWalSourceRange::new(11, 29).unwrap(),
        CheckpointRootBasis::new(5, 71),
        43,
        [9; 32],
        8,
    )
    .unwrap()
}
