//! Join current catalog metadata to the exact selected original-input prefix.
use super::source::FrozenOriginalInputs;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;

pub(super) fn require(
    original: &FrozenOriginalInputs,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let Some(base) = &original.base else {
        return Ok(());
    };
    let prior = &base.directory.manifest;
    if prior.commits.len() > original.artifacts.len()
        || prior.commits.len() != base.state.normalized().canonical.len()
    {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    for ((index, record), artifact) in prior
        .commits
        .iter()
        .zip(&base.state.normalized().canonical)
        .zip(original.artifacts.iter())
    {
        worth_relational_contracts::internal_port::execution::request_custody::check_request(
            request,
        )
        .map_err(Denial::Request)?;
        let input = artifact
            .original_inputs()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        if index.commit.commit_id != artifact.commit_id()
            || index.commit.version_id != artifact.version_id()
            || &index.commit.branch_id != artifact.identity().authoring_branch()
            || index.commit.parents.as_slice() != artifact.parentage().as_slice()
            || &index.root != artifact.roots()
            || index.schema_version != artifact.schema_version()
            || artifact.envelope_residency().captured_position() != Some(index.position)
            || record.position != index.position
            || record.commit != index.commit.commit_id
            || record.payload_digest != index.payload_digest
            || record.mutation != input.mutation
            || input.prestate_schema.carrier_digest() != record.prestate_schema
            || input.result_schema.carrier_digest() != record.result_schema
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
    }
    Ok(())
}
