//! Read the frozen original image without admitting or warming the live cache.
use super::*;
use crate::durability::checkpoints::region_directory::corrupt;
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
        for (position, generation) in captured.generations.iter().enumerate() {
            consult(request)?;
            if seen.get(&generation.generation_id).is_some() {
                return Err(corrupt("duplicate original index generation").into());
            }
            seen.insert_cloned_native_for_request(&generation.generation_id, &position, *request)
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
        self.validate_carried_bases(&captured, &seen, request)?;
        Ok(captured)
    }
    /// The carried selections are original DATA too: the same affinity the
    /// admitting reader requires, in the writers' strictly ascending order.
    fn validate_carried_bases(
        &self,
        captured: &captured_decode::CapturedCheckpointGenerations,
        seen: &worth_execution::ExecutionOrderedMap<
            crate::indexes::data::DerivedIndexGenerationId,
            usize,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRecoveryStop> {
        let directory = self.directory();
        let mut prior = None;
        for basis in &captured.bases {
            consult(request)?;
            let key = (&basis.branch, basis.version);
            if prior.is_some_and(|prior| prior >= key) {
                return Err(corrupt("unordered original index basis").into());
            }
            prior = Some(key);
            self.history
                .validate_recovered_scoped_index_basis(
                    &basis.branch,
                    basis.commit,
                    basis.version,
                    basis.schema,
                )
                .map_err(|detail| corrupt(&detail))?;
            let mut bound = None;
            for (index, generation) in &basis.bindings {
                consult(request)?;
                let generation = seen
                    .get(generation)
                    .and_then(|position| captured.generations.get(*position))
                    .ok_or_else(|| corrupt("original index basis generation is unavailable"))?;
                if generation.index_id != *index
                    || generation.applicability.schema_version != basis.schema
                    || generation.applicability.version_id > basis.version
                    || bound.is_some_and(|bound| bound >= *index)
                {
                    return Err(
                        corrupt("original index basis binding has inconsistent affinity").into(),
                    );
                }
                bound = Some(*index);
            }
            let mut empty = None;
            for index in &basis.empty {
                consult(request)?;
                let defined = directory
                    .manifest
                    .index_definitions
                    .iter()
                    .any(|definition| definition.index_id == *index);
                if !defined
                    || empty.is_some_and(|empty| empty >= *index)
                    || basis
                        .bindings
                        .binary_search_by_key(index, |(bound, _)| *bound)
                        .is_ok()
                {
                    return Err(corrupt("original index empty selection is inconsistent").into());
                }
                empty = Some(*index);
            }
        }
        Ok(())
    }
}
fn consult(request: &ExecutionRequest<'_, '_>) -> Result<(), RelationalRecoveryStop> {
    request
        .consult()
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))
}
