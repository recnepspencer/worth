//! Reuse exact admitted region DATA without demanding its covered bodies.
use super::*;
use crate::history::data::CommitId;
use crate::identity::data::PartitionId;

pub(super) struct CoveredRegions<'a> {
    rows: ExecutionOrderedMap<(CommitId, PartitionId, [u8; 32]), &'a RelationalCheckpointRegion>,
}
impl<'a> CoveredRegions<'a> {
    pub(super) fn capture(
        base: Option<&'a crate::durability::checkpoint_admission::FrozenSelectedCaptureBase>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut rows = ExecutionOrderedMap::default();
        if let Some(base) = base {
            for root in &base.directory.manifest.roots {
                for leaf in &root.regions {
                    crate::execution::check_request(request).map_err(request_stop)?;
                    // The current Complete producer emits flat Full histories.
                    // A layered history requires its retained DATA projection,
                    // rather than dropping any of its dependency members.
                    if !base
                        .directory
                        .history_members
                        .bases(leaf.history)
                        .is_empty()
                    {
                        continue;
                    }
                    rows.insert_cloned_native_for_request(
                        &(
                            root.commit_id,
                            leaf.partition_id,
                            root.schema_carrier_digest,
                        ),
                        &leaf,
                        *request,
                    )
                    .map_err(map_stop)?;
                }
            }
        }
        Ok(Self { rows })
    }
    pub(super) fn get(
        &self,
        commit: CommitId,
        partition: PartitionId,
        schema: [u8; 32],
    ) -> Option<&RelationalCheckpointRegion> {
        self.rows.get(&(commit, partition, schema)).copied()
    }
}
