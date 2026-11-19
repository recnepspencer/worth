//! Coordinates of a selected Complete section, read for the stale-capture check.
//! Home admission sections are read first; the earlier plain envelope is only read.
use worth_foundational::facade::BoundaryProtocolVersion;
use worth_foundational::CanonicalDigestId;

const MAGIC: &[u8; 8] = b"WSRCSRC1";
const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
pub(super) const HEADER: usize = 66;

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
