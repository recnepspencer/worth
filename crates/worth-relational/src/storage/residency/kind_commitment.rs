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
    digest_controlled(projection, &mut |_, _| Ok(()))
}

pub(super) fn digest_controlled(
    projection: &[RegionRelationProjection],
    checkpoint: &mut super::kind_projection::ProjectionCheckpoint<'_>,
) -> Result<[u8; 32], Cause> {
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.region-kind-authority.v2\0");
    // Stream the same MessagePack representation; no unbounded serialized Vec.
    let mut writer = ProjectionHashWriter {
        hash: &mut hash,
        checkpoint,
    };
    projection
        .serialize(&mut rmp_serde::Serializer::new(&mut writer))
        .map_err(|_| Cause::Damaged)?;
    Ok(hash.finalize().into())
}

struct ProjectionHashWriter<'borrow, 'control> {
    hash: &'borrow mut Sha256,
    checkpoint: &'borrow mut super::kind_projection::ProjectionCheckpoint<'control>,
}
impl std::io::Write for ProjectionHashWriter<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        for chunk in bytes.chunks(4096) {
            (self.checkpoint)(chunk.len() as u64, 0)
                .map_err(|_| std::io::Error::other("projection admission stopped"))?;
            self.hash.update(chunk);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
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
