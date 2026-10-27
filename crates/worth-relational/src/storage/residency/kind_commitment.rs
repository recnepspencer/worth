//! Relation projections bound to the selected manifest's root commitment.
//! Member hashing alone cannot replace the leaf's admitted projection digest.
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::RegionRelationProjection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RelationalRegionKindCommitment {
    pub(crate) head: [u8; 32],
    pub(crate) history: [u8; 32],
}

pub(super) fn digest(projection: &[RegionRelationProjection]) -> Result<[u8; 32], Cause> {
    let bytes = rmp_serde::to_vec(projection).map_err(|_| Cause::Damaged)?;
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.region-kind-authority.v2\0");
    hash.update(bytes);
    Ok(hash.finalize().into())
}

pub(super) fn verify(
    projection: &[RegionRelationProjection],
    expected: [u8; 32],
) -> Result<(), Cause> {
    if digest(projection)? != expected {
        return Err(Cause::IntegrityMismatch);
    }
    Ok(())
}

impl RelationalRegionKindCommitment {
    pub(crate) fn capture(
        partition: &crate::storage::overlay::PartitionState,
    ) -> Result<Self, Cause> {
        use crate::storage::partition::AdjacencyKindBasis;
        Ok(Self {
            head: digest(&super::kind_projection::capture(
                partition,
                AdjacencyKindBasis::Current,
            )?)?,
            history: digest(&super::kind_projection::capture(
                partition,
                AdjacencyKindBasis::Historical,
            )?)?,
        })
    }
}
