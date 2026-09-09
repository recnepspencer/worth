use crate::basis::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::ProductBranchIdentity;

/// Shared application entry bound to one retained World occurrence. Selecting
/// another occurrence requires a new admission; this context never resolves latest.
pub struct WorthQuerySelectedProductOperation<'runtime, Schema> {
    application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: WorthQueryProductBranchLease,
    application_basis:
        super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn product_runtime(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        &self.product_runtime
    }

    pub fn select_product_branch(
        &self,
        identity: &ProductBranchIdentity,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        self.on_product(self.product_runtime.admit_product_branch(identity)?)
    }

    pub fn on_product(
        &self,
        product: WorthQueryProductBranchLease,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        if product.observation().owner_identity() != self.product_runtime.owner.owner_identity() {
            return Err(WorthQueryProductBranchAdmissionDenial::ForeignOwner);
        }
        let application_basis = self.retain_product_application_basis(product.observation())?;
        Ok(WorthQuerySelectedProductOperation {
            application: self,
            product,
            application_basis,
        })
    }
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub fn product(&self) -> &WorthQueryProductBranchLease {
        &self.product
    }

    pub(crate) fn application(&self) -> &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.application
    }

    pub(crate) fn application_basis(
        &self,
    ) -> &super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease {
        &self.application_basis
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryProductBranchLease,
        super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    ) {
        (self.application, self.product, self.application_basis)
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub(in crate::domain_computation::primary_graph) fn retain_selection(
        &self,
    ) -> Result<Self, WorthQueryProductBranchAdmissionDenial> {
        self.application.on_product(self.product.retained_clone())
    }
}
