use crate::basis::{
    WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::ProductBranchIdentity;

pub use worth_query_execution_lineage::facade::primary_graph::product_operation::SelectedQueryOrigin;

pub(crate) type SelectedProductOperationInner<'runtime, Schema> =
    worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation<'runtime, Schema>;

/// Shared application entry bound to one retained World occurrence. Selecting
/// another occurrence requires a new admission; this context never resolves latest.
pub struct WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub(crate) application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(crate) inner: SelectedProductOperationInner<'runtime, Schema>,
}

/// Public application entry naming one product occurrence. It performs no
/// lookup until a concrete operation asks for an admitted selection.
pub struct WorthQueryProductEntry<'runtime, Schema> {
    pub(crate) application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(crate) branch: WorthQueryProductBranch,
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub(crate) fn from_inner(
        application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        inner: SelectedProductOperationInner<'runtime, Schema>,
    ) -> Self {
        Self { application, inner }
    }

    pub(crate) fn inner(&self) -> &SelectedProductOperationInner<'runtime, Schema> {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut SelectedProductOperationInner<'runtime, Schema> {
        &mut self.inner
    }

    pub(crate) fn into_inner(self) -> SelectedProductOperationInner<'runtime, Schema> {
        self.inner
    }

    /// Inspects the rostered program carried by this exact selected occurrence.
    pub fn inspect_selected_program(
        &self,
    ) -> Result<
        super::super::WorthQuerySelectedProgramInspection,
        super::super::WorthQuerySelectedProgramInspectionDenial,
    >
    where
        Schema: ApplicationSchema,
    {
        self.inner.inspect_selected_program()
    }

    pub fn selection_origin(&self) -> SelectedQueryOrigin {
        self.inner.selection_origin()
    }

    pub fn product(&self) -> &WorthQueryProductBranchLease {
        self.inner.product()
    }

    pub fn application(&self) -> &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.application
    }

    pub fn application_basis(
        &self,
    ) -> &super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease {
        self.inner.application_basis()
    }

    pub fn original_settled_producer(
        &self,
    ) -> Option<
        &std::sync::Arc<
            super::super::application_output_demand::OriginalSettledProducerObservation,
        >,
    > {
        self.inner.original_settled_producer()
    }

    pub fn application_basis_mut(
        &mut self,
    ) -> &mut super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease
    {
        self.inner.application_basis_mut()
    }

    pub fn into_parts(
        self,
    ) -> (
        &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryProductBranchLease,
        super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    ) {
        let (_, product, application_basis) = self.inner.into_parts();
        (self.application, product, application_basis)
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub fn with_original_settled_producer(
        self,
        producer: Option<
            std::sync::Arc<
                super::super::application_output_demand::OriginalSettledProducerObservation,
            >,
        >,
    ) -> Self {
        Self {
            application: self.application,
            inner: self.inner.with_original_settled_producer(producer),
        }
    }

    pub fn retain_selection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorthQueryProductBranchAdmissionDenial> {
        self.inner
            .retain_selection(execution)
            .map(|inner| Self::from_inner(self.application, inner))
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Installed host owner for lending a phase to public operations.
    pub fn product_runtime(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        self.inner.product_runtime()
    }

    pub fn select_product_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &ProductBranchIdentity,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        self.inner
            .select_product_branch(execution, identity)
            .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
    }

    /// Selects only a Query-issued occurrence token.
    ///
    /// ```compile_fail
    /// use worth_query_execution::facade::{
    ///     primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
    ///     runtime::ProductBranchIdentity,
    /// };
    /// use worth_query_installation::facade::ApplicationSchema;
    ///
    /// fn raw_world_identity_cannot_select<Schema: ApplicationSchema>(
    ///     app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ///     raw: ProductBranchIdentity,
    /// ) {
    ///     let _ = app.on_branch(raw);
    /// }
    /// ```
    pub fn on_branch(&self, branch: WorthQueryProductBranch) -> WorthQueryProductEntry<'_, Schema> {
        WorthQueryProductEntry {
            application: self,
            branch,
        }
    }

    pub fn on_product(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: WorthQueryProductBranchLease,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        self.inner
            .on_product(execution, product)
            .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
    }

    pub fn on_mutation_product(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        lease: &super::super::application_attempt::WorthQueryApplicationSnapshotLease,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        self.inner
            .on_mutation_product(execution, lease)
            .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
    }

    pub fn on_product_for_owner_recovery(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: WorthQueryProductBranchLease,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        self.inner
            .on_product_for_owner_recovery(execution, product)
            .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
    }
}

impl<'runtime, Schema> WorthQueryProductEntry<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn select_for_managed_index<Stop: From<WorthQueryProductBranchAdmissionDenial>>(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        before_basis: impl FnOnce(
            &worth_runtime_world::facade::ProductBranchObservation,
        ) -> Result<(), Stop>,
    ) -> Result<WorthQuerySelectedProductOperation<'runtime, Schema>, Stop> {
        self.application
            .inner
            .on_branch(self.branch)
            .select_for_managed_index(request, before_basis)
            .map(|inner| WorthQuerySelectedProductOperation::from_inner(self.application, inner))
    }

    pub fn select(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        self.application
            .inner
            .on_branch(self.branch)
            .select(phase)
            .map(|inner| WorthQuerySelectedProductOperation::from_inner(self.application, inner))
    }

    pub fn select_with_request(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        self.application
            .inner
            .on_branch(self.branch)
            .select_with_request(request)
            .map(|inner| WorthQuerySelectedProductOperation::from_inner(self.application, inner))
    }

    #[doc(hidden)]
    pub fn select_with_request_for_test(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        Self::select_with_request(self, request)
    }
}
