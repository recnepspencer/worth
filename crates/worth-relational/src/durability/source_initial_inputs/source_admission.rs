//! Source readmission binds recorded chronology to the selected image and exact cut.
use super::{
    image::*, normalized::*, RelationalCompleteSourceArtifact,
    RelationalInitialInputsRefusal as Refusal, RelationalSourceInitialInputsArtifact,
};
use crate::durability::data::DurableCheckpoint;
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

#[derive(Debug)]
pub(crate) struct SourceOriginalInputs {
    pub(crate) world_selection: Option<Arc<crate::durability::RelationalWorldSourceCatalog>>,
    pub(crate) selected: Option<RelationalCompleteSourceArtifact>,
    pub(crate) current: Arc<InitialInputsImage>,
    pub(crate) covered: Option<Arc<InitialInputsImage>>,
    pub(crate) normalized: Option<Arc<NormalizedOriginalInputs>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(crate) fn readmit(
    current: ExecutionImmutableBytes,
    selected: Option<&RelationalCompleteSourceArtifact>,
    checkpoint: Option<&DurableCheckpoint>,
    world_selection: Option<Arc<crate::durability::RelationalWorldSourceCatalog>>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<SourceOriginalInputs, Refusal> {
    let bytes = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<SourceOriginalInputs>())
        .map_err(|_| Refusal::Malformed)?
        .0
        .pad_to_align()
        .size();
    let memory = request
        .reserve_retained_memory(bytes as u64)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let current =
        RelationalSourceInitialInputsArtifact::from_owned(current, request)?.readmit(request)?;
    let Some(selected) = selected else {
        if checkpoint.is_some() {
            return Err(Refusal::Malformed);
        }
        return Ok(SourceOriginalInputs {
            world_selection,
            current,
            selected: None,
            covered: None,
            normalized: None,
            _memory: memory,
        });
    };
    let checkpoint = checkpoint.ok_or(Refusal::Malformed)?;
    let covered = selected.initial_artifact()?.readmit(request)?;
    if covered.main != current.main
        || current.entries.get(..covered.entries.len()) != Some(&covered.entries[..])
    {
        return Err(Refusal::Malformed);
    }
    let prior = covered.artifact.as_ref().ok_or(Refusal::Malformed)?;
    let next = current.artifact.as_ref().ok_or(Refusal::Malformed)?;
    if prior.as_bytes() != next.as_bytes() {
        if covered.revision != 1
            || current.revision != 2
            || current.predecessor != Some(prior.payload_digest())
            || covered.entries.len() != 1
            || selected.covered().0 != 0
        {
            return Err(Refusal::Malformed);
        }
        let Some(InitialInputEntry::Installation {
            follows_transition, ..
        }) = current.entries.get(1)
        else {
            return Err(Refusal::Malformed);
        };
        if *follows_transition < selected.covered_transitions() {
            return Err(Refusal::Malformed);
        }
    }
    let normalized = super::normalized_admission::readmit(
        selected.normalized_bytes(),
        selected.covered().0,
        selected.covered_transitions().0,
        request,
    )?;
    let directory = checkpoint
        .directory
        .as_ref()
        .or(checkpoint.canonical_directory.as_ref())
        .ok_or(Refusal::Malformed)?;
    if directory.manifest.covered_position != selected.covered()
        || checkpoint.coverage.up_to_branch_transition != selected.covered_transitions()
        || normalized.canonical.len() != directory.manifest.commits.len()
    {
        return Err(Refusal::Malformed);
    }
    let commitments = super::input_commitments::OriginalInputCommitments::capture(
        &checkpoint.branch_root_schema_images,
        request,
    )?;
    for (record, index) in normalized.canonical.iter().zip(&directory.manifest.commits) {
        commitments.require_record(record, index, request)?;
    }
    for transition in &normalized.transitions {
        if !commitments.has_schema(transition.schema) {
            return Err(Refusal::Malformed);
        }
    }
    for image in &covered.schemas {
        if !checkpoint
            .branch_root_schema_images
            .iter()
            .any(|actual| actual == image)
        {
            return Err(Refusal::Malformed);
        }
    }
    super::chronology::validate(
        &covered,
        &normalized,
        &directory.manifest,
        &checkpoint.branch_root_schema_images,
        world_selection
            .as_deref()
            .and_then(|world| world.selected_checkpoint()),
        request,
    )?;
    Ok(SourceOriginalInputs {
        selected: Some(selected.clone()),
        world_selection,
        current,
        covered: Some(covered),
        normalized: Some(normalized),
        _memory: memory,
    })
}
