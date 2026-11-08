//! Funded original-frame export and admission projection share one actual cut.
use super::super::{
    branches::BranchImages, canonical::CanonicalOutput, roots::CapturedRoots,
    schemas::CompleteSchemas, symbols::OriginalSymbolDictionary, Denial,
};
use super::{verified::FrozenMemoryOriginals, CapturedNativeInterimAdmission};
use crate::durability::data::*;
use worth_execution::ExecutionRequest;

pub(super) fn capture(
    runtime: &crate::runtime::RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(RelationalNativeCheckpoint, CapturedNativeInterimAdmission), Denial> {
    let mut answer = None;
    let stage = request.run_native_with_checkpoint(|_| {
        answer = Some(capture_inner(runtime, request));
    });
    match (stage, answer) {
        (Ok(_), Some(answer)) => answer,
        (Ok(_), None) => unreachable!("the capture stage produced its answer"),
        (Err(cause), Some(Err(inner)))
            if crate::execution::RelationalExecutionDenialCause::from(cause)
                == crate::execution::RelationalExecutionDenialCause::NestedStopped =>
        {
            Err(inner)
        }
        (Err(cause), _) => Err(Denial::Request(cause.into())),
    }
}
fn capture_inner(
    runtime: &crate::runtime::RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(RelationalNativeCheckpoint, CapturedNativeInterimAdmission), Denial> {
    let frozen = FrozenMemoryOriginals::capture(runtime, request)?;
    let originals = frozen.read_frames(request)?;
    let frames = &originals.frames;
    let dictionary = OriginalSymbolDictionary::capture(&frozen.symbols, frames, request)?;
    let roots = CapturedRoots::capture(frozen.current.components.observations(), request)?;
    let main = &frozen.current.branches.branch_cells[frozen.current.main_cell].branch_id;
    let branches = BranchImages::capture(&frozen.current.branches, request)?;
    let schemas = CompleteSchemas::capture_interim(
        frames,
        frozen.current.components.observations(),
        request,
    )?;
    let indexes = frozen.current.indexes.resolve_for_request(request)?;
    let lineage = super::super::metadata::capture_lineage(frames, None, request)?;
    let mut metadata = super::super::metadata::capture_metadata_members(
        &dictionary,
        &lineage,
        &indexes,
        None,
        request,
    )?;
    let canonical = CanonicalOutput::prepare(
        frames,
        &frozen.artifacts,
        None,
        &mut metadata.members,
        request,
    )?;
    let main_schema = frozen
        .current
        .components
        .observations()
        .find_map(|(descriptor, root)| {
            (descriptor.branch_id() == main && branches.is_current_component(descriptor))
                .then(|| root.retained_schema_authority())
        })
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    let mirror_carrier = schemas
        .carrier_for(&main_schema)
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    let region_output = super::super::regions::capture_interim(
        || frozen.current.components.observations(),
        main,
        &branches,
        &roots,
        &schemas,
        super::super::regions::CheckpointMirror::Substrate {
            edition: &frozen.mirror,
            schema: &main_schema,
            carrier: mirror_carrier,
        },
        &dictionary,
        &mut metadata.members,
        request,
    )?;
    let (images, _) = schemas.into_parts();
    let (mut draft, member_map) = super::super::draft::assemble(
        &super::super::draft::CheckpointDraftMetadata {
            branches: &frozen.current.branches,
            identity: &frozen.current.identity,
            indexes: &indexes,
            symbol_frontier: frozen.current.symbol_frontier,
            through: frozen.transitions,
        },
        &canonical,
        images,
        lineage.basis.clone(),
        metadata,
        request,
    )?;
    super::super::contracts::install(&main_schema, &mut draft, request)?;
    let region_custody = region_output.install_into(&mut draft);
    let canonical_publication = canonical.publication(&draft, request)?;
    let inline_bytes = canonical.inline_bytes;
    let covered = draft.covered_position;
    let transitions = draft.coverage.up_to_branch_transition;
    let manifest = draft.seal_for_request(request)?;
    drop(branches);
    let members = CapturedCheckpointMembers::capture(
        request,
        member_map.len(),
        member_map.values().cloned(),
    )?;
    drop(member_map);
    let (bytes, sections) = crate::durability::log::native_checkpoint_codec::encode_complete(
        &manifest,
        &members,
        request,
        true,
        inline_bytes,
    )?;
    let digest = crate::durability::checkpoint_admission::capture_digest(bytes.as_ref(), request)?;
    drop(lineage);
    drop(dictionary);
    drop(canonical);
    let proof = FrozenMemoryOriginals::project(
        frozen.artifacts,
        frozen.current.branches,
        &frozen.owner,
        &manifest,
        originals,
        &roots,
        request,
    )?;
    let admission = CapturedNativeInterimAdmission::prepare(
        proof,
        frozen.coordinate,
        digest,
        covered.0,
        transitions.0,
        request,
    )?;
    let holds = frozen.current.components.into_holds();
    let checkpoint =
        RelationalNativeCheckpoint::from_captured_bytes(bytes, sections, covered, transitions)
            .with_complete_component_holds(holds, region_custody.cells, None, request)?
            .with_canonical_publication(Some(canonical_publication))
            .with_captured_members(members);
    Ok((checkpoint, admission))
}
