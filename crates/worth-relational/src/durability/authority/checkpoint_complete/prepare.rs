//! One invoked original-source fold produces the strong Complete result.
use super::{
    branches::BranchImages, canonical::CanonicalOutput, capture::FrozenCompleteMetadata,
    roots::CapturedRoots, schemas::CompleteSchemas, symbols::OriginalSymbolDictionary,
    CompleteRelationalNativeCheckpoint, Denial,
};
use crate::durability::data::*;
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;

pub(super) fn prepare(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    placement: RelationalCheckpointArtifactPolicy<'_>,
) -> Result<CompleteRelationalNativeCheckpoint, Denial> {
    crate::execution::check_request(execution).map_err(Denial::Request)?;
    if let (Some(actual), Some(selected)) = (
        runtime.durability.cold_source(),
        placement.publication_source(),
    ) && !std::sync::Arc::ptr_eq(&actual, &selected)
    {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    let mut frozen =
        FrozenCompleteMetadata::capture(runtime, execution, &placement)?.resolve(execution)?;
    let commits = frozen.original.read_canonical(execution, 1 << 30)?;
    let dictionary =
        OriginalSymbolDictionary::capture(&frozen.original.symbols, &commits, execution)?;
    let roots = CapturedRoots::capture(frozen.components.observations(), execution)?;
    let main = &frozen.branches.branch_cells[frozen.main_cell].branch_id;
    let branches = BranchImages::capture(&frozen.branches, execution)?;
    let schemas = CompleteSchemas::capture(&frozen.original, &commits, execution)?;
    super::covered_prefix::require(&frozen.original, &schemas, execution)?;
    let initial = crate::durability::source_initial_inputs::capture::capture_complete(
        main,
        &frozen.original.branches,
        None,
        execution,
    )
    .map_err(initial_stop)?;
    let original_image = super::original_image::emit(&frozen.original, &commits, execution)?;
    let lineage = super::metadata::capture_lineage(
        &commits,
        frozen
            .original
            .base
            .as_ref()
            .map(|base| &base.directory.manifest),
        execution,
    )?;
    let mut metadata = super::metadata::capture_metadata_members(
        &dictionary,
        &lineage,
        &frozen.indexes,
        frozen
            .original
            .base
            .as_ref()
            .map(|base| &base.directory.manifest),
        execution,
    )?;
    let canonical =
        CanonicalOutput::prepare(&commits, &frozen.original, &mut metadata.members, execution)?;
    let region_output = super::regions::capture(
        || frozen.components.observations(),
        main,
        &branches,
        &roots,
        frozen.original.base.as_ref(),
        &schemas,
        &dictionary,
        &mut metadata.members,
        execution,
    )?;
    let main_schema = frozen
        .components
        .observations()
        .find_map(|(descriptor, original_root)| {
            (descriptor.branch_id() == main && branches.is_current_component(descriptor))
                .then(|| original_root.retained_schema_authority())
        })
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    let (schema_images, carriers) = schemas.into_parts();
    let (mut draft, member_map) = super::draft::assemble(
        &frozen,
        &canonical,
        schema_images,
        lineage.basis.clone(),
        metadata,
        execution,
    )?;
    super::contracts::install(&main_schema, &mut draft, execution)?;
    let region_custody = region_output.install_into(&mut draft);
    super::destination_members::require(&placement, &member_map, execution)?;
    let canonical_publication = canonical.publication(&draft, execution)?;
    let inline_bytes = canonical.inline_bytes;
    let coverage = draft.covered_position;
    let through = draft.coverage.up_to_branch_transition;
    let manifest = draft.seal_for_request(execution)?;
    let members = CapturedCheckpointMembers::capture(
        execution,
        member_map.len(),
        member_map.values().cloned(),
    )?;
    drop(member_map);
    let (bytes, sections) = crate::durability::log::native_checkpoint_codec::encode_complete(
        &manifest,
        &members,
        execution,
        placement.is_embedded(),
        inline_bytes,
    )?;
    let publication = if placement.is_embedded() {
        None
    } else {
        placement
            .publication_source()
            .map(|source| {
                RelationalHistoryMemberDirectory::empty_for_request(execution).map(|history| {
                    runtime.durability.checkpoint_selection().prepare(
                        manifest.clone(),
                        source,
                        history,
                    )
                })
            })
            .transpose()
            .map_err(stop)?
    };
    // Borrowed lookup scratch ends before the verified originals move into
    // their thin owning phase. The image consumes the same protected prefix.
    drop(lineage);
    drop(branches);
    drop(dictionary);
    drop(canonical);
    let original_read = frozen.original.take_original_read();
    let holds = frozen.components.into_holds();
    let mut checkpoint =
        RelationalNativeCheckpoint::from_captured_bytes(bytes, sections, coverage, through)
            .with_complete_component_holds(holds, region_custody.cells, original_read, execution)?
            .with_canonical_publication(Some(canonical_publication))
            .with_captured_members(members);
    if publication.is_some() {
        checkpoint = checkpoint.with_complete_region_publications(
            region_custody.publications.len(),
            region_custody.publications,
            execution,
        )?;
    }
    let initial_artifact = initial
        .artifact
        .as_ref()
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    checkpoint = checkpoint
        .with_manifest_publication(
            publication
                .map(|publication| {
                    publication
                        .with_initial_inputs(initial_artifact.clone())
                        .with_capture_base(execution)
                })
                .transpose()?,
        )
        .bind_original_publication(
            runtime.durability.medium(),
            Some(initial_artifact.clone()),
            execution,
        )?;
    let source_artifact =
        crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact::emit(
            &checkpoint,
            initial
                .artifact
                .as_ref()
                .ok_or(Denial::OriginalAuthorityUnavailable)?,
            &original_image,
            execution,
        )
        .map_err(initial_stop)?;
    let verified = super::verified_originals::VerifiedCheckpointOriginals::from_verified(
        super::verified_originals::VerifiedOriginalCustody {
            checkpoint,
            source_artifact,
            manifest,
            commits,
            roots,
            original: frozen.original,
            carriers,
        },
        execution,
    )?;
    let admission_image = super::admission_projection::capture(&verified, execution)?;
    let (checkpoint, source_artifact) = verified.consume(|originals| originals.into_parts());
    Ok(CompleteRelationalNativeCheckpoint {
        checkpoint,
        source_artifact,
        admission_image,
        initial_predecessor: initial.predecessor,
    })
}
pub(super) fn initial_stop(
    cause: crate::durability::source_initial_inputs::RelationalInitialInputsRefusal,
) -> Denial {
    use crate::durability::source_initial_inputs::RelationalInitialInputsRefusal as Refusal;
    match cause {
        Refusal::Request(cause) => Denial::Request(cause),
        Refusal::Native(cause) => stop(cause),
        cause => Denial::Durability(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!("Complete original input image refused: {cause:?}"),
        )),
    }
}
fn stop(cause: RelationalRecoveryStop) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(cause)
}
