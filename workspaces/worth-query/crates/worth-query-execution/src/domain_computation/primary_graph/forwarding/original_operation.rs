//! Thin forwarding shims for `original_operation` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_original_operation {
    use worth_query_execution_query::facade::domain_computation::primary_graph::query_obligations::*;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::original_operation::WorthQueryOriginalOperationDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::original_operation::WorthQueryOriginalOperationOwner;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn original_operation_requires_record(
            &self,
            __arg0: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        ) -> bool {
            self.inner.original_operation_requires_record(__arg0)
        }
        pub fn admit_original_operation_retention(
            &self,
            owner: Option<&WorthQueryOriginalOperationOwner>,
            access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        ) -> Result<(), WorthQueryOriginalOperationDenial> {
            self.inner.admit_original_operation_retention(owner, access)
        }
        pub fn admit_original_operation_owner(
            &self,
            owner: &WorthQueryOriginalOperationOwner,
            __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        ) -> Result<(), WorthQueryOriginalOperationDenial> {
            self.inner.admit_original_operation_owner(owner, __arg1)
        }
        pub fn progress_original_operation_in_advancement(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            owner: &WorthQueryOriginalOperationOwner,
            position: Option<QueryOriginalOperationPosition>,
        ) -> Result<(), WorthQueryOriginalOperationDenial> {
            self.inner
                .progress_original_operation_in_advancement(phase, access, owner, position)
        }
    }
}
