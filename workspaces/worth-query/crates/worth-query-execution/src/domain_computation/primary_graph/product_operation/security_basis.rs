use crate::basis::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease};
use crate::domain_computation::primary_graph::{
    application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_runtime_world::facade::ProductBranchObservation;

/// Security truth is resolved freshly from the selected branch at each
/// admission stage. A different lifecycle occurrence is rejected before this
/// basis retains the current application snapshot; exact data remains in the
/// operation's separately retained product basis.
pub(in crate::domain_computation) struct WorthQueryProductSecurityBasis {
    _observation: ProductBranchObservation,
    application_basis: WorthQueryApplicationBasisLease,
}

impl WorthQueryProductSecurityBasis {
    pub(in crate::domain_computation) fn snapshot_handle(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.application_basis.snapshot_handle()
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn admit_product_security_basis(
        &self,
        product: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryProductSecurityBasis, WorthQueryProductBranchAdmissionDenial> {
        self.product_runtime
            .with_product_observation(product.branch_identity(), |observation| {
                if observation.lifecycle_incarnation()
                    != product.observation().lifecycle_incarnation()
                {
                    return Err(WorthQueryProductBranchAdmissionDenial::IncarnationChanged);
                }
                let application_basis = self.retain_product_application_basis(&observation)?;
                Ok(WorthQueryProductSecurityBasis {
                    _observation: observation,
                    application_basis,
                })
            })
    }
}
