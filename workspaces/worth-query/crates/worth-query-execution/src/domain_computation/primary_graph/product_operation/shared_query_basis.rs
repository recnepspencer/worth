//! One funded final owner for an exact selected wave's Query snapshot custody.
use super::WorthQuerySelectedProductOperation;
use crate::basis::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductObservationLease};
use crate::domain_computation::primary_graph::{
    application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    output_lineage::invalidation::InvalidationEditAdmission,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub(in crate::domain_computation) struct SharedSelectedProductOperation<'runtime, Schema> {
    selected: WorthQuerySelectedProductOperation<'runtime, Schema>,
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum SelectedQueryBasisRetentionStop {
    Basis(WorthQueryProductBranchAdmissionDenial),
    Admission(CompanionPreflightStop),
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    /// This phase preserves the issued snapshot and its program interpretation;
    /// it performs no Native registration, latest selection or program inspection.
    pub(in crate::domain_computation::primary_graph) fn prepare_shared_query_basis(
        mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<SharedSelectedProductOperation<'runtime, Schema>, (Self, CompanionPreflightStop)>
    {
        if let Err(stop) = self.application_basis_mut().prepare_shared(admission) {
            return Err((self, stop));
        }
        Ok(SharedSelectedProductOperation { selected: self })
    }
}

impl<'runtime, Schema> SharedSelectedProductOperation<'runtime, Schema> {
    pub(in crate::domain_computation) fn selected(
        &self,
    ) -> &WorthQuerySelectedProductOperation<'runtime, Schema> {
        &self.selected
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn retain_selected_query_basis_admitted(
        &self,
        shared: &SharedSelectedProductOperation<'_, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        (
            WorthQueryProductObservationLease,
            WorthQueryApplicationBasisLease,
        ),
        SelectedQueryBasisRetentionStop,
    > {
        let selected = shared.selected();
        admission
            .charge_external_work(1)
            .map_err(SelectedQueryBasisRetentionStop::Admission)?;
        if !std::ptr::eq(self, selected.application()) {
            return Err(SelectedQueryBasisRetentionStop::Basis(
                WorthQueryProductBranchAdmissionDenial::ForeignOwner,
            ));
        }
        // The sealed progression owns the unchanged pair bound by on_product.
        // Cloning a local share cannot change its snapshot, Native root, World
        // occurrence or program guard. Actual initialized identity backing is
        // funded by the lease owner before cloning it.
        let product = selected.product().read_lease_ref();
        let product_clones = if product.current_security_guard().is_some() {
            4
        } else {
            2
        };
        admission
            .charge_external_work(product_clones)
            .map_err(SelectedQueryBasisRetentionStop::Admission)?;
        let basis = selected
            .application_basis()
            .retain_shared(admission)
            .map_err(SelectedQueryBasisRetentionStop::Admission)?;
        Ok((product.retained_clone(), basis))
    }
}
