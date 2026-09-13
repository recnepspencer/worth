use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::domain_computation::primary_graph::{
    application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_runtime_world::facade::ProductBranchObservation;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn retain_product_application_basis(
        &self,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryApplicationBasisLease, WorthQueryProductBranchAdmissionDenial> {
        if observation.owner_identity() != self.product_runtime.owner.owner_identity() {
            return Err(WorthQueryProductBranchAdmissionDenial::ForeignOwner);
        }
        let mut lease = self
            .basis_leases
            .register(
                observation.basis().relational_basis().clone(),
                self.primary_provider.graph.clone(),
            )
            .map_err(map_registration_denial)?;
        lease.bind_product_observation(observation);
        Ok(lease)
    }
}
fn map_registration_denial(
    denial: crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistrationDenial,
) -> WorthQueryProductBranchAdmissionDenial {
    use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistrationDenial as Registration;
    use worth_relational::facade::branch::RelationalBranchBasisDenial as Basis;
    use worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial as Snapshot;
    match denial {
        Registration::Basis(Basis::RetentionCapacityExhausted) => {
            WorthQueryProductBranchAdmissionDenial::RetentionCapacityExhausted
        }
        Registration::Basis(Basis::RetentionIdentityExhausted) => {
            WorthQueryProductBranchAdmissionDenial::RetentionIdentityExhausted
        }
        Registration::Basis(Basis::SnapshotIdentityExhausted) => {
            WorthQueryProductBranchAdmissionDenial::SnapshotIdentityExhausted
        }
        Registration::Basis(_) => {
            WorthQueryProductBranchAdmissionDenial::RelationalBasisUnavailable
        }
        Registration::Snapshot(Snapshot::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => WorthQueryProductBranchAdmissionDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Registration::Snapshot(Snapshot::SnapshotIdentityExhausted) => {
            WorthQueryProductBranchAdmissionDenial::SnapshotIdentityExhausted
        }
        Registration::Snapshot(Snapshot::ForeignRuntime { .. }) => {
            WorthQueryProductBranchAdmissionDenial::RelationalSnapshotUnavailable
        }
    }
}
