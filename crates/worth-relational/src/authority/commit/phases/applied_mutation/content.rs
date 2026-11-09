//! Admit content for the same real mutation image that carries its field references.
use super::*;

pub(crate) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    before: &crate::branch::RelationalBranchRoot,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    attachments: &crate::content::RelationalContentAttachments,
    state: &WorkingState,
    changed: &[crate::transactions::data::RecordRef],
    version: crate::identity::data::VersionId,
    source: Option<&crate::content::ContentDirectory>,
) -> Result<crate::content::ContentPublication, MutationApplyDenial> {
    let mut publication =
        crate::content::run_commit_stage(execution, runtime.commit_work_budget.as_ref(), || {
            crate::content::prepare_publication(
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
        .map_err(crate::content::commit_denial)?;
    publication
        .bind_schema(schema)
        .map_err(crate::content::commit_denial)?;
    Ok(publication)
}
