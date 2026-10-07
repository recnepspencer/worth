//! Reproduces the exact WAL-named COW metadata effect. No replay-time head
//! allocation or routed-control inference is allowed here.

use worth_store_physical_format::{RecordArtifactFile, ReleaseCustodyHeadBlockReferenceV1};

use super::{CandidateBuild, CandidateBuildDenial};
use crate::progression::planned::basis::RecoveryBaseImagePlan;

pub(super) fn result_fields(
    base: &RecoveryBaseImagePlan,
) -> (Option<ReleaseCustodyHeadBlockReferenceV1>, u64) {
    match base.release_head_replay() {
        Some(replay) => (
            Some(replay.effect().result_root()),
            replay.effect().result_next_block(),
        ),
        None => (
            base.selected_root().release_custody_head_root(),
            base.selected_root().next_release_custody_head_block(),
        ),
    }
}

pub(super) fn append_exact_writes(
    base: &RecoveryBaseImagePlan,
    build: &mut CandidateBuild,
) -> Result<(), CandidateBuildDenial> {
    if let Some(replay) = base.release_head_replay() {
        for write in replay.effect().node_writes() {
            let reference = write.reference();
            let mut frame = build.allowance.reserve::<u8>(write.frame().len())?;
            frame.extend_from_slice(write.frame());
            let frame = build.allowance.into_box(frame)?;
            build.push_owned(
                RecordArtifactFile::ReleaseCustodyHeadBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                frame,
            )?;
        }
    }
    Ok(())
}
