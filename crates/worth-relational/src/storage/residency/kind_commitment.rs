//! Manifest-carried proof of the admitted adjacency projection. Unlike a
//! member's checksum, these facts remain fixed when untrusted member bytes
//! are readmitted. Each region verifies its own projection independently.
use super::region_image::RegionAdjacencyKinds;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RelationalRegionKindCommitment {
    pub(crate) head: [u8; 32],
    pub(crate) history: [u8; 32],
}

pub(super) fn digest(
    outgoing: &[RegionAdjacencyKinds],
    incoming: &[RegionAdjacencyKinds],
) -> Result<[u8; 32], Cause> {
    let bytes = rmp_serde::to_vec(&(outgoing, incoming)).map_err(|_| Cause::Damaged)?;
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.region-kind-authority.v1\0");
    hash.update(bytes);
    Ok(hash.finalize().into())
}

pub(super) fn verify(
    outgoing: &[RegionAdjacencyKinds],
    incoming: &[RegionAdjacencyKinds],
    expected: [u8; 32],
) -> Result<(), Cause> {
    if digest(outgoing, incoming)? != expected {
        return Err(Cause::IntegrityMismatch);
    }
    Ok(())
}
