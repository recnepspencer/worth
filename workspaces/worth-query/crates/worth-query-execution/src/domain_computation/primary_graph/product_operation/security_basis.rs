use crate::basis::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease};
use crate::domain_computation::primary_graph::{
    application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_runtime_world::facade::ProductBranchObservation;

pub(in crate::domain_computation) trait WorthQueryProductObservationSource {
    fn product_observation(&self) -> &ProductBranchObservation;

    fn current_security_guard(&self) -> Option<&ProductBranchObservation> {
        None
    }
}

impl WorthQueryProductObservationSource for WorthQueryProductBranchLease {
    fn product_observation(&self) -> &ProductBranchObservation {
        self.observation()
    }
}

impl WorthQueryProductObservationSource for crate::basis::WorthQueryProductObservationLease {
    fn product_observation(&self) -> &ProductBranchObservation {
        self.observation()
    }

    fn current_security_guard(&self) -> Option<&ProductBranchObservation> {
        self.current_security_guard()
    }
}

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
    fn retain_indexed_security_basis(
        &self,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryApplicationBasisLease, WorthQueryProductBranchAdmissionDenial> {
        let graph = self
            .runtime
            .primary_graph()
            .ok_or(WorthQueryProductBranchAdmissionDenial::ObservationRejected)?
            .integration_handle();
        graph
            .with_runtime_mut(|runtime| {
                graph.ensure_primary_indexes_for_basis(
                    runtime,
                    observation.basis().relational_basis(),
                )
            })
            .map_err(|_| WorthQueryProductBranchAdmissionDenial::ObservationRejected)?;
        self.retain_product_application_basis(observation)
    }

    pub(in crate::domain_computation) fn admit_product_security_basis(
        &self,
        product: &impl WorthQueryProductObservationSource,
    ) -> Result<WorthQueryProductSecurityBasis, WorthQueryProductBranchAdmissionDenial> {
        let selected = product.product_observation();
        if let Some(guard) = product.current_security_guard() {
            return self.product_runtime.with_product_observation(
                guard.branch_identity(),
                |current| {
                    if current.lifecycle_incarnation() != guard.lifecycle_incarnation()
                        || current.selected_commit() != guard.selected_commit()
                        || current
                            .basis()
                            .relational_basis()
                            .materialization_is_complete()
                        || selected.lifecycle_incarnation() != guard.lifecycle_incarnation()
                        || !selected
                            .basis()
                            .relational_basis()
                            .materialization_is_complete()
                    {
                        return Err(WorthQueryProductBranchAdmissionDenial::IncarnationChanged);
                    }
                    let application_basis = self.retain_indexed_security_basis(selected)?;
                    Ok(WorthQueryProductSecurityBasis {
                        _observation: selected.clone(),
                        application_basis,
                    })
                },
            );
        }
        self.product_runtime
            .with_product_observation(selected.branch_identity(), |observation| {
                if observation.lifecycle_incarnation() != selected.lifecycle_incarnation() {
                    return Err(WorthQueryProductBranchAdmissionDenial::IncarnationChanged);
                }
                let application_basis = self.retain_indexed_security_basis(&observation)?;
                Ok(WorthQueryProductSecurityBasis {
                    _observation: observation,
                    application_basis,
                })
            })
    }
}
