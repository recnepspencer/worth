use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    ExtentArenaRange, PersistedPhysicalRecoveryFrame, RecordArtifactFile, RecordFrameCoordinate,
};

use super::{ExecutionBasisDenial, RecoveryStagingCommandPlan};

pub(super) fn exact_commands(
    frames: impl IntoIterator<Item = PersistedPhysicalRecoveryFrame>,
    manifests: impl IntoIterator<Item = (RecordFrameCoordinate, Box<[u8]>)>,
    source_artifacts: &[RecordArtifactFile],
    protected_ranges: &[ExtentArenaRange],
    destination_ranges: &[ExtentArenaRange],
) -> Result<Box<[RecoveryStagingCommandPlan]>, ExecutionBasisDenial> {
    let mut grouped = std::collections::BTreeMap::<
        RecordArtifactFile,
        std::collections::BTreeMap<u64, Box<[u8]>>,
    >::new();
    for frame in frames {
        let coordinate = frame.coordinate();
        let old = grouped
            .entry(coordinate.artifact())
            .or_default()
            .insert(coordinate.offset(), frame.bytes().into());
        if old.is_some() {
            return Err(ExecutionBasisDenial::Invalid);
        }
    }
    for (coordinate, bytes) in manifests {
        if grouped
            .entry(coordinate.artifact())
            .or_default()
            .insert(coordinate.offset(), bytes)
            .is_some()
        {
            return Err(ExecutionBasisDenial::Invalid);
        }
    }
    let mut commands = Vec::new();
    for (artifact, ranges) in grouped {
        if let RecordArtifactFile::ExtentArena { arena } = artifact {
            for range in destination_ranges
                .iter()
                .filter(|range| range.arena().get() == arena)
            {
                if protected_ranges
                    .iter()
                    .any(|protected| protected.overlaps(*range))
                {
                    return Err(ExecutionBasisDenial::Invalid);
                }
                let length =
                    usize::try_from(range.length()).map_err(|_| ExecutionBasisDenial::Invalid)?;
                let mut bytes = vec![0; length];
                let mut prior_end = range.offset();
                for (&offset, frame) in ranges.range(range.offset()..range.end()) {
                    let end = offset
                        .checked_add(frame.len() as u64)
                        .ok_or(ExecutionBasisDenial::Invalid)?;
                    if offset < prior_end || end > range.end() {
                        return Err(ExecutionBasisDenial::Invalid);
                    }
                    let start = (offset - range.offset()) as usize;
                    bytes[start..start + frame.len()].copy_from_slice(frame);
                    prior_end = end;
                }
                push(&mut commands, artifact, range.offset(), bytes.into(), &[])?;
            }
            if ranges.iter().any(|(&offset, bytes)| {
                !destination_ranges.iter().any(|range| {
                    range.arena().get() == arena
                        && offset >= range.offset()
                        && offset
                            .checked_add(bytes.len() as u64)
                            .is_some_and(|end| end <= range.end())
                })
            }) {
                return Err(ExecutionBasisDenial::Invalid);
            }
            continue;
        }
        let mut bytes = Vec::new();
        for (offset, range) in ranges {
            if offset != bytes.len() as u64 || range.is_empty() {
                return Err(ExecutionBasisDenial::Invalid);
            }
            bytes.extend_from_slice(&range);
        }
        push(&mut commands, artifact, 0, bytes.into(), source_artifacts)?;
    }
    commands.sort_by_key(|command| (command.artifact, command.offset));
    for (ordinal, command) in commands.iter_mut().enumerate() {
        command.ordinal = ordinal as u64;
    }
    Ok(commands.into_boxed_slice())
}

fn push(
    commands: &mut Vec<RecoveryStagingCommandPlan>,
    artifact: RecordArtifactFile,
    offset: u64,
    bytes: Box<[u8]>,
    source_artifacts: &[RecordArtifactFile],
) -> Result<(), ExecutionBasisDenial> {
    if bytes.is_empty()
        || source_artifacts.contains(&artifact)
        || commands.iter().any(|command| {
            command.artifact == artifact
                && offset < command.offset.saturating_add(command.byte_count())
                && command.offset < offset.saturating_add(bytes.len() as u64)
        })
    {
        return Err(ExecutionBasisDenial::Invalid);
    }
    commands.push(RecoveryStagingCommandPlan {
        ordinal: 0,
        artifact,
        offset,
        payload_digest: Sha256::digest(&bytes).into(),
        bytes,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ExecutionBasisDenial;

    use worth_store_physical_format::{
        PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame, PhysicalGeneration,
        PhysicalGenerationAuthority, PhysicalPageId, PhysicalSegmentId, RecordArtifactFile,
        RecordFrameCoordinate,
    };

    #[test]
    fn nonzero_page_range_is_carried_with_its_complete_artifact_prefix() {
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let segment = PhysicalSegmentId::from_raw(7).unwrap();
        let generation = PhysicalGeneration::from_raw(3).unwrap();
        let artifact = RecordArtifactFile::Segment {
            segment: 7,
            generation: 3,
        };
        let frames = [
            frame(&authority, segment, generation, artifact, 1, 0, b"base"),
            frame(&authority, segment, generation, artifact, 2, 4, b"redo"),
        ];

        let commands = super::exact_commands(frames, [], &[], &[], &[]).unwrap();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].artifact(), artifact);
        assert_eq!(commands[0].bytes(), b"baseredo");
        assert_eq!(commands[0].byte_count(), 8);
    }

    #[test]
    fn source_artifact_cannot_be_reused_as_a_recovery_staging_command() {
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let segment = PhysicalSegmentId::from_raw(7).unwrap();
        let generation = PhysicalGeneration::from_raw(3).unwrap();
        let artifact = RecordArtifactFile::Segment {
            segment: 7,
            generation: 3,
        };
        let frames = [frame(
            &authority, segment, generation, artifact, 1, 0, b"base",
        )];

        assert_eq!(
            super::exact_commands(frames, [], &[artifact], &[], &[]),
            Err(ExecutionBasisDenial::Invalid)
        );
    }

    fn frame(
        authority: &PhysicalGenerationAuthority,
        segment: PhysicalSegmentId,
        generation: PhysicalGeneration,
        artifact: RecordArtifactFile,
        page: u64,
        offset: u64,
        bytes: &[u8],
    ) -> PersistedPhysicalRecoveryFrame {
        let page = authority
            .page_cell(segment, PhysicalPageId::from_raw(page).unwrap())
            .with_page_generation(generation);
        let coordinate = RecordFrameCoordinate::new(artifact, offset, bytes.len() as u32).unwrap();
        PersistedPhysicalRecoveryFrame::new(
            PersistedPhysicalDataFrameSubject::InlinePage(page),
            coordinate,
            bytes,
        )
        .unwrap()
    }
}
