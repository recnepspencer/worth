//! Binary codec helpers for `WHBM4` per-basis chunk descriptor manifest pages
//! (Store Milestone 4, Phase M4.4).

use sha2::{Digest, Sha256};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::{CanonicalCursorKey, CanonicalKeyDomain, HistoricalReadDenial};

use super::basis_manifest::{BasisManifestChildRef, RouteMaxKey, MAX_INDEX_SEPARATOR_BYTES};
use super::chunk_format::{ChunkMemberId, SealedChunkDescriptor};

const MANIFEST_SEAL_DOMAIN: &[u8] = b"worth.historical.basis-manifest.v4";

pub(super) fn write_child_ref(body: &mut Vec<u8>, child: &BasisManifestChildRef) {
    write_route_max(body, &child.route_max);
    body.extend_from_slice(&child.member.sha256);
    body.extend_from_slice(&child.member.length.to_le_bytes());
    body.extend_from_slice(&child.locator);
    body.extend_from_slice(&child.rightmost_leaf_member.sha256);
    body.extend_from_slice(&child.rightmost_leaf_member.length.to_le_bytes());
    body.extend_from_slice(&child.rightmost_leaf_locator);
}

pub(super) fn read_child_ref(
    r: &mut Reader<'_>,
) -> Result<BasisManifestChildRef, HistoricalReadDenial> {
    let route_max = read_route_max(r)?;
    let member = ChunkMemberId {
        sha256: r.array::<32>()?,
        length: r.u64()?,
    };
    let locator = r.array::<64>()?;
    let rightmost_leaf_member = ChunkMemberId {
        sha256: r.array::<32>()?,
        length: r.u64()?,
    };
    let rightmost_leaf_locator = r.array::<64>()?;
    Ok(BasisManifestChildRef {
        route_max,
        member,
        locator,
        rightmost_leaf_member,
        rightmost_leaf_locator,
    })
}

pub(super) fn read_chunk_descriptor(
    r: &mut Reader<'_>,
) -> Result<SealedChunkDescriptor, HistoricalReadDenial> {
    let semantic_commitment = r.array::<32>()?;
    let max_key = read_cursor_key(r)?;
    let chunk_member = ChunkMemberId {
        sha256: r.array::<32>()?,
        length: r.u64()?,
    };
    let chunk_locator = match r.u8()? {
        0 => None,
        1 => Some(r.array::<64>()?),
        _ => return Err(corrupt()),
    };
    let continuation_manifest_member = match r.u8()? {
        0 => None,
        1 => Some(ChunkMemberId {
            sha256: r.array::<32>()?,
            length: r.u64()?,
        }),
        _ => return Err(corrupt()),
    };
    let continuation_manifest_locator = match r.u8()? {
        0 => None,
        1 => Some(r.array::<64>()?),
        _ => return Err(corrupt()),
    };
    let continuation_count = r.u32()?;
    Ok(SealedChunkDescriptor {
        semantic_commitment,
        min_key: CanonicalCursorKey::Start,
        max_key,
        chunk_member,
        continuation_members: Vec::new(),
        chunk_locator,
        continuation_locators: Vec::new(),
        continuation_manifest_member,
        continuation_manifest_locator,
        continuation_count,
    })
}

pub(super) fn encode_chunk_descriptor(body: &mut Vec<u8>, chunk: &SealedChunkDescriptor) {
    body.extend_from_slice(&chunk.semantic_commitment);
    write_cursor_key(body, &chunk.max_key);
    body.extend_from_slice(&chunk.chunk_member.sha256);
    body.extend_from_slice(&chunk.chunk_member.length.to_le_bytes());
    match &chunk.chunk_locator {
        Some(loc) => {
            body.push(1);
            body.extend_from_slice(loc);
        }
        None => body.push(0),
    }
    match chunk.continuation_manifest_member {
        Some(member) => {
            body.push(1);
            body.extend_from_slice(&member.sha256);
            body.extend_from_slice(&member.length.to_le_bytes());
        }
        None => body.push(0),
    }
    match &chunk.continuation_manifest_locator {
        Some(loc) => {
            body.push(1);
            body.extend_from_slice(loc);
        }
        None => body.push(0),
    }
    body.extend_from_slice(&chunk.continuation_count.to_le_bytes());
}

pub(super) fn write_route_max(out: &mut Vec<u8>, route: &RouteMaxKey) {
    if route.is_start {
        out.push(0);
    } else {
        out.push(if route.truncated { 2 } else { 1 });
        out.extend_from_slice(&route.partition_id.to_le_bytes());
        out.push(route.domain.tag());
        out.extend_from_slice(&(route.prefix.len() as u16).to_le_bytes());
        out.extend_from_slice(&route.prefix);
    }
}

pub(super) fn read_route_max(r: &mut Reader<'_>) -> Result<RouteMaxKey, HistoricalReadDenial> {
    match r.u8()? {
        0 => Ok(RouteMaxKey {
            is_start: true,
            partition_id: 0,
            domain: CanonicalKeyDomain::Entity,
            prefix: Vec::new(),
            truncated: false,
        }),
        tag @ (1 | 2) => {
            let partition_id = r.u32()?;
            let domain = CanonicalKeyDomain::from_tag(r.u8()?).ok_or_else(corrupt)?;
            let len = r.u16()? as usize;
            if len > MAX_INDEX_SEPARATOR_BYTES {
                return Err(corrupt());
            }
            let prefix = r.take(len)?.to_vec();
            Ok(RouteMaxKey {
                is_start: false,
                partition_id,
                domain,
                prefix,
                truncated: tag == 2,
            })
        }
        _ => Err(corrupt()),
    }
}

fn write_cursor_key(out: &mut Vec<u8>, key: &CanonicalCursorKey) {
    match key {
        CanonicalCursorKey::Start => out.push(0),
        CanonicalCursorKey::After {
            partition_id,
            domain,
            logical_key,
        } => {
            out.push(1);
            out.extend_from_slice(&partition_id.to_le_bytes());
            out.push(domain.tag());
            out.extend_from_slice(&(logical_key.len() as u32).to_le_bytes());
            out.extend_from_slice(logical_key);
        }
    }
}

fn read_cursor_key(r: &mut Reader<'_>) -> Result<CanonicalCursorKey, HistoricalReadDenial> {
    match r.u8()? {
        0 => Ok(CanonicalCursorKey::Start),
        1 => {
            let pid = PartitionId(r.u32()?);
            let dom = CanonicalKeyDomain::from_tag(r.u8()?).ok_or_else(corrupt)?;
            let len = r.u32()? as usize;
            CanonicalCursorKey::after(pid, dom, r.take(len)?.to_vec()).ok_or_else(corrupt)
        }
        _ => Err(corrupt()),
    }
}

pub(super) fn manifest_seal(body: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(MANIFEST_SEAL_DOMAIN);
    h.update(body);
    h.finalize().into()
}

pub(super) fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}

pub(super) struct Reader<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) at: usize,
}

impl<'a> Reader<'a> {
    pub(super) fn bounded_count(&mut self, min_elem: usize) -> Result<usize, HistoricalReadDenial> {
        let count = self.u32()? as usize;
        if min_elem > 0 && count > self.bytes.len().saturating_sub(self.at) / min_elem {
            return Err(corrupt());
        }
        Ok(count)
    }
    pub(super) fn take(&mut self, len: usize) -> Result<&'a [u8], HistoricalReadDenial> {
        let end = self.at.checked_add(len).ok_or_else(corrupt)?;
        let slice = self.bytes.get(self.at..end).ok_or_else(corrupt)?;
        self.at = end;
        Ok(slice)
    }
    pub(super) fn u8(&mut self) -> Result<u8, HistoricalReadDenial> {
        Ok(self.take(1)?[0])
    }
    pub(super) fn u16(&mut self) -> Result<u16, HistoricalReadDenial> {
        Ok(u16::from_le_bytes(self.array::<2>()?))
    }
    pub(super) fn u32(&mut self) -> Result<u32, HistoricalReadDenial> {
        Ok(u32::from_le_bytes(self.array::<4>()?))
    }
    pub(super) fn u64(&mut self) -> Result<u64, HistoricalReadDenial> {
        Ok(u64::from_le_bytes(self.array::<8>()?))
    }
    pub(super) fn array<const N: usize>(&mut self) -> Result<[u8; N], HistoricalReadDenial> {
        Ok(self.take(N)?.try_into().expect("N"))
    }
}
