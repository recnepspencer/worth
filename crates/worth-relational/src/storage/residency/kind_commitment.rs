//! Relation projections bound to the selected manifest's root commitment.
//! Member hashing alone cannot replace the leaf's admitted projection digest.
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::RegionRelationProjection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RelationalRegionKindCommitment {
    pub(crate) head: [u8; 32],
    pub(crate) history: [u8; 32],
}

pub(super) fn digest(projection: &[RegionRelationProjection]) -> Result<[u8; 32], Cause> {
    digest_controlled(projection, &mut |_, _| Ok(()))
}

pub(super) fn digest_controlled(
    projection: &[RegionRelationProjection],
    checkpoint: &mut super::kind_projection::ProjectionCheckpoint<'_>,
) -> Result<[u8; 32], Cause> {
    // cost-lane: reconstruction. Ordinary publication edits the carried tree.
    checkpoint(1, 0)?;
    super::kind_index::KindIndex::rebuild(projection, checkpoint).map(|index| index.digest())
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

#[cfg(test)]
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
