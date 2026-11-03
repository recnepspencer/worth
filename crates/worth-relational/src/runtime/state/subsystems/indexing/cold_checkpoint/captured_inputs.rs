//! Read the frozen original image without admitting or warming the live cache.
use super::*;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::derived_index_artifacts::{captured_decode, validate_checkpoint_generation};
use worth_execution::ExecutionRequest;
impl DeferredIndexImage {
    pub(in super::super) fn capture_generations(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<captured_decode::CapturedCheckpointGenerations, RelationalRecoveryStop> {
        let key = self.directory().manifest.derived_indexes;
        let bytes = self
            .directory()
            .canonical_read
            .read_member(&self.directory().source, request, key, 1 << 30)
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
        let captured = captured_decode::decode(payload, request)?;
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
            let definition = self
                .directory()
                .manifest
                .index_definitions
                .iter()
                .find(|definition| definition.index_id == generation.index_id);
            validate_checkpoint_generation(
                generation,
                definition.map(AsRef::as_ref),
                &self.history,
                self.directory().manifest.index_generation_frontier,
            )?;
        }
        Ok(captured)
    }
}
