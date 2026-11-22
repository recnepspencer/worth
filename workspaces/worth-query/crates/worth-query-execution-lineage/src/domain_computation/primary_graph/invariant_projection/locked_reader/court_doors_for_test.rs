//! Sealed test doors over this module's private members.
use super::*;

impl<'runtime, Schema> WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    #[doc(hidden)]
    pub fn authority_identity_for_test(&self) -> &u64 {
        &self.authority_identity
    }
}

impl<'runtime, Schema> WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    #[doc(hidden)]
    pub fn execution_for_test(
        &self,
    ) -> &&'runtime worth_execution::ExecutionRequest<'runtime, 'runtime> {
        &self.execution
    }
}

impl<'runtime, Schema> WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    #[doc(hidden)]
    pub fn snapshot_for_test(
        &self,
    ) -> &&'runtime worth_relational::facade::snapshots::SnapshotHandle {
        &self.snapshot
    }
}

impl<'runtime, Schema> WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    #[doc(hidden)]
    pub fn layout_for_test(
        &self,
    ) -> &&'runtime super::super::super::schema_layout::WorthQueryPrimaryGraphLayout {
        &self.layout
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn project_with_work_limit_for_test<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        maximum_work: Option<usize>,
        basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompletedInvariantProjection<Schema, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        Self::project_with_work_limit(
            self,
            execution,
            maximum_work,
            basis,
            projection,
            request,
            allocation_policy,
        )
    }
}

impl<'runtime, Schema> WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    #[doc(hidden)]
    pub fn work_for_test(&self) -> &WorthQueryInvariantProjectionWork {
        &self.work
    }
}

impl<'runtime, Schema> WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    #[doc(hidden)]
    pub fn native_for_test(&self) -> &NativeProjectionRead<'runtime> {
        &self.native
    }
}
