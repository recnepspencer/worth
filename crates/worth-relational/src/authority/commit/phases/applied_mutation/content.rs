//! Admit content for the same real mutation image that carries its field references.
use super::*;

pub(crate) fn prepare<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    before: &crate::branch::root::RelationalBranchRoot,
    schema: &crate::branch::root::schema::RelationalBranchRootSchemaAuthority,
    attachments: &crate::content::RelationalContentAttachments,
    state: &WorkingState,
    changed: &[crate::transactions::data::primitives::RecordRef],
    version: crate::identity::data::VersionId,
    source: Option<&crate::content::ContentDirectory>,
) -> Result<crate::content::publication::ContentPublication, MutationApplyDenial> {
    let inputs = inputs.into();
    let mut publication =
        crate::execution::content_stage::run_commit_stage(execution, inputs.work.as_ref(), || {
            crate::content::publication::prepare_publication(
                execution,
                before,
                schema.registry(),
                attachments,
                state,
                changed,
                version,
                source,
            )
        })
        .map_err(crate::content::publication::commit_denial)?;
    publication
        .bind_schema(schema)
        .map_err(crate::content::publication::commit_denial)?;
    Ok(publication)
}
