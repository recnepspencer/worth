//! Protected source-path observation for a pre-WAL release-head transition.
//! The Store root fence supplies source stability; each framed node is checked
//! against the exact root-reachable reference before it enters the plan.

use std::collections::BTreeSet;

use worth_store_physical_format::{
    DurablePhysicalRootManifest, RecordArtifactFile, ReleaseCustodyHeadBlockV1,
    ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadPathNodeV1, ReleaseCustodyHeadTransitionLimitsV1,
};

use super::super::{
    planning::inline_plan_failure::layout_failure,
    residency::{record_frame_reader::RecordFrameReader, PhysicalResidencyWorkPort},
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, RecordAppendDenial,
    RecordAppendError,
};

pub(in crate::physical_runtime::record_serving) fn read_release_head_path(
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    residency: PhysicalResidencyWorkPort,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    root: &DurablePhysicalRootManifest,
    key: ReleaseCustodyHeadKeyV1,
    limits: ReleaseCustodyHeadTransitionLimitsV1,
) -> Result<Vec<ReleaseCustodyHeadPathNodeV1>, RecordAppendError> {
    let reader = RecordFrameReader::serving(residency);
    let mut path = Vec::new();
    let mut seen = BTreeSet::new();
    let mut resident_bytes = 0_u64;
    let mut selected = root.release_custody_head_root();
    let byte_limit = access
        .transfer_limit()
        .get()
        .min(format.declaration().page_size().bytes());
    while let Some(reference) = selected {
        if path.len() >= usize::from(limits.max_path_nodes())
            || !seen.insert((reference.generation(), reference.block()))
            || reference.block() >= root.next_release_custody_head_block()
        {
            return Err(damaged());
        }
        let loaded = reader
            .load_bounded(
                allocation,
                RecordArtifactFile::ReleaseCustodyHeadBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                byte_limit,
            )
            .map_err(layout_failure)?;
        resident_bytes = resident_bytes
            .checked_add(loaded.len() as u64)
            .filter(|bytes| *bytes <= limits.max_total_frame_bytes())
            .ok_or_else(damaged)?;
        let frame = loaded.to_vec();
        let (block, observed_format) =
            ReleaseCustodyHeadBlockV1::decode(&frame, reference, root.tree_identity())
                .map_err(|_| damaged())?;
        if observed_format != format.declaration() {
            return Err(damaged());
        }
        selected = block.children().map(|children| {
            let index = children
                .partition_point(|child| child.first() <= key)
                .saturating_sub(1);
            children[index]
        });
        path.push(ReleaseCustodyHeadPathNodeV1::new(reference, frame));
    }
    Ok(path)
}

fn damaged() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}
