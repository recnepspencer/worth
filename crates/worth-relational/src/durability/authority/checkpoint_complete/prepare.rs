//! One invoked original-source fold produces the strong Complete result.
use super::{
    branches::BranchImages, canonical::CanonicalOutput, capture::FrozenCompleteMetadata,
    roots::FoldedRoots, schemas::CompleteSchemas, symbols::OriginalSymbolDictionary,
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
    let mut frozen = FrozenCompleteMetadata::capture(runtime, execution)?;
    frozen.original.protect_original_read(execution)?;
    let commits = frozen.original.read_canonical(execution, 1 << 30)?;
    let dictionary =
        OriginalSymbolDictionary::capture(&frozen.original.symbols, &commits, execution)?;
    let roots = FoldedRoots::derive(&commits, &dictionary, execution)?;
    let main = &frozen.branches.branch_cells[frozen.main_cell].branch_id;
    let branches =
        BranchImages::derive(main, &frozen.original.branches, &commits, &roots, execution)?;
    branches.require_current(
        &frozen.branches.branch_cells,
        &frozen.branches.branch_lifecycle,
        &frozen.branches.retired_branch_names,
        &roots,
    )?;
    let schemas = CompleteSchemas::capture(&frozen.original, &commits, execution)?;
    let lineage = super::metadata::capture_lineage(&commits, execution)?;
    let mut metadata = super::metadata::capture_metadata_members(
        &dictionary,
        &lineage,
        &frozen.indexes.generations,
        execution,
    )?;
    let canonical = CanonicalOutput::prepare(&commits, &mut metadata.members, execution)?;
    let region_output = super::regions::capture(
        || frozen.components.observations(),
        main,
        &branches,
        &roots,
        &schemas,
        &dictionary,
        &mut metadata.members,
        execution,
    )?;
    let main_schema = frozen
        .components
        .observations()
        .find_map(|(descriptor, original_root)| {
            (descriptor.branch_id() == main && branches.is_current_component(descriptor, &roots))
                .then(|| original_root.retained_schema_authority())
        })
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    let (mut draft, member_map) = super::draft::assemble(
        &frozen,
        &canonical,
        schemas.into_images(),
        lineage.basis.clone(),
        metadata,
        execution,
    )?;
    super::contracts::install(&main_schema, &mut draft, execution)?;
    let region_custody = region_output.install_into(&mut draft);
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
    // None of the all-prefix decode, lookup or root-build scratch is output
    // custody. Only actual N, component obligations and rebuilt cells cross.
    drop(lineage);
    drop(branches);
    drop(roots);
    drop(dictionary);
    drop(commits);
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
    checkpoint = checkpoint
        .with_manifest_publication(publication)
        .bind_publication(runtime.durability.medium(), execution)?;
    Ok(CompleteRelationalNativeCheckpoint { checkpoint })
}
fn stop(cause: RelationalRecoveryStop) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(cause)
}
