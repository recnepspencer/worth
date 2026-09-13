use super::frame_identity::frame_identity;
use super::*;

pub(super) fn preflight_staging_cost(
    pending: &[&worth_store_recovery_physics::PhysicalRedoProjection],
    maximum_staging_bytes: u64,
    maximum_dirty_frames: u64,
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
    let mut frames = BTreeMap::new();
    let mut manifests = BTreeSet::new();
    for projection in pending {
        for frame in projection.materialization().frames() {
            let identity = frame_identity(frame.subject());
            if let Some(retained) = frames.get(&identity) {
                if *retained != frame {
                    return Err(ExecutionBasisDenial::Invalid);
                }
                continue;
            }
            admission.admit(identity, frame.bytes().len() as u64)?;
            frames.insert(identity, frame);
        }
        for manifest in projection.materialization().manifests() {
            if !manifests.insert(manifest.artifact()) {
                return Err(ExecutionBasisDenial::Invalid);
            }
            admission.admit_bytes(manifest.bytes().len() as u64)?;
        }
    }
    Ok(admission.allocated_bytes)
}

struct StagingCostAdmission {
    maximum_bytes: u64,
    maximum_frames: u64,
    allocated_bytes: u64,
    admitted_targets: BTreeSet<PhysicalRedoTargetIdentity>,
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
            admitted_targets: BTreeSet::new(),
        })
    }

    fn admit(
        &mut self,
        identity: PhysicalRedoTargetIdentity,
        bytes: u64,
    ) -> Result<(), ExecutionBasisDenial> {
        if self.admitted_targets.contains(&identity) {
            return Ok(());
        }
        let observed_frames = self.admitted_targets.len() as u64 + 1;
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
        self.admitted_targets.insert(identity);
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

    #[test]
    fn staging_cost_admits_exact_limits_without_double_charging_one_target() {
        let target = identity(1);
        let mut admission = StagingCostAdmission::new(100, 125, 1).unwrap();
        admission.admit(target, 25).unwrap();
        admission.admit(target, 25).unwrap();
        assert_eq!(admission.allocated_bytes, 125);
        assert_eq!(admission.admitted_targets.len(), 1);
    }

    #[test]
    fn staging_cost_rejects_before_retaining_a_crossing_frame_or_byte_claim() {
        let mut frame_limited = StagingCostAdmission::new(100, 200, 1).unwrap();
        frame_limited.admit(identity(1), 25).unwrap();
        assert!(matches!(
            frame_limited.admit(identity(2), 25),
            Err(ExecutionBasisDenial::DirtyFrames { observed: 2 })
        ));
        assert_eq!(frame_limited.admitted_targets.len(), 1);
        assert_eq!(frame_limited.allocated_bytes, 125);

        let mut byte_limited = StagingCostAdmission::new(100, 124, 2).unwrap();
        assert!(matches!(
            byte_limited.admit(identity(1), 25),
            Err(ExecutionBasisDenial::StagingBytes { observed: 125 })
        ));
        assert!(byte_limited.admitted_targets.is_empty());
        assert_eq!(byte_limited.allocated_bytes, 100);
    }

    fn identity(page: u64) -> PhysicalRedoTargetIdentity {
        PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page,
            generation: 2,
        }
    }
}
