//! Crate-boundary internal port for `WorthQueryApplicationInvariantProjectionAuthority`
//! operation/native projection execution. Never exported on
//! `worth-query-execution::facade` or Host.

use worth_query_installation::facade::ApplicationSchema;

use super::super::locked_reader::borrowed_snapshot::{CompletedProjectionRead, NativeReadAccess};
use super::super::operation_reader::{
    AppliedOperationProjection, BorrowedOperationProjection,
    WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryCompletedOperationInvariantProjection,
    WorthQueryInspectedOperationInvariantProjection,
};
use super::super::work::WorthQueryInvariantProjectionWorkBudget;
use super::super::{
    WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionDenial, WorthQueryInvariantProjectionWork,
    WorthQueryOperationProjectionDenial,
};
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use crate::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use crate::ports::{SourceQueryReaderPort, TypedAdmittedOperationPort};

pub trait ApplicationInvariantProjectionExecutionInternalPort<Schema: ApplicationSchema> {
    fn project_operation_on_product<Operation, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::basis::WorthQueryProductBranchLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryInspectedOperationInvariantProjection<Operation, Output>,
        WorthQueryInvariantProjectionDenial,
    >;

    fn project_admitted_operation_with_source_access<A, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &A,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<
                '_,
                '_,
                Schema,
                A::Operation,
            >,
            &WorthQueryInvariantEntityIdentity<Schema, A::Scope>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        source_query: Option<&dyn SourceQueryReaderPort>,
        source_context_use: Option<
            &std::cell::Cell<crate::domain_computation::primary_graph::handler::DecisionContextUse>,
        >,
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, A::Operation, Output>,
        WorthQueryOperationProjectionDenial,
    >
    where
        A: TypedAdmittedOperationPort<Schema>;

    #[allow(clippy::too_many_arguments)]
    fn project_applied_operation<A, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &A,
        before: &worth_relational::facade::runtime::RelationalPinnedRead,
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
        predecessor_access: Option<&dyn SourceQueryReaderPort>,
        source_context_use: Option<&std::cell::Cell<crate::domain_computation::primary_graph::handler::DecisionContextUse>>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<
                '_,
                '_,
                Schema,
                A::Operation,
            >,
            &WorthQueryInvariantEntityIdentity<Schema, A::Scope>,
        ) -> Output,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            Output,
            AppliedOperationProjection<Schema, A::Operation>,
            WorthQueryInvariantProjectionWork,
        ),
        WorthQueryOperationProjectionDenial,
    >
    where
        A: TypedAdmittedOperationPort<Schema>;

    fn project_existing_operation<A, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &A,
        lease: &WorthQueryApplicationSnapshotLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<
                '_,
                '_,
                Schema,
                A::Operation,
            >,
            &WorthQueryInvariantEntityIdentity<Schema, A::Scope>,
        ) -> Output,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            Output,
            BorrowedOperationProjection<Schema, A::Operation>,
            WorthQueryInvariantProjectionWork,
        ),
        WorthQueryOperationProjectionDenial,
    >
    where
        A: TypedAdmittedOperationPort<Schema>;

    fn project_borrowed_input<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        input: &worth_relational::facade::mvcc::RelationalPredecessorInput,
        maximum_work: Option<usize>,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
        projection: impl FnOnce(&mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>) -> T,
        retention: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<T>, WorthQueryInvariantProjectionDenial>;

    fn read_snapshot<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<Output>, WorthQueryInvariantProjectionDenial>;

    #[allow(clippy::too_many_arguments)]
    fn read_native_with_access<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        access: NativeReadAccess<'_>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<Output>, WorthQueryInvariantProjectionDenial>;
}

impl<Schema: ApplicationSchema> ApplicationInvariantProjectionExecutionInternalPort<Schema>
    for WorthQueryApplicationInvariantProjectionAuthority<Schema>
{
    fn project_operation_on_product<Operation, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::basis::WorthQueryProductBranchLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryInspectedOperationInvariantProjection<Operation, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        Self::project_operation_on_product(self, execution, product, projection, allocation_policy)
    }

    fn project_admitted_operation_with_source_access<A, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &A,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<
                '_,
                '_,
                Schema,
                A::Operation,
            >,
            &WorthQueryInvariantEntityIdentity<Schema, A::Scope>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        source_query: Option<&dyn SourceQueryReaderPort>,
        source_context_use: Option<
            &std::cell::Cell<crate::domain_computation::primary_graph::handler::DecisionContextUse>,
        >,
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, A::Operation, Output>,
        WorthQueryOperationProjectionDenial,
    >
    where
        A: TypedAdmittedOperationPort<Schema>,
    {
        Self::project_admitted_operation_with_source_access(
            self,
            execution,
            admission,
            projection,
            allocation_policy,
            source_query,
            source_context_use,
        )
    }

    fn project_applied_operation<A, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &A,
        before: &worth_relational::facade::runtime::RelationalPinnedRead,
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
        predecessor_access: Option<&dyn SourceQueryReaderPort>,
        source_context_use: Option<&std::cell::Cell<crate::domain_computation::primary_graph::handler::DecisionContextUse>>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<
                '_,
                '_,
                Schema,
                A::Operation,
            >,
            &WorthQueryInvariantEntityIdentity<Schema, A::Scope>,
        ) -> Output,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            Output,
            AppliedOperationProjection<Schema, A::Operation>,
            WorthQueryInvariantProjectionWork,
        ),
        WorthQueryOperationProjectionDenial,
    >
    where
        A: TypedAdmittedOperationPort<Schema>,
    {
        Self::project_applied_operation(
            self,
            execution,
            admission,
            before,
            candidate,
            own_outputs,
            predecessor_access,
            source_context_use,
            projection,
            allocation,
        )
    }

    fn project_existing_operation<A, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &A,
        lease: &WorthQueryApplicationSnapshotLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<
                '_,
                '_,
                Schema,
                A::Operation,
            >,
            &WorthQueryInvariantEntityIdentity<Schema, A::Scope>,
        ) -> Output,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            Output,
            BorrowedOperationProjection<Schema, A::Operation>,
            WorthQueryInvariantProjectionWork,
        ),
        WorthQueryOperationProjectionDenial,
    >
    where
        A: TypedAdmittedOperationPort<Schema>,
    {
        Self::project_existing_operation(self, execution, admission, lease, projection, allocation)
    }

    fn project_borrowed_input<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        input: &worth_relational::facade::mvcc::RelationalPredecessorInput,
        maximum_work: Option<usize>,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
        projection: impl FnOnce(&mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>) -> T,
        retention: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<T>, WorthQueryInvariantProjectionDenial> {
        Self::project_borrowed_input(
            self,
            execution,
            native,
            input,
            maximum_work,
            own_outputs,
            projection,
            retention,
        )
    }

    fn read_snapshot<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<Output>, WorthQueryInvariantProjectionDenial> {
        Self::read_snapshot(
            self,
            execution,
            work_budget,
            basis,
            snapshot,
            projection,
            retention_control,
        )
    }

    fn read_native_with_access<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        access: NativeReadAccess<'_>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<Output>, WorthQueryInvariantProjectionDenial> {
        Self::read_native_with_access(
            self,
            execution,
            work_budget,
            basis,
            snapshot,
            access,
            projection,
            retention_control,
        )
    }
}
