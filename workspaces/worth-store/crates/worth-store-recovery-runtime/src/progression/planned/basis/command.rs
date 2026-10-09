use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    ExtentArenaRange, PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest,
    RecordArtifactFile,
};

use super::{ExecutionBasisDenial, PlanningResidentAllowance, RecoveryStagingCommandPlan};

#[path = "command/spans.rs"]
mod spans;
use spans::{analyze, group_end, Span};

pub(super) fn exact_commands<'a>(
    frames: impl ExactSizeIterator<Item = &'a PersistedPhysicalRecoveryFrame>,
    manifests: impl ExactSizeIterator<Item = &'a PersistedPhysicalRecoveryManifest>,
    source_artifacts: &[RecordArtifactFile],
    protected_ranges: &[ExtentArenaRange],
    destination_ranges: &[ExtentArenaRange],
    allowance: &mut PlanningResidentAllowance,
) -> Result<Box<[RecoveryStagingCommandPlan]>, ExecutionBasisDenial> {
    let count = frames
        .len()
        .checked_add(manifests.len())
        .ok_or(ExecutionBasisDenial::RecoveryMemoryBytes { observed: u64::MAX })?;
    let mut spans = allowance.reserve::<Span<'a>>(count)?;
    for frame in frames {
        if spans.len() == count {
            return Err(ExecutionBasisDenial::Invalid);
        }
        let coordinate = frame.coordinate();
        spans.push(Span {
            artifact: coordinate.artifact(),
            offset: coordinate.offset(),
            bytes: frame.bytes(),
        });
    }
    for manifest in manifests {
        if spans.len() == count {
            return Err(ExecutionBasisDenial::Invalid);
        }
        let coordinate = manifest.coordinate();
        spans.push(Span {
            artifact: coordinate.artifact(),
            offset: coordinate.offset(),
            bytes: manifest.bytes(),
        });
    }
    if spans.len() != count {
        return Err(ExecutionBasisDenial::Invalid);
    }
    spans.sort_unstable_by_key(|span| (span.artifact, span.offset));
    let shape = analyze(
        &spans,
        source_artifacts,
        protected_ranges,
        destination_ranges,
    )?;
    let mut commands = allowance.reserve::<RecoveryStagingCommandPlan>(shape.count)?;
    let payload_window = shape
        .payload_bytes
        .checked_add(shape.largest_payload)
        .ok_or(ExecutionBasisDenial::RecoveryMemoryBytes { observed: u64::MAX })?;
    allowance.transient(payload_window)?;
    let mut start = 0;
    while start < spans.len() {
        let end = group_end(&spans, start);
        let artifact = spans[start].artifact;
        if let RecordArtifactFile::ExtentArena { arena } = artifact {
            for range in destination_ranges
                .iter()
                .filter(|range| range.arena().get() == arena)
            {
                let length =
                    usize::try_from(range.length()).map_err(|_| ExecutionBasisDenial::Invalid)?;
                let mut bytes = allowance.reserve::<u8>(length)?;
                bytes.resize(length, 0);
                for span in &spans[start..end] {
                    if span.offset < range.offset() || span.offset >= range.end() {
                        continue;
                    }
                    let offset = usize::try_from(span.offset - range.offset())
                        .map_err(|_| ExecutionBasisDenial::Invalid)?;
                    bytes[offset..offset + span.bytes.len()].copy_from_slice(span.bytes);
                }
                push(
                    &mut commands,
                    artifact,
                    range.offset(),
                    allowance.into_box(bytes)?,
                    shape.count,
                )?;
            }
        } else {
            let length = spans[start..end]
                .iter()
                .try_fold(0_usize, |sum, span| sum.checked_add(span.bytes.len()))
                .ok_or(ExecutionBasisDenial::Invalid)?;
            let mut bytes = allowance.reserve::<u8>(length)?;
            for span in &spans[start..end] {
                bytes.extend_from_slice(span.bytes);
            }
            push(
                &mut commands,
                artifact,
                0,
                allowance.into_box(bytes)?,
                shape.count,
            )?;
        }
        start = end;
    }
    if commands.len() != shape.count {
        return Err(ExecutionBasisDenial::Invalid);
    }
    commands.sort_unstable_by_key(|command| (command.artifact, command.offset));
    for (ordinal, command) in commands.iter_mut().enumerate() {
        command.ordinal = ordinal as u64;
    }
    let span_heap = PlanningResidentAllowance::vector_bytes(&spans)?;
    drop(spans);
    allowance.release(span_heap);
    allowance.into_box(commands).map_err(Into::into)
}

fn push(
    commands: &mut Vec<RecoveryStagingCommandPlan>,
    artifact: RecordArtifactFile,
    offset: u64,
    bytes: Box<[u8]>,
    expected_count: usize,
) -> Result<(), ExecutionBasisDenial> {
    if commands.len() >= expected_count {
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
    use super::{ExecutionBasisDenial, PlanningResidentAllowance};

    use worth_store_physical_format::{
        ExtentArenaId, ExtentArenaRange, PersistedPhysicalDataFrameSubject,
        PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest, PhysicalGeneration,
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

        let mut allowance = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
        let commands = super::exact_commands(
            frames.iter(),
            std::iter::empty(),
            &[],
            &[],
            &[],
            &mut allowance,
        )
        .unwrap();

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

        let mut allowance = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
        assert_eq!(
            super::exact_commands(
                frames.iter(),
                std::iter::empty(),
                &[artifact],
                &[],
                &[],
                &mut allowance,
            ),
            Err(ExecutionBasisDenial::Invalid)
        );
    }

    #[test]
    fn coordinate_gap_and_duplicate_are_rejected_before_payload_construction() {
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let segment = PhysicalSegmentId::from_raw(7).unwrap();
        let generation = PhysicalGeneration::from_raw(3).unwrap();
        let artifact = RecordArtifactFile::Segment {
            segment: 7,
            generation: 3,
        };
        let gap = [frame(
            &authority, segment, generation, artifact, 1, 1, b"gap",
        )];
        let mut allowance = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
        assert_eq!(
            super::exact_commands(
                gap.iter(),
                std::iter::empty(),
                &[],
                &[],
                &[],
                &mut allowance,
            ),
            Err(ExecutionBasisDenial::Invalid)
        );

        let duplicate = [
            frame(&authority, segment, generation, artifact, 1, 0, b"one"),
            frame(&authority, segment, generation, artifact, 2, 0, b"two"),
        ];
        let mut allowance = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
        assert_eq!(
            super::exact_commands(
                duplicate.iter(),
                std::iter::empty(),
                &[],
                &[],
                &[],
                &mut allowance,
            ),
            Err(ExecutionBasisDenial::Invalid)
        );
    }

    #[test]
    fn combined_command_payload_window_denies_before_first_payload_buffer() {
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let segment = PhysicalSegmentId::from_raw(7).unwrap();
        let generation = PhysicalGeneration::from_raw(3).unwrap();
        let artifact = RecordArtifactFile::Segment {
            segment: 7,
            generation: 3,
        };
        let frames = [frame(
            &authority, segment, generation, artifact, 1, 0, b"payload",
        )];
        let required = std::mem::size_of::<super::spans::Span<'_>>()
            + std::mem::size_of::<super::RecoveryStagingCommandPlan>()
            + 2 * b"payload".len();
        let retained_baseline = 64_u64;
        let mut allowance = PlanningResidentAllowance::new(
            retained_baseline,
            retained_baseline + required as u64 - 1,
        )
        .unwrap();
        assert!(matches!(
            super::exact_commands(
                frames.iter(),
                std::iter::empty(),
                &[],
                &[],
                &[],
                &mut allowance,
            ),
            Err(ExecutionBasisDenial::RecoveryMemoryBytes { .. })
        ));

        // The same live baseline also admits the real command when its full
        // header, payload, and conversion overlap fit in the shared window.
        let mut adequate =
            PlanningResidentAllowance::new(retained_baseline, retained_baseline + 4096).unwrap();
        let commands = super::exact_commands(
            frames.iter(),
            std::iter::empty(),
            &[],
            &[],
            &[],
            &mut adequate,
        )
        .unwrap();
        assert_eq!(commands[0].bytes(), b"payload");
    }

    #[test]
    fn arena_command_keeps_full_zero_filled_range_and_source_name_exception() {
        let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 8).unwrap();
        let manifests = [arena_manifest(2, b"ab")];
        let artifact = RecordArtifactFile::ExtentArena { arena: 2 };
        let mut allowance = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
        let commands = super::exact_commands(
            std::iter::empty(),
            manifests.iter(),
            &[artifact],
            &[],
            &[range],
            &mut allowance,
        )
        .unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].artifact(), artifact);
        assert_eq!(commands[0].bytes(), b"\0\0ab\0\0\0\0");
    }

    #[test]
    fn arena_protection_and_crossing_are_rejected_before_payload_construction() {
        let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 8).unwrap();
        let protected = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 4, 2).unwrap();
        let manifests = [arena_manifest(2, b"ab")];
        let mut allowance = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
        assert_eq!(
            super::exact_commands(
                std::iter::empty(),
                manifests.iter(),
                &[],
                &[protected],
                &[range],
                &mut allowance,
            ),
            Err(ExecutionBasisDenial::Invalid)
        );

        let crossing = [arena_manifest(7, b"ab")];
        let mut allowance = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
        assert_eq!(
            super::exact_commands(
                std::iter::empty(),
                crossing.iter(),
                &[],
                &[],
                &[range],
                &mut allowance,
            ),
            Err(ExecutionBasisDenial::Invalid)
        );
    }

    fn arena_manifest(offset: u64, bytes: &[u8]) -> PersistedPhysicalRecoveryManifest {
        let coordinate = RecordFrameCoordinate::new(
            RecordArtifactFile::ExtentArena { arena: 2 },
            offset,
            bytes.len() as u32,
        )
        .unwrap();
        PersistedPhysicalRecoveryManifest::new(coordinate, bytes).unwrap()
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
