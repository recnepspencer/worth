//! Descriptive predecessor forms; neither carries fresh succession permission.
use super::super::{PublishedWorkflowInstanceRef, WorthQueryWorkflowInstanceDiscoveryIntent};
mod resolution;

/// Select an exact predecessor through the ordinary fresh-admission funnel.
/// Published references keep their native runtime/branch affinity. Original
/// start descriptors are located on the selected current mutation basis.
///
/// A bare record ID is neither original intent nor a native reference:
/// ```compile_fail,E0277
/// use worth_query_execution::publication_boundary::workflow_succession::WorkflowSuccessionTarget;
/// use worth_relational::facade::identity::EntityId;
/// fn naked_id<S>(entity: EntityId) -> WorkflowSuccessionTarget<S> {
///     entity.into()
/// }
/// ```
pub enum WorkflowSuccessionTarget<Schema> {
    Published(PublishedWorkflowInstanceRef),
    Original(WorthQueryWorkflowInstanceDiscoveryIntent<Schema>),
}
impl<Schema> From<PublishedWorkflowInstanceRef> for WorkflowSuccessionTarget<Schema> {
    fn from(reference: PublishedWorkflowInstanceRef) -> Self {
        Self::Published(reference)
    }
}
impl<Schema> From<WorthQueryWorkflowInstanceDiscoveryIntent<Schema>>
    for WorkflowSuccessionTarget<Schema>
{
    fn from(original: WorthQueryWorkflowInstanceDiscoveryIntent<Schema>) -> Self {
        Self::Original(original)
    }
}
