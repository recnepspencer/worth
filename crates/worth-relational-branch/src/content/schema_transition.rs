//! Only a changed schema authority admits full content-directory reconstruction.
use super::{ContentDirectory, RelationalContentAttachments, RelationalContentDenial as Denial};

pub(super) fn prepare_directory(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    root: &crate::branch::root::RelationalBranchRoot,
    schema: &crate::schema::data::RelationalSchemaRegistry,
    attachments: &RelationalContentAttachments,
    state: &dyn crate::storage::overlay::CurrentPartitionAccess,
    changed: &[crate::transactions::data::primitives::RecordRef],
    version: crate::identity::data::VersionId,
    source: Option<&ContentDirectory>,
) -> Result<ContentDirectory, Denial> {
    let prior = root.retained_content();
    if std::ptr::eq(root.retained_schema_authority().registry(), schema) {
        return prior.prepare(
            execution,
            schema,
            attachments,
            state,
            changed,
            version,
            source,
        );
    }
    crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
        prior.prepare_schema_transition(
            lane,
            execution,
            schema,
            attachments,
            state,
            version,
            source,
        )
    })
}
