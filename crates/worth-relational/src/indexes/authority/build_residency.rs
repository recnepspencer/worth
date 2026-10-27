//! Index construction admits restored generation state before allocating a successor.
use super::{DerivedIndexBuildOutcome, DerivedIndexBuildRequest, IndexAuthority};
impl IndexAuthority<'_> {
    pub(super) fn demand_build_index_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &DerivedIndexBuildRequest,
    ) -> Result<(), DerivedIndexBuildOutcome> {
        self.runtime
            .indexes
            .definition_read_binding()
            .demand_checkpoint_image(execution)
            .map_err(|failure| refused(request, failure.into()))
    }
}
pub(super) fn refused(
    request: &DerivedIndexBuildRequest,
    cause: crate::execution::RelationalExecutionDenialCause,
) -> DerivedIndexBuildOutcome {
    DerivedIndexBuildOutcome {
        source_commit_id: request.source_commit_id,
        generations: Vec::new(),
        failed_indexes: request.index_ids.clone(),
        basis_denial: None,
        execution_denial: Some(crate::indexes::data::DerivedIndexExecutionDenial {
            kind: crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause),
            partition_identity: if matches!(
                cause,
                crate::execution::RelationalExecutionDenialCause::StorageUnavailable(_)
            ) {
                None
            } else {
                request.index_ids.first().map(|_| 0)
            },
        }),
    }
}
