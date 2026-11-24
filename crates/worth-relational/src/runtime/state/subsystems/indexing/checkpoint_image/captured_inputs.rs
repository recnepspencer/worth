//! Validate the authenticated checkpoint roster without publishing a generation.
use super::{admission::validate_checkpoint_generation, CapturedCheckpointGenerations};
use crate::durability::data::recovery_errors::DurabilityError;
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::indexes::data::DerivedIndexGenerationId;
use crate::indexes::retained_definition::RetainedDerivedIndexDefinition;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::storage::substrate::maintenance::ReconstructionLane;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(crate) fn validate(
    lane: &ReconstructionLane<'_>,
    history: &HistorySubsystem,
    definitions: &[RetainedDerivedIndexDefinition],
    frontier: u64,
    captured: &CapturedCheckpointGenerations,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let mut seen = worth_execution::ExecutionOrderedMap::default();
    for (position, generation) in captured.generations.iter().enumerate() {
        consult(request)?;
        if seen.get(&generation.generation_id).is_some() {
            return Err(
                DurabilityError::corrupt_checkpoint("duplicate original index generation").into(),
            );
        }
        seen.insert_cloned_native_for_request(&generation.generation_id, &position, *request)
            .map_err(crate::durability::data::recovery_stop::map_stop)?;
        let definition = definitions
            .iter()
            .find(|definition| definition.index_id == generation.index_id);
        validate_checkpoint_generation(
            lane,
            generation,
            definition.map(AsRef::as_ref),
            history,
            frontier,
        )?;
    }
    validate_carried_bases(history, definitions, captured, &seen, request)
}

/// Carried selections preserve admitted affinity and writer ordering.
fn validate_carried_bases(
    history: &HistorySubsystem,
    definitions: &[RetainedDerivedIndexDefinition],
    captured: &CapturedCheckpointGenerations,
    seen: &ExecutionOrderedMap<DerivedIndexGenerationId, usize>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let mut prior = None;
    for basis in &captured.bases {
        consult(request)?;
        let key = (&basis.branch, basis.version);
        if prior.is_some_and(|prior| prior >= key) {
            return Err(
                DurabilityError::corrupt_checkpoint("unordered original index basis").into(),
            );
        }
        prior = Some(key);
        history
            .validate_recovered_scoped_index_basis(
                &basis.branch,
                basis.commit,
                basis.version,
                basis.schema,
            )
            .map_err(|detail| DurabilityError::corrupt_checkpoint(&detail))?;
        let mut bound = None;
        for (index, generation) in &basis.bindings {
            consult(request)?;
            let generation = seen
                .get(generation)
                .and_then(|position| captured.generations.get(*position))
                .ok_or_else(|| {
                    DurabilityError::corrupt_checkpoint(
                        "original index basis generation is unavailable",
                    )
                })?;
            if generation.index_id != *index
                || generation.applicability.schema_version != basis.schema
                || generation.applicability.version_id > basis.version
                || bound.is_some_and(|bound| bound >= *index)
            {
                return Err(DurabilityError::corrupt_checkpoint(
                    "original index basis binding has inconsistent affinity",
                )
                .into());
            }
            bound = Some(*index);
        }
        let mut empty = None;
        for index in &basis.empty {
            consult(request)?;
            let defined = definitions
                .iter()
                .any(|definition| definition.index_id == *index);
            if !defined
                || empty.is_some_and(|empty| empty >= *index)
                || basis
                    .bindings
                    .binary_search_by_key(index, |(bound, _)| *bound)
                    .is_ok()
            {
                return Err(DurabilityError::corrupt_checkpoint(
                    "original index empty selection is inconsistent",
                )
                .into());
            }
            empty = Some(*index);
        }
    }
    Ok(())
}

fn consult(request: &ExecutionRequest<'_, '_>) -> Result<(), RelationalRecoveryStop> {
    request
        .consult()
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))
}
