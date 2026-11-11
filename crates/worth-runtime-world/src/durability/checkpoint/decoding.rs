//! Decode only the inline image supplied by the protected Home selection.
use super::*;
use crate::durability::{
    codec::{decoded, fields::Read},
    RecoveredCustodyDisposition, WorldDurableFrameDenial as Frame, WorldDurableHomeBinding,
    WorldDurableMediumDenial as Medium, WorldReadmissionDenial as Denial, WorldRecord,
    WorldRelationalReleaseEvidence,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion,
    DurableCompositePublicationAttemptIdentity, HomeIncarnation,
};

impl WorldCheckpointImage {
    pub(crate) fn decode_selected(
        bytes: &[u8],
        binding: &WorldDurableHomeBinding,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let payload = super::framing::payload(bytes, request).map_err(Denial::Medium)?;
        let mut read = Read::new(payload, binding.owner().home());
        let capture_incarnation = read_binding(&mut read, binding, request)?;
        let coverage = read_coverage(&mut read)?;
        let frontiers = read_frontiers(&mut read)?;
        let mut image = Self::empty(binding);
        image.owner = match read.fixed::<1>().map_err(frame)? {
            [0] => None,
            [1] => {
                let (position, digest) = location(&mut read)?;
                Some(WorldOwnerRecordLocation { position, digest })
            }
            _ => return Err(frame(Frame::Malformed)),
        };
        read_rows(&mut read, &mut image, true, request)?;
        read_rows(&mut read, &mut image, false, request)?;
        read_dispositions(&mut read, &mut image, request)?;
        read_occurrences(&mut read, &mut image, capture_incarnation, request)?;
        for created in image.created.values() {
            if created.created().publishing_incarnation > capture_incarnation {
                return Err(Denial::FutureIncarnation);
            }
        }
        image.resolve_collected_history(request)?;
        image.validate_open_identities(request)?;
        image.validate_image_coordinates(&coverage, request)?;
        if [
            image.frontiers.branch,
            image.frontiers.commit,
            image.frontiers.bootstrap,
            image.frontiers.attempt,
            image.frontiers.custody,
        ] != frontiers
        {
            return Err(Denial::DuplicateIdentity);
        }
        super::graph::validate(&mut read, &image, request)?;
        read.finished().map_err(frame)?;
        image.coverage = coverage;
        Ok(image)
    }
}
fn frame(cause: Frame) -> Denial {
    Denial::Medium(Medium::Frame(cause))
}
fn read_binding(
    read: &mut Read<'_>,
    binding: &WorldDurableHomeBinding,
    request: &ExecutionRequest<'_, '_>,
) -> Result<HomeIncarnation, Denial> {
    let row_version =
        BoundaryProtocolVersion::try_new(u32::from_le_bytes(read.fixed().map_err(frame)?))
            .map_err(|_| frame(Frame::Malformed))?;
    BoundaryProtocolCompatibilityWindow::inclusive(
        crate::durability::WORLD_ARTIFACT_VERSION,
        crate::durability::WORLD_ARTIFACT_VERSION,
    )
    .admit(row_version)
    .map_err(|cause| frame(Frame::UnsupportedVersion(cause)))?;
    if read.owner().map_err(frame)? != binding.owner() {
        return Err(Denial::ForeignOwner);
    }
    let incarnation = HomeIncarnation::from_recorded(read.number().map_err(frame)?)
        .ok_or_else(|| frame(Frame::Malformed))?;
    if incarnation > binding.incarnation() {
        return Err(Denial::FutureIncarnation);
    }
    if !binding.admits_checkpoint_revision(read.fixed::<32>().map_err(frame)?, request)? {
        return Err(Denial::ProgramRevisionMismatch);
    }
    Ok(incarnation)
}
fn read_coverage(read: &mut Read<'_>) -> Result<WorldCheckpointCoverage, Denial> {
    let mut next = || {
        let (position, digest) = location(read)?;
        if position == 0 && digest != [0; 32] {
            return Err(Denial::JournalSourceMismatch);
        }
        Ok(WorldJournalCoverage { position, digest })
    };
    Ok(WorldCheckpointCoverage {
        branches: next()?,
        custody: next()?,
        occurrences: next()?,
    })
}
fn read_frontiers(read: &mut Read<'_>) -> Result<[u64; 5], Denial> {
    let mut values = [0; 5];
    for value in &mut values {
        *value = read.number().map_err(frame)?;
    }
    Ok(values)
}
pub(super) fn count(read: &mut Read<'_>, minimum: usize) -> Result<usize, Denial> {
    let count =
        usize::try_from(read.number().map_err(frame)?).map_err(|_| frame(Frame::SizeLimit))?;
    if count > read.remaining() / minimum {
        return Err(frame(Frame::Malformed));
    }
    Ok(count)
}
fn location(read: &mut Read<'_>) -> Result<(u64, [u8; 32]), Denial> {
    Ok((read.number().map_err(frame)?, read.fixed().map_err(frame)?))
}
fn bytes<'a>(read: &mut Read<'a>, maximum: usize) -> Result<&'a [u8], Denial> {
    let size =
        usize::try_from(read.number().map_err(frame)?).map_err(|_| frame(Frame::SizeLimit))?;
    if size > maximum {
        return Err(frame(Frame::SizeLimit));
    }
    read.take(size).map_err(frame)
}
fn read_rows(
    read: &mut Read<'_>,
    image: &mut WorldCheckpointImage,
    branch: bool,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    for _ in 0..count(read, 49)? {
        let (position, digest) = location(read)?;
        let row = bytes(read, super::framing::MAXIMUM)?;
        let record =
            decoded::admit(row, image.binding.owner().home(), request).map_err(Denial::Medium)?;
        let allowed = match &record.record {
            WorldRecord::Branch { .. } | WorldRecord::BranchCreated(_) => branch,
            WorldRecord::OpenAttempt(_) | WorldRecord::CreationIntent(_) => !branch,
            WorldRecord::Owner { .. }
            | WorldRecord::Occurrence { .. }
            | WorldRecord::Released { .. }
            | WorldRecord::Continued { .. }
            | WorldRecord::CreationReleased { .. } => false,
        };
        if !allowed {
            return Err(Denial::WrongJournal);
        }
        if record.record.owner() != image.binding.owner() {
            return Err(Denial::ForeignOwner);
        }
        image
            .frontiers
            .include_record(&record.record)
            .map_err(Denial::IdentityExhaustion)?;
        image.admit_record(record, position, digest, request)?;
    }
    Ok(())
}
fn attempt(
    read: &mut Read<'_>,
    image: &WorldCheckpointImage,
) -> Result<DurableCompositePublicationAttemptIdentity, Denial> {
    Ok(DurableCompositePublicationAttemptIdentity::from_recorded(
        image.binding.owner(),
        read.number().map_err(frame)?,
    ))
}
fn read_dispositions(
    read: &mut Read<'_>,
    image: &mut WorldCheckpointImage,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    for _ in 0..count(read, 49)? {
        super::framing::charge(request, 1).map_err(Denial::Medium)?;
        let (position, digest) = location(read)?;
        let attempt = attempt(read, image)?;
        let tag = read.fixed::<1>().map_err(frame)?;
        if tag == [3] {
            let intent = image
                .creations
                .get(&attempt)
                .ok_or(Denial::MissingCreationIntent)?
                .clone();
            image.admit_creation_release(
                &intent.intent().association,
                intent.intent().custody,
                position,
                digest,
                request,
            )?;
            continue;
        }
        let value = match tag {
            [0] => RecoveredCustodyDisposition::Released(None),
            [1] => RecoveredCustodyDisposition::Released(Some(
                WorldRelationalReleaseEvidence::from_json_in(
                    request,
                    bytes(read, super::framing::MAXIMUM)?,
                )
                .map_err(Denial::Medium)?,
            )),
            [2] => RecoveredCustodyDisposition::Continued,
            _ => return Err(frame(Frame::Malformed)),
        };
        let open = image
            .opens
            .get(&attempt)
            .ok_or(Denial::MissingOpenAttempt)?
            .clone();
        image.admit_disposition(
            &open.association,
            open.custody,
            value,
            position,
            digest,
            request,
        )?;
    }
    Ok(())
}
fn read_occurrences(
    read: &mut Read<'_>,
    image: &mut WorldCheckpointImage,
    capture_incarnation: HomeIncarnation,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let mut previous_position = 0u64;
    for _ in 0..count(read, 105)? {
        super::framing::charge(request, 1).map_err(Denial::Medium)?;
        let (position, digest) = location(read)?;
        if previous_position.checked_add(1) != Some(position) {
            return Err(Denial::JournalSourceMismatch);
        }
        let attempt = attempt(read, image)?;
        let owners = crate::durability::codec::owner_evidence::read(read, request)
            .map_err(Denial::Medium)?;
        let publishing = HomeIncarnation::from_recorded(read.number().map_err(frame)?)
            .ok_or_else(|| frame(Frame::Malformed))?;
        if publishing > capture_incarnation {
            return Err(Denial::FutureIncarnation);
        }
        image.collect_occurrence(attempt, owners, publishing, position, digest, request)?;
        previous_position = position;
    }
    Ok(())
}
