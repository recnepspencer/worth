//! Logical content posture follows the actual Source interpretation.
use super::*;
use crate::content::{NativeContentSourceView, RelationalContentDenial as Denial};
#[derive(Debug)]
pub(super) struct ContentViews {
    before: NativeContentSourceView,
    after: NativeContentSourceView,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl NativeManagedSourceInterpretation {
    pub(crate) fn bind_content(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        directory: crate::content::ContentDirectory,
        schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    ) -> Result<(), Denial> {
        let memory = execution
            .reserve_retained_memory(size_of::<ContentViews>() as u64)
            .map_err(|cause| Denial::Execution(cause.into()))?;
        let root = &self.physical_basis.inner.root;
        let before = NativeContentSourceView::new(
            root.retained_content().clone(),
            root.retained_schema_authority().as_ref().clone(),
        );
        let after = if let (Some(RelationalMaterializationTransactionMode::Suspend), Some(source)) =
            (self.posture, self.materialization_source())
        {
            if source.basis() != &self.physical_basis {
                return Err(Denial::EncodingInconsistency);
            }
            before.clone()
        } else {
            NativeContentSourceView::new(directory, schema.clone())
        };
        self.content
            .set(ContentViews {
                before,
                after,
                _memory: memory,
            })
            .map_err(|_| Denial::EncodingInconsistency)
    }
    pub(crate) fn content_view(
        &self,
        before: bool,
    ) -> Result<&NativeContentSourceView, crate::derived_relation::DerivedRelationReadDenial> {
        let views = self
            .content
            .get()
            .ok_or(crate::derived_relation::DerivedRelationDenial::ForeignObservation)?;
        Ok(if before { &views.before } else { &views.after })
    }
    pub(crate) fn restoration_content(&self) -> Option<&crate::content::ContentDirectory> {
        if self.has_restoration_origin() {
            self.materialization_source()
                .map(|source| source.basis().inner.root.retained_content())
        } else {
            None
        }
    }
}
