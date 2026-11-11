//! One declared Home package binds Native admission and the opaque World companion.
use super::{
    RelationalSectionCheckpointDenial as Capture, RelationalSectionReadmissionDenial as Read,
};
use sha2::{Digest, Sha256};
use std::ops::Range;
use worth_execution::{ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::*;
use worth_relational::facade::durability::{
    CheckpointAdmissionAuthenticationMismatch as Authentication,
    CheckpointAdmissionBindingMismatch as Binding, RelationalCheckpointAdmissionArtifact as Image,
    RelationalCheckpointAdmissionRefusal as Refusal, RelationalCompleteSourceArtifact,
    RelationalDurableCheckpointSource,
};

const MAGIC: &[u8; 8] = b"WSRCAI01";
const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
pub(super) const PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.store.relational.admission-section");
const HEADER: usize = 292;

pub(super) struct DecodedHomeAdmission {
    pub(super) checkpoint: RelationalDurableCheckpointSource,
    pub(super) image: Image,
    pub(super) backing: ExecutionImmutableBytes,
    pub(super) companion: Range<usize>,
}

pub(super) fn encode(
    source: &RelationalCompleteSourceArtifact,
    image: &Image,
    companion: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(Vec<u8>, Option<ExecutionMemoryReservation>), Capture> {
    let scope = image.cut().scope();
    let CheckpointAdmissionScope::Home {
        home,
        native_origin,
        world_through,
        world_image_digest,
        ..
    } = scope
    else {
        return Err(Capture::AdmissionImage(Refusal::BindingMismatch(
            Binding::Scope,
        )));
    };
    RelationalDurableCheckpointSource::from_checkpoint_admission_image(
        source.clone(),
        image.clone(),
        request,
    )
    .map_err(Capture::AdmissionImage)?;
    let children = [source.as_bytes(), image.as_bytes(), companion];
    if children.iter().any(|bytes| bytes.is_empty()) {
        return Err(Capture::AdmissionImage(Refusal::MissingArtifact));
    }
    let size = children
        .iter()
        .try_fold(HEADER, |size, child| size.checked_add(child.len()))
        .filter(|size| *size <= 1 << 30)
        .ok_or(Capture::SizeLimit)?;
    let memory = request
        .reserve_retained_memory(size as u64)
        .map_err(Capture::Execution)?;
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.get().to_le_bytes());
    for value in [
        source.covered().0,
        source.covered_transitions().0,
        source.initial_coordinate().0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(native_origin);
    bytes.extend_from_slice(&home.bytes());
    for value in world_through {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(image.cut().checkpoint_digest());
    bytes.extend_from_slice(world_image_digest);
    for child in children {
        bytes.extend_from_slice(&(child.len() as u64).to_le_bytes());
    }
    for child in children {
        bytes.extend_from_slice(&digest(child, request).map_err(Capture::AdmissionImage)?);
    }
    for child in children {
        for chunk in child.chunks(64 << 10) {
            charge(chunk.len(), request).map_err(Capture::AdmissionImage)?;
            bytes.extend_from_slice(chunk);
        }
    }
    Ok((bytes, memory))
}

pub(super) fn decode(
    backing: ExecutionImmutableBytes,
    region: Range<usize>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<DecodedHomeAdmission, Read> {
    let raw = backing.get(region.clone()).ok_or(Read::MalformedSection)?;
    predecessor(raw).map_err(Read::AdmissionImage)?;
    if raw.get(..8) != Some(MAGIC.as_slice()) || raw.len() > 1 << 30 {
        return Err(Read::MalformedSection);
    }
    let produced = BoundaryProtocolVersion::try_new(u32::from_le_bytes(
        take(raw, 8).map_err(Read::AdmissionImage)?,
    ))
    .map_err(|_| Read::AdmissionImage(Refusal::InvalidVersion))?;
    BoundaryProtocolCompatibilityWindow::inclusive(VERSION, VERSION)
        .admit(produced)
        .map_err(|cause| Read::AdmissionImage(Refusal::UnsupportedVersion(cause)))?;
    if raw.len() < HEADER {
        return Err(Read::MalformedSection);
    }
    let home = HomeIdentity::from_observed_bytes(take(raw, 68).map_err(Read::AdmissionImage)?)
        .map_err(|_| Read::MalformedSection)?;
    let integer = |offset| -> Result<u64, Read> {
        Ok(u64::from_le_bytes(
            take(raw, offset).map_err(Read::AdmissionImage)?,
        ))
    };
    let expected = CheckpointAdmissionCut::new(
        CheckpointAdmissionScope::Home {
            home,
            native_origin: take(raw, 36).map_err(Read::AdmissionImage)?,
            complete_source_digest: take(raw, 196).map_err(Read::AdmissionImage)?,
            world_through: [integer(84)?, integer(92)?, integer(100)?],
            world_image_digest: take(raw, 140).map_err(Read::AdmissionImage)?,
        },
        integer(12)?,
        integer(20)?,
        take(raw, 108).map_err(Read::AdmissionImage)?,
    );
    let mut children = [0..0, 0..0, 0..0];
    let mut cursor = HEADER;
    for (index, range) in children.iter_mut().enumerate() {
        let length =
            usize::try_from(integer(172 + index * 8)?).map_err(|_| Read::MalformedSection)?;
        if length == 0 {
            return Err(Read::AdmissionImage(Refusal::MissingArtifact));
        }
        let end = cursor
            .checked_add(length)
            .filter(|end| *end <= raw.len())
            .ok_or(Read::MalformedSection)?;
        let expected: [u8; 32] = take(raw, 196 + index * 32).map_err(Read::AdmissionImage)?;
        if digest(&raw[cursor..end], request).map_err(Read::AdmissionImage)? != expected {
            return Err(Read::AdmissionImage(Refusal::AuthenticationMismatch(
                Authentication::Directory,
            )));
        }
        *range = region.start + cursor..region.start + end;
        cursor = end;
    }
    if cursor != raw.len() {
        return Err(Read::MalformedSection);
    }
    let source = RelationalCompleteSourceArtifact::from_untrusted_region(
        backing.clone(),
        children[0].clone(),
        request,
    )
    .map_err(Read::InitialInputs)?;
    if source.initial_coordinate().0 != integer(28)? {
        return Err(Read::MalformedSection);
    }
    let image = Image::from_untrusted_region(backing.clone(), children[1].clone(), request)
        .map_err(Read::AdmissionImage)?;
    if image.cut() != &expected {
        return Err(Read::AdmissionImage(Refusal::BindingMismatch(Binding::Cut)));
    }
    let checkpoint = RelationalDurableCheckpointSource::from_checkpoint_admission_image(
        source,
        image.clone(),
        request,
    )
    .map_err(Read::AdmissionImage)?;
    Ok(DecodedHomeAdmission {
        checkpoint,
        image,
        backing,
        companion: children[2].clone(),
    })
}

pub(super) fn coordinates(
    bytes: &[u8],
) -> Option<super::complete_codec::CompleteSectionCoordinates> {
    if bytes.get(..8)? != MAGIC || u32::from_le_bytes(take(bytes, 8).ok()?) != VERSION.get() {
        return None;
    }
    Some(super::complete_codec::CompleteSectionCoordinates {
        covered: u64::from_le_bytes(take(bytes, 12).ok()?),
        through: u64::from_le_bytes(take(bytes, 20).ok()?),
        revision: u64::from_le_bytes(take(bytes, 28).ok()?),
        digest: CanonicalDigestId::new(take(bytes, 36).ok()?),
    })
}

fn predecessor(raw: &[u8]) -> Result<(), Refusal> {
    let protocol = match raw.get(..8) {
        Some(b"WSRCSRC1") => {
            BoundaryProtocolIdentity::new("worth.store.relational.complete-section")
        }
        Some(b"WSRCKPT1") => {
            BoundaryProtocolIdentity::new("worth.store.relational.checkpoint-section")
        }
        Some(_) => return Ok(()),
        None => return Err(Refusal::MissingArtifact),
    };
    let version = BoundaryProtocolVersion::try_new(u16::from_le_bytes(take(raw, 8)?) as u32)
        .map_err(|_| Refusal::InvalidVersion)?;
    let next = version
        .get()
        .checked_add(1)
        .ok_or(Refusal::InvalidVersion)?;
    let retired = BoundaryProtocolCompatibilityWindow::inclusive(version, version)
        .retire_before(BoundaryProtocolVersion::new(next))
        .admit(version)
        .unwrap_err();
    Err(Refusal::PredecessorFormat {
        protocol,
        version: retired,
    })
}
fn take<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N], Refusal> {
    bytes
        .get(offset..offset.checked_add(N).ok_or(Refusal::CapacityOverflow)?)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(Refusal::Malformed)
}
pub(super) fn digest(
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], Refusal> {
    let mut hash = Sha256::new();
    for chunk in bytes.chunks(64 << 10) {
        charge(chunk.len(), request)?;
        hash.update(chunk);
    }
    Ok(hash.finalize().into())
}
fn charge(bytes: usize, request: &ExecutionRequest<'_, '_>) -> Result<(), Refusal> {
    request
        .run_native_with_checkpoint(|context| context.checkpoint(bytes as u64))
        .map_err(|cause| Refusal::Request(cause.into()))?
        .0
        .map_err(|cause| {
            Refusal::Request(worth_execution::WorkCeilingDenial::Stopped(cause).into())
        })
}
