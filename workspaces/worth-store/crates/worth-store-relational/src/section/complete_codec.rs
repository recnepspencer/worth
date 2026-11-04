//! Complete Home sections have an additive envelope; ordinary section v3 is unchanged.
use super::{
    RelationalSectionCheckpointDenial as Capture, RelationalSectionReadmissionDenial as Read,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion};
use worth_foundational::CanonicalDigestId;
use worth_relational::facade::durability::{
    CompleteRelationalNativeCheckpoint, RelationalCompleteSourceArtifact,
    RelationalDurableCheckpointSource,
};

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

pub(super) fn decode(
    backing: ExecutionImmutableBytes,
    region: std::ops::Range<usize>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableCheckpointSource, Read> {
    let raw = backing.get(region.clone()).ok_or(Read::MalformedSection)?;
    if raw.get(..8) == Some(b"WSRCKPT1".as_slice()) {
        return Err(Read::InitialInputs(
            worth_relational::facade::durability::RelationalInitialInputsRefusal::Missing,
        ));
    }
    if raw.get(..8) != Some(MAGIC.as_slice()) {
        return Err(Read::MalformedSection);
    }
    let produced = BoundaryProtocolVersion::try_new(u32::from(u16::from_le_bytes(
        raw.get(8..10)
            .ok_or(Read::MalformedSection)?
            .try_into()
            .map_err(|_| Read::MalformedSection)?,
    )))
    .map_err(|_| Read::MalformedSection)?;
    BoundaryProtocolCompatibilityWindow::inclusive(VERSION, VERSION)
        .admit(produced)
        .map_err(Read::UnsupportedCompleteSectionVersion)?;
    let coordinates = coordinates(raw).ok_or(Read::MalformedSection)?;
    let source = RelationalCompleteSourceArtifact::from_untrusted_region(
        backing,
        region
            .start
            .checked_add(HEADER)
            .ok_or(Read::MalformedSection)?..region.end,
        request,
    )
    .map_err(Read::InitialInputs)?;
    if source.covered().0 != coordinates.covered
        || source.covered_transitions().0 != coordinates.through
        || source.initial_coordinate() != (coordinates.revision, *coordinates.digest.bytes())
    {
        return Err(Read::MalformedSection);
    }
    RelationalDurableCheckpointSource::from_complete_artifact(source).map_err(Read::InitialInputs)
}

pub(super) struct CompleteSectionCoordinates {
    pub(super) covered: u64,
    pub(super) through: u64,
    pub(super) revision: u64,
    pub(super) digest: CanonicalDigestId,
}
pub(super) fn coordinates(bytes: &[u8]) -> Option<CompleteSectionCoordinates> {
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
