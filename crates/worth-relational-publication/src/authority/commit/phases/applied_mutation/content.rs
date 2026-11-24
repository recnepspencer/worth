//! Admit content for the same real mutation image that carries its field references.
use super::*;
use worth_relational_contracts::internal_port as contracts;

pub fn prepare<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    before: &worth_relational_branch::internal_port::branch::root::RelationalBranchRoot,
    schema: &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
    attachments: &worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
    state: &WorkingState,
    changed: &[contracts::transactions::data::primitives::RecordRef],
    version: contracts::identity::data::primitives::VersionId,
    source: Option<&worth_relational_storage::internal_port::content::directory::ContentDirectory>,
) -> Result<
    worth_relational_branch::internal_port::content::publication::ContentPublication,
    MutationApplyDenial,
> {
    let inputs = inputs.into();
    let mut publication = contracts::execution::content_stage::run_commit_stage(
        execution,
        inputs.work.as_ref(),
        || {
            worth_relational_branch::internal_port::content::publication::prepare_publication(
                execution,
                before,
                schema.registry(),
                attachments,
                state,
                changed,
                version,
                source,
            )
        },
    )
    .map_err(worth_relational_branch::internal_port::content::publication::commit_denial)?;
    publication
        .bind_schema(schema)
        .map_err(worth_relational_branch::internal_port::content::publication::commit_denial)?;
    Ok(publication)
}
