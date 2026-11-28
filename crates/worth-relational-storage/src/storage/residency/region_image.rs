//! Current-format region framing and the separate head/history payloads.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::storage::data::partition_checkpoint_image::{
    PartitionCheckpointImage, VersionedEntityMetadataImage, VersionedRelationMetadataImage,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionHistoryFrontiers;

pub const MAGIC: &[u8] = b"WORTH-REL-REG8\n";
pub const VERSION: u16 = 8;
const FAMILY: &[u8] = b"WORTH-REL-REG";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalRegionImageKind {
    Head,
    History,
}

pub struct RelationalRegionImage<'bytes> {
    pub payload: &'bytes [u8],
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionHeadImage {
    pub kind_projection: Vec<crate::storage::partition::RegionRelationProjection>,
    pub history_kind_projection: Vec<crate::storage::partition::RegionRelationProjection>,
    pub partition: PartitionCheckpointImage,
    pub history_frontiers: PartitionHistoryFrontiers,
    pub adjacency_kinds: Vec<RegionAdjacencyKinds>,
    pub reverse_adjacency_kinds: Vec<RegionAdjacencyKinds>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionHistoryImage {
    pub kind_projection: Vec<crate::storage::partition::RegionRelationProjection>,
    pub adjacency_kinds: Vec<RegionAdjacencyKinds>,
    pub reverse_adjacency_kinds: Vec<RegionAdjacencyKinds>,
    pub entities: Vec<RegionHistoryRow<VersionedEntityMetadataImage>>,
    pub relations: Vec<RegionHistoryRow<VersionedRelationMetadataImage>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionHistoryBase {
    pub member: RelationalDurableMemberLocator,
    pub partition_id: PartitionId,
    pub schema_carrier_digest: [u8; 32],
    pub row_index: u64,
    pub kind_commitment: [u8; 32],
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionHistoryRow<T> {
    pub prefix: Option<RegionHistoryBase>,
    pub earlier: Vec<T>,
}

impl RegionHistoryImage {
    pub fn direct_bases(
        &self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    ) -> Vec<RelationalDurableMemberLocator> {
        self.entities
            .iter()
            .filter_map(|row| row.prefix.as_ref())
            .chain(self.relations.iter().filter_map(|row| row.prefix.as_ref()))
            .map(|base| base.member)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionAdjacencyKinds {
    pub slot: u64,
    pub kinds: Vec<(KindId, Vec<RelationId>)>,
}

impl<'bytes> RelationalRegionImage<'bytes> {
    pub fn encode(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        checkpoint: &mut dyn FnMut(u64, u64) -> Result<(), Cause>,
        partition_id: PartitionId,
        schema_carrier_digest: [u8; 32],
        kind: RelationalRegionImageKind,
        payload: &impl Serialize,
    ) -> Result<(RelationalDurableMemberLocator, Vec<u8>), Cause> {
        let mut payload_bytes = Vec::new();
        let mut writer = AdmittedImageWriter {
            bytes: &mut payload_bytes,
            checkpoint,
            stopped: None,
        };
        let encoded = rmp_serde::encode::write(&mut writer, payload);
        if let Some(cause) = writer.stopped {
            return Err(cause);
        }
        encoded.map_err(|_| Cause::Damaged)?;
        let payload = payload_bytes;
        checkpoint(
            payload.len() as u64,
            (MAGIC.len() + 39 + payload.len()) as u64,
        )?;
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

    pub fn decode(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        checkpoint: &mut dyn FnMut(u64, u64) -> Result<(), Cause>,
        bytes: &'bytes [u8],
        locator: RelationalDurableMemberLocator,
        partition_id: PartitionId,
        schema_carrier_digest: [u8; 32],
        kind: RelationalRegionImageKind,
    ) -> Result<Self, Cause> {
        Self::decode_body(
            checkpoint,
            bytes,
            locator,
            partition_id,
            schema_carrier_digest,
            kind,
        )
    }

    fn decode_body(
        checkpoint: &mut dyn FnMut(u64, u64) -> Result<(), Cause>,
        bytes: &'bytes [u8],
        locator: RelationalDurableMemberLocator,
        partition_id: PartitionId,
        schema_carrier_digest: [u8; 32],
        kind: RelationalRegionImageKind,
    ) -> Result<Self, Cause> {
        checkpoint(1, 0)?;
        if bytes.len() as u64 != locator.length
            || <[u8; 32]>::from(Sha256::digest(bytes)) != locator.sha256
        {
            return Err(Cause::IntegrityMismatch);
        }
        let frame = bytes
            .strip_prefix(MAGIC)
            .ok_or_else(|| obsolete_frame(bytes))?;
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

impl RelationalRegionImage<'_> {
    pub(super) fn encode_for_request(
        partition_id: PartitionId,
        schema_digest: [u8; 32],
        kind: RelationalRegionImageKind,
        payload: &impl Serialize,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            RelationalDurableMemberLocator,
            worth_execution::ExecutionImmutableBytes,
        ),
        crate::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
        let mut header = [0u8; MAGIC.len() + 39];
        header[..MAGIC.len()].copy_from_slice(MAGIC);
        let frame = &mut header[MAGIC.len()..];
        frame[..2].copy_from_slice(&VERSION.to_be_bytes());
        frame[2..6].copy_from_slice(&partition_id.as_u32().to_be_bytes());
        frame[6..38].copy_from_slice(&schema_digest);
        frame[38] = match kind {
            RelationalRegionImageKind::Head => 0,
            RelationalRegionImageKind::History => 1,
        };
        let bytes = crate::durability::artifact_payload_emission::request::encode_compact(
            &header, payload, request,
        )?;
        let locator = RelationalDurableMemberLocator {
            sha256: Sha256::digest(bytes.as_ref()).into(),
            length: bytes.len() as u64,
        };
        Ok((locator, bytes))
    }
}

/// A typed version refusal for another `WORTH-REL-REG<n>` grammar; anything
/// else is damage. No earlier region grammar is decoded.
fn obsolete_frame(bytes: &[u8]) -> Cause {
    let version = bytes.strip_prefix(FAMILY).and_then(|label| {
        let end = label.iter().take(6).position(|byte| *byte == NEWLINE)?;
        let digits = &label[..end];
        if digits.is_empty() || digits[0] == b'0' || !digits.iter().all(u8::is_ascii_digit) {
            return None;
        }
        let found = std::str::from_utf8(digits).ok()?.parse::<u16>().ok()?;
        (found != VERSION).then_some(found)
    });
    match version {
        Some(found) => Cause::UnsupportedRegionImageVersion {
            found,
            supported: VERSION,
        },
        None => Cause::Damaged,
    }
}
const NEWLINE: u8 = 0x0a;

struct AdmittedImageWriter<'a> {
    bytes: &'a mut Vec<u8>,
    checkpoint: &'a mut dyn FnMut(u64, u64) -> Result<(), Cause>,
    stopped: Option<Cause>,
}
impl std::io::Write for AdmittedImageWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Err(cause) = (self.checkpoint)(0, bytes.len() as u64) {
            self.stopped = Some(cause);
            return Err(std::io::Error::other("original checkpoint request stopped"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn obsolete_and_future_region_grammars_refuse_with_their_typed_version() {
        let decode_err = |bytes: &[u8]| {
            let loc = crate::storage::data::member_frame::locator(bytes);
            RelationalRegionImage::decode_body(
                &mut |_, _| Ok(()),
                bytes,
                loc,
                PartitionId(0),
                [0; 32],
                RelationalRegionImageKind::Head,
            )
            .err()
            .unwrap()
        };
        for (frame, found) in [(&b"WORTH-REL-REG7\n"[..], 7), (&b"WORTH-REL-REG9\n"[..], 9)] {
            let bytes = [frame, &[0; 40]].concat();
            let expected = Cause::UnsupportedRegionImageVersion {
                found,
                supported: VERSION,
            };
            assert_eq!(obsolete_frame(&bytes), expected);
            assert_eq!(decode_err(&bytes), expected);
        }
        for frame in [
            &b"WORTH-REL-REGX\n"[..],
            b"WORTH-REL-REG0\n",
            b"WORTH-REL-REG08\n",
            b"WORTH-REL-REG+8\n",
            b"WORTH-REL-REG+7\n",
            b"WORTH-REL-REG07\n",
            b"WORTH-REL-CHK25\n",
            b"",
        ] {
            let bytes = [frame, &[0; 40]].concat();
            assert_eq!(obsolete_frame(&bytes), Cause::Damaged);
            assert_eq!(decode_err(&bytes), Cause::Damaged);
        }
        assert_eq!(MAGIC, b"WORTH-REL-REG8\n");
    }
}
