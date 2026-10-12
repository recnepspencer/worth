use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use crate::domain_computation::primary_graph::{
    application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_runtime_world::facade::ProductBranchObservation;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn retain_product_application_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryApplicationBasisLease, WorthQueryProductBranchAdmissionDenial> {
        if observation.owner_identity() != self.product_runtime.owner.owner_identity() {
            return Err(WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::ForeignOwner,
            ));
        }
        let mut lease = self
            .basis_leases
            .register(
                execution,
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
            WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::RetentionCapacityExhausted,
            )
        }
        Registration::Basis(Basis::RetentionIdentityExhausted) => {
            WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::RetentionIdentityExhausted,
            )
        }
        Registration::Basis(Basis::SnapshotIdentityExhausted) => {
            WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::SnapshotIdentityExhausted,
            )
        }
        Registration::Basis(_) => WorthQueryProductBranchAdmissionDenial::Refused(
            AdmissionRefusal::RelationalBasisUnavailable,
        ),
        Registration::Snapshot(Snapshot::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => WorthQueryProductBranchAdmissionDenial::Refused(
            AdmissionRefusal::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
        ),
        Registration::Snapshot(Snapshot::SnapshotIdentityExhausted) => {
            WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::SnapshotIdentityExhausted,
            )
        }
        Registration::Snapshot(Snapshot::ForeignRuntime { .. }) => {
            WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::RelationalSnapshotUnavailable,
            )
        }
    }
}
