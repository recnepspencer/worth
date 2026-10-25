//! Current-format region framing and the separate head/history payloads.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::durability::data::{
    PartitionCheckpointImage, VersionedEntityMetadataImage, VersionedRelationMetadataImage,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionHistoryFrontiers;

const MAGIC: &[u8] = b"WORTH-REL-REG2\n";
const VERSION: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RelationalRegionImageKind {
    Head,
    History,
}

pub(crate) struct RelationalRegionImage<'bytes> {
    pub(crate) payload: &'bytes [u8],
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegionHeadImage {
    pub(crate) partition: PartitionCheckpointImage,
    pub(crate) history_frontiers: PartitionHistoryFrontiers,
    pub(crate) adjacency_kinds: Vec<RegionAdjacencyKinds>,
    pub(crate) reverse_adjacency_kinds: Vec<RegionAdjacencyKinds>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegionHistoryImage {
    pub(crate) entities: Vec<Vec<VersionedEntityMetadataImage>>,
    pub(crate) relations: Vec<Vec<VersionedRelationMetadataImage>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegionAdjacencyKinds {
    pub(crate) slot: u64,
    pub(crate) kinds: Vec<(KindId, Vec<RelationId>)>,
}

impl<'bytes> RelationalRegionImage<'bytes> {
    pub(crate) fn encode(
        partition_id: PartitionId,
        schema_carrier_digest: [u8; 32],
        kind: RelationalRegionImageKind,
        payload: &impl Serialize,
    ) -> Result<(RelationalDurableMemberLocator, Vec<u8>), Cause> {
        let payload = rmp_serde::to_vec(payload).map_err(|_| Cause::Damaged)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(MAGIC.len() + 39 + payload.len())
            .map_err(|_| Cause::MemoryExhausted)?;
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&VERSION.to_be_bytes());
        bytes.extend_from_slice(&partition_id.as_u32().to_be_bytes());
        bytes.extend_from_slice(&schema_carrier_digest);
        bytes.push(match kind {
            RelationalRegionImageKind::Head => 0,
            RelationalRegionImageKind::History => 1,
        });
        bytes.extend_from_slice(&payload);
        Ok((
            RelationalDurableMemberLocator {
                sha256: Sha256::digest(&bytes).into(),
                length: bytes.len() as u64,
            },
            bytes,
        ))
    }

    pub(crate) fn decode(
        bytes: &'bytes [u8],
        locator: RelationalDurableMemberLocator,
        partition_id: PartitionId,
        schema_carrier_digest: [u8; 32],
        kind: RelationalRegionImageKind,
    ) -> Result<Self, Cause> {
        if bytes.len() as u64 != locator.length
            || <[u8; 32]>::from(Sha256::digest(bytes)) != locator.sha256
        {
            return Err(Cause::IntegrityMismatch);
        }
        let frame = bytes.strip_prefix(MAGIC).ok_or(Cause::Damaged)?;
        let (header, payload) = frame.split_at_checked(39).ok_or(Cause::Damaged)?;
        let expected_kind = match kind {
            RelationalRegionImageKind::Head => 0,
            RelationalRegionImageKind::History => 1,
        };
        if header[..2] != VERSION.to_be_bytes()
            || header[2..6] != partition_id.as_u32().to_be_bytes()
            || header[6..38] != schema_carrier_digest
            || header[38] != expected_kind
        {
            return Err(Cause::Damaged);
        }
        Ok(Self { payload })
    }
}
