//! One funded final owner for an exact selected wave's Query snapshot custody.
use super::WorthQuerySelectedProductOperation;
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease;
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub struct SharedSelectedProductOperation<'runtime, Schema> {
    selected: WorthQuerySelectedProductOperation<'runtime, Schema>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedQueryBasisRetentionStop {
    Basis,
    Admission(CompanionPreflightStop),
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    /// This phase preserves the issued snapshot and its program interpretation;
    /// it performs no Native registration, latest selection or program inspection.
    pub(in crate::domain_computation::primary_graph) fn prepare_shared_query_basis(
        mut self,
    ) -> Result<SharedSelectedProductOperation<'runtime, Schema>, (Self, CompanionPreflightStop)>
    {
        if let Err(stop) = self.application_basis_mut().prepare_shared() {
            return Err((self, stop));
        }
        Ok(SharedSelectedProductOperation { selected: self })
    }
}

impl<'runtime, Schema> SharedSelectedProductOperation<'runtime, Schema> {
    pub fn selected(&self) -> &WorthQuerySelectedProductOperation<'runtime, Schema> {
        &self.selected
    }
}

pub(in crate::domain_computation::primary_graph) trait SharedQueryBasisScopedOps<Schema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn retain_selected_query_basis_admitted(
        &self,
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<
        (
            WorthQueryProductObservationLease,
            WorthQueryApplicationBasisLease,
        ),
        SelectedQueryBasisRetentionStop,
    > {
        self.retain_issued_query_basis_admitted(shared.selected())
    }

    fn retain_issued_query_basis_admitted(
        &self,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> Result<
        (
            WorthQueryProductObservationLease,
            WorthQueryApplicationBasisLease,
        ),
        SelectedQueryBasisRetentionStop,
    > {
        if !std::ptr::addr_eq(self, selected.application()) {
            return Err(SelectedQueryBasisRetentionStop::Basis);
        }
        // The sealed progression owns the unchanged pair bound by on_product.
        // Cloning a local share cannot change its snapshot, Native root, World
        // occurrence or program guard. Actual initialized identity backing is
        // funded by the lease owner before cloning it.
        let product = selected.product().read_lease_ref();

        let basis = selected
            .application_basis()
            .retain_shared()
            .map_err(SelectedQueryBasisRetentionStop::Admission)?;
        Ok((product.retained_clone(), basis))
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > SharedQueryBasisScopedOps<Schema> for HostRuntime
{
}

pub(crate) mod court_doors_for_test;
