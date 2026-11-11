//! Complete Home sections have an additive envelope; ordinary section v3 is unchanged.
use super::RelationalSectionCheckpointDenial as Capture;
use worth_foundational::facade::BoundaryProtocolVersion;
use worth_foundational::CanonicalDigestId;
use worth_relational::facade::durability::CompleteRelationalNativeCheckpoint;

const MAGIC: &[u8; 8] = b"WSRCSRC1";
const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
pub(super) const HEADER: usize = 66;

pub(super) fn encode(complete: &CompleteRelationalNativeCheckpoint) -> Result<Vec<u8>, Capture> {
    let source = complete.source_artifact();
    let mut bytes = Vec::with_capacity(
        HEADER
            .checked_add(source.as_bytes().len())
            .ok_or(Capture::SizeLimit)?,
    );
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(VERSION.get() as u16).to_le_bytes());
    bytes.extend_from_slice(&source.covered().0.to_le_bytes());
    bytes.extend_from_slice(&source.covered_transitions().0.to_le_bytes());
    let (revision, digest) = source.initial_coordinate();
    bytes.extend_from_slice(&revision.to_le_bytes());
    bytes.extend_from_slice(&digest);
    bytes.extend_from_slice(source.as_bytes());
    Ok(bytes)
}

pub(super) struct CompleteSectionCoordinates {
    pub(super) covered: u64,
    pub(super) through: u64,
    pub(super) revision: u64,
    pub(super) digest: CanonicalDigestId,
}
pub(super) fn coordinates(bytes: &[u8]) -> Option<CompleteSectionCoordinates> {
    if let Some(coordinates) = super::admission_section::coordinates(bytes) {
        return Some(coordinates);
    }
    if bytes.get(..8)? != MAGIC
        || u16::from_le_bytes(bytes.get(8..10)?.try_into().ok()?) != VERSION.get() as u16
    {
        return None;
    }
    let integer = |offset| {
        Some(u64::from_le_bytes(
            bytes.get(offset..offset + 8)?.try_into().ok()?,
        ))
    };
    Some(CompleteSectionCoordinates {
        covered: integer(10)?,
        through: integer(18)?,
        revision: integer(26)?,
        digest: CanonicalDigestId::new(bytes.get(34..HEADER)?.try_into().ok()?),
    })
}
