//! Logical content posture follows the actual Source interpretation.
use super::*;
use crate::storage::content::RelationalContentDenial as Denial;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_branch::internal_port::content::source::NativeContentSourceView;
#[derive(Debug)]
pub(super) struct ContentViews {
    before: NativeContentSourceView,
    after: NativeContentSourceView,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl NativeManagedSourceInterpretation {
    pub fn bind_content(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        directory: crate::storage::content::directory::ContentDirectory,
        schema: &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
    ) -> Result<(), Denial> {
        let memory = execution
            .reserve_retained_memory(size_of::<ContentViews>() as u64)
            .map_err(|cause| Denial::Execution(cause.into()))?;
        let root = &self.physical_basis.owner_inner().root;
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
    ) -> Result<
        &NativeContentSourceView,
        worth_relational_contracts::internal_port::derived_relation::read_denial::DerivedRelationReadDenial,
    >{
        let views = self.content.get().ok_or(
            worth_relational_contracts::internal_port::derived_relation::definition::DerivedRelationDenial::ForeignObservation,
        )?;
        Ok(if before { &views.before } else { &views.after })
    }
    pub fn restoration_content(
        &self,
    ) -> Option<&crate::storage::content::directory::ContentDirectory> {
        if self.has_restoration_origin() {
            self.materialization_source()
                .map(|source| source.basis().owner_inner().root.retained_content())
        } else {
            None
        }
    }
}
