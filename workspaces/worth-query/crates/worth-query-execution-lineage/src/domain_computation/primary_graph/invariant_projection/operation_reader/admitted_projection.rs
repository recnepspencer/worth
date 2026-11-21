//! Original admitted projection, with an optional real source-query loan.
use super::admitted::SourceAccess;
use super::*;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    pub fn project_admitted_operation<A, Output>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, A::Operation, Output>,
        WorthQueryOperationProjectionDenial,
    >
    where
        A: crate::ports::TypedAdmittedOperationPort<Schema>,
    {
        let execution = phase
            .request_for_source(self.graph.relational_runtime_instance_id())
            .map_err(|_| {
                WorthQueryOperationProjectionDenial::from_invariant(
                    WorthQueryInvariantProjectionDenial::from_kind(
                        super::super::WorthQueryInvariantProjectionDenialKind::ForeignPhase,
                    ),
                    admission.operation(),
                )
            })?;
        self.project_admitted_operation_with_request(
            &execution,
            admission,
            projection,
            allocation_policy,
        )
    }

    pub fn project_admitted_operation_with_request<A, Output>(
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
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, A::Operation, Output>,
        WorthQueryOperationProjectionDenial,
    >
    where
        A: crate::ports::TypedAdmittedOperationPort<Schema>,
    {
        self.project_admitted_operation_with_source_access_port(
            execution,
            admission,
            projection,
            allocation_policy,
            None,
            None,
        )
    }

    pub fn project_admitted_operation_with_source_access<A, Output>(
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
        A: crate::ports::TypedAdmittedOperationPort<Schema>,
    {
        self.project_admitted_operation_with_source_access_port(
            execution,
            admission,
            projection,
            allocation_policy,
            source_query,
            source_context_use,
        )
    }

    pub fn project_admitted_operation_with_source_access_port<Operation, Scope, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &dyn AdmittedOperationPort,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        source_query: Option<&dyn SourceQueryReaderPort>,
        source_context_use: Option<
            &std::cell::Cell<crate::domain_computation::primary_graph::handler::DecisionContextUse>,
        >,
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output>,
        WorthQueryOperationProjectionDenial,
    > {
        admission.validate_projection_authority(
            self.runtime_authority,
            &self.binding_identity,
            &self.installed_operations,
        )?;
        let product = admission
            .mutation_product()
            .expect("admitted operation retains its selected mutation lease")
            .retained_clone();
        let allocation_control =
            crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(
                admission.publication_request(),
                allocation_policy,
            );
        let completed = self
            .project_with_work_limit(
                execution,
                admission.projection_work_budget(),
                product.relational_basis().clone(),
                |reader| {
                    super::admitted::project(
                        self,
                        reader,
                        admission,
                        &product,
                        projection,
                        SourceAccess {
                            source_query,
                            source_context_use,
                        },
                    )
                },
                Some(admission.publication_request()),
                allocation_control.policy(),
            )
            .map_err(|denial| {
                WorthQueryOperationProjectionDenial::from_invariant(denial, admission.operation())
            })?;
        Ok(WorthQueryCompletedOperationInvariantProjection {
            completed,
            admission_identity: admission.admission_identity(),
            product,
            _operation: PhantomData,
        })
    }
}
