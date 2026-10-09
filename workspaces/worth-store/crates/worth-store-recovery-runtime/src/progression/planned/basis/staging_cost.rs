use super::frame_identity::frame_identity;
use super::*;

pub(super) fn preflight_staging_cost(
    pending: &[&worth_store_recovery_physics::PhysicalRedoProjection],
    maximum_staging_bytes: u64,
    maximum_dirty_frames: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<u64, ExecutionBasisDenial> {
    let root_bytes = pending
        .iter()
        .try_fold(0_u64, |total, projection| {
            total.checked_add(
                projection
                    .materialization()
                    .root_state()
                    .root_publication_allocation_bytes(),
            )
        })
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let mut admission =
        StagingCostAdmission::new(root_bytes, maximum_staging_bytes, maximum_dirty_frames)?;
    let counts = pending
        .iter()
        .try_fold(
            (0_usize, 0_usize, 0_usize),
            |(frames, manifests, ranges), projection| {
                let image = projection.materialization();
                Some((
                    frames.checked_add(image.frames()?.len())?,
                    manifests.checked_add(image.manifests().len())?,
                    ranges.checked_add(image.placements().len())?,
                ))
            },
        )
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let mut frames = allowance.reserve(counts.0)?;
    let mut manifests = allowance.reserve(counts.1)?;
    let mut arena_ranges = allowance.reserve(counts.2)?;
    for projection in pending {
        for placement in projection.materialization().placements() {
            if let CurrentPhysicalRecordPlacement::Extent(extent) = placement {
                arena_ranges.push(extent.arena_range());
            }
        }
        for frame in projection
            .materialization()
            .frames()
            .ok_or(ExecutionBasisDenial::Invalid)?
        {
            frames.push((frame_identity(frame.subject()), frame));
        }
        manifests.extend(projection.materialization().manifests());
    }
    arena_ranges.sort_unstable();
    arena_ranges.dedup();
    for range in &arena_ranges {
        admission.admit_bytes(range.length())?;
    }
    canonical_frames(&mut frames)?;
    for &(identity, frame) in &frames {
        let bytes = if matches!(
            frame.coordinate().artifact(),
            RecordArtifactFile::ExtentArena { .. }
        ) {
            0
        } else {
            frame.bytes().len() as u64
        };
        admission.admit(identity, bytes)?;
    }
    canonical_manifests(&mut manifests)?;
    for manifest in &manifests {
        if !matches!(manifest.artifact(), RecordArtifactFile::ExtentArena { .. }) {
            admission.admit_bytes(manifest.bytes().len() as u64)?;
        }
    }
    let scratch = PlanningResidentAllowance::vector_bytes(&frames)?
        .checked_add(PlanningResidentAllowance::vector_bytes(&manifests)?)
        .and_then(|bytes| {
            bytes.checked_add(PlanningResidentAllowance::vector_bytes(&arena_ranges).ok()?)
        })
        .ok_or(ExecutionBasisDenial::Invalid)?;
    drop((frames, manifests, arena_ranges));
    allowance.release(scratch);
    Ok(admission.allocated_bytes)
}

fn canonical_frames(
    frames: &mut Vec<(
        PhysicalRedoTargetIdentity,
        &worth_store_physical_format::PersistedPhysicalRecoveryFrame,
    )>,
) -> Result<(), ExecutionBasisDenial> {
    frames.sort_unstable_by_key(|entry| entry.0);
    if frames
        .windows(2)
        .any(|pair| pair[0].0 == pair[1].0 && pair[0].1 != pair[1].1)
    {
        return Err(ExecutionBasisDenial::Invalid);
    }
    frames.dedup_by_key(|entry| entry.0);
    Ok(())
}

fn canonical_manifests(
    manifests: &mut Vec<&worth_store_physical_format::PersistedPhysicalRecoveryManifest>,
) -> Result<(), ExecutionBasisDenial> {
    manifests.sort_unstable_by_key(|manifest| manifest.coordinate());
    if manifests
        .windows(2)
        .any(|pair| pair[0].coordinate() == pair[1].coordinate())
    {
        return Err(ExecutionBasisDenial::Invalid);
    }
    Ok(())
}

struct StagingCostAdmission {
    maximum_bytes: u64,
    maximum_frames: u64,
    allocated_bytes: u64,
    last_target: Option<PhysicalRedoTargetIdentity>,
    admitted_frames: u64,
}

impl StagingCostAdmission {
    fn new(
        root_bytes: u64,
        maximum_bytes: u64,
        maximum_frames: u64,
    ) -> Result<Self, ExecutionBasisDenial> {
        if root_bytes > maximum_bytes {
            return Err(ExecutionBasisDenial::StagingBytes {
                observed: root_bytes,
            });
        }
        Ok(Self {
            maximum_bytes,
            maximum_frames,
            allocated_bytes: root_bytes,
            last_target: None,
            admitted_frames: 0,
        })
    }

    fn admit(
        &mut self,
        identity: PhysicalRedoTargetIdentity,
        bytes: u64,
    ) -> Result<(), ExecutionBasisDenial> {
        if self.last_target == Some(identity) {
            return Ok(());
        }
        if self.last_target.is_some_and(|previous| previous > identity) {
            return Err(ExecutionBasisDenial::Invalid);
        }
        let observed_frames = self.admitted_frames + 1;
        if observed_frames > self.maximum_frames {
            return Err(ExecutionBasisDenial::DirtyFrames {
                observed: observed_frames,
            });
        }
        let observed_bytes = self
            .allocated_bytes
            .checked_add(bytes)
            .ok_or(ExecutionBasisDenial::Invalid)?;
        if observed_bytes > self.maximum_bytes {
            return Err(ExecutionBasisDenial::StagingBytes {
                observed: observed_bytes,
            });
        }
        self.last_target = Some(identity);
        self.admitted_frames = observed_frames;
        self.allocated_bytes = observed_bytes;
        Ok(())
    }

    fn admit_bytes(&mut self, bytes: u64) -> Result<(), ExecutionBasisDenial> {
        let observed = self
            .allocated_bytes
            .checked_add(bytes)
            .ok_or(ExecutionBasisDenial::Invalid)?;
        if observed > self.maximum_bytes {
            return Err(ExecutionBasisDenial::StagingBytes { observed });
        }
        self.allocated_bytes = observed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{
        PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
        PersistedPhysicalRecoveryManifest, PhysicalGeneration, PhysicalGenerationAuthority,
        PhysicalPageId, PhysicalSegmentId, RecordFrameCoordinate,
    };

    #[test]
    fn staging_cost_admits_exact_limits_without_double_charging_one_target() {
        let target = identity(1);
        let mut admission = StagingCostAdmission::new(100, 125, 1).unwrap();
        admission.admit(target, 25).unwrap();
        admission.admit(target, 25).unwrap();
        assert_eq!(admission.allocated_bytes, 125);
        assert_eq!(admission.admitted_frames, 1);
    }

    #[test]
    fn staging_cost_rejects_before_retaining_a_crossing_frame_or_byte_claim() {
        let mut frame_limited = StagingCostAdmission::new(100, 200, 1).unwrap();
        frame_limited.admit(identity(1), 25).unwrap();
        assert!(matches!(
            frame_limited.admit(identity(2), 25),
            Err(ExecutionBasisDenial::DirtyFrames { observed: 2 })
        ));
        assert_eq!(frame_limited.admitted_frames, 1);
        assert_eq!(frame_limited.allocated_bytes, 125);

        let mut byte_limited = StagingCostAdmission::new(100, 124, 2).unwrap();
        assert!(matches!(
            byte_limited.admit(identity(1), 25),
            Err(ExecutionBasisDenial::StagingBytes { observed: 125 })
        ));
        assert_eq!(byte_limited.admitted_frames, 0);
        assert_eq!(byte_limited.allocated_bytes, 100);
    }

    #[test]
    fn duplicate_frame_identity_accepts_equal_frame_but_denies_different_bytes() {
        let original = frame(b"same");
        let repeated = original.clone();
        let mut equal = vec![
            (frame_identity(original.subject()), &original),
            (frame_identity(repeated.subject()), &repeated),
        ];
        canonical_frames(&mut equal).expect("equal frame repetition is one charged target");
        assert_eq!(equal.len(), 1);

        let conflicting = frame(b"else");
        let mut different = vec![
            (frame_identity(original.subject()), &original),
            (frame_identity(conflicting.subject()), &conflicting),
        ];
        assert_eq!(
            canonical_frames(&mut different),
            Err(ExecutionBasisDenial::Invalid)
        );
    }

    #[test]
    fn duplicate_manifest_coordinate_denies_even_equal_payloads() {
        let coordinate =
            RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 2 }, 7, 4).unwrap();
        let original = PersistedPhysicalRecoveryManifest::new(coordinate, b"same").unwrap();
        let repeated = original.clone();
        let mut manifests = vec![&original, &repeated];
        assert_eq!(
            canonical_manifests(&mut manifests),
            Err(ExecutionBasisDenial::Invalid)
        );
    }

    fn frame(bytes: &[u8]) -> PersistedPhysicalRecoveryFrame {
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let segment = PhysicalSegmentId::from_raw(7).unwrap();
        let generation = PhysicalGeneration::from_raw(3).unwrap();
        let page = authority
            .page_cell(segment, PhysicalPageId::from_raw(1).unwrap())
            .with_page_generation(generation);
        let coordinate = RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 7,
                generation: 3,
            },
            0,
            bytes.len() as u32,
        )
        .unwrap();
        PersistedPhysicalRecoveryFrame::new(
            PersistedPhysicalDataFrameSubject::InlinePage(page),
            coordinate,
            bytes,
        )
        .unwrap()
    }

    fn identity(page: u64) -> PhysicalRedoTargetIdentity {
        PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page,
            generation: 2,
        }
    }
}
