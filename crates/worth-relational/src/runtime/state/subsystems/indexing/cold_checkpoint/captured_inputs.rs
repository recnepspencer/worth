//! Read the frozen original image without admitting or warming the live cache.
use super::*;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::derived_index_artifacts::{captured_decode, validate_checkpoint_generation};
use worth_execution::ExecutionRequest;
impl DeferredIndexImage {
    pub(in super::super) fn capture_generations_in_reconstruction(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<captured_decode::CapturedCheckpointGenerations, RelationalRecoveryStop> {
        let directory = self.directory();
        let key = directory.manifest.derived_indexes;
        let bytes = directory
            .canonical_read
            .read_member(&directory.source, request, key, 1 << 30)
            .map_err(|cause| {
                crate::durability::checkpoint_history::capture::owned_stop(
                    cause,
                    RelationalStorageSubject::Index(crate::indexes::data::DerivedIndexId(0)),
                )
            })?;
        let payload = member_frame::decode(bytes.bytes(), key, MemberKind::DerivedIndex).map_err(
            |cause| RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Index(crate::indexes::data::DerivedIndexId(0)),
                cause,
            },
        )?;
        let captured = captured_decode::decode(lane, payload, request)?;
        let mut seen = worth_execution::ExecutionOrderedMap::default();
        for generation in captured.generations.iter() {
            request
                .consult()
                .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
            if seen.get(&generation.generation_id).is_some() {
                return Err(crate::durability::checkpoints::region_directory::corrupt(
                    "duplicate original index generation",
                )
                .into());
            }
            seen.insert_cloned_native_for_request(&generation.generation_id, &(), *request)
                .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
            let definition = directory
                .manifest
                .index_definitions
                .iter()
                .find(|definition| definition.index_id == generation.index_id);
            validate_checkpoint_generation(
                lane,
                generation,
                definition.map(AsRef::as_ref),
                &self.history,
                directory.manifest.index_generation_frontier,
            )?;
        }
        Ok(captured)
    }
}
