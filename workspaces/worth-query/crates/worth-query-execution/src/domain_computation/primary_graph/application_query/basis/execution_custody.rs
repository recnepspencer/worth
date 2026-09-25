use super::super::resource_lifecycle::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisLease,
    WorthQueryApplicationBasisReleaseReceipt,
};

/// The basis admitted by the query pipeline owns the exact selected World
/// occurrence and its matching application component lease.
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationQueryBasisCustody {
    product: crate::basis::WorthQueryProductObservationLease,
    application_basis: WorthQueryApplicationBasisLease,
}

impl WorthQueryApplicationQueryBasisCustody {
    pub(super) fn new(
        product: crate::basis::WorthQueryProductObservationLease,
        application_basis: WorthQueryApplicationBasisLease,
    ) -> Self {
        Self {
            product,
            application_basis,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn retained_product(
        &self,
    ) -> crate::basis::WorthQueryProductObservationLease {
        self.product.retained_clone()
    }

    /// The selected snapshot may serve a fresh security stage only while its
    /// complete World occurrence and Relational owner still match that stage.
    pub(in crate::domain_computation::primary_graph) fn can_reuse_security_snapshot_at(
        &self,
        current: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.product.observation() == current
            && self.application_basis.identity().runtime_instance_id()
                == current
                    .basis()
                    .relational_basis()
                    .identity()
                    .runtime_instance_id()
            && self.application_basis.is_live()
    }

    pub(in crate::domain_computation::primary_graph) fn reusable_security_snapshot(
        &self,
        selected_program: Option<&crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation>,
    ) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        self.application_basis
            .carries_selected_program_interpretation(selected_program)
            .then(|| self.application_basis.snapshot_handle())
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn identity(
        &self,
    ) -> &WorthQueryApplicationBasisIdentity {
        self.application_basis.identity()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn version_id(
        &self,
    ) -> worth_relational::facade::identity::VersionId {
        self.application_basis.version_id()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn snapshot_handle(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.application_basis.snapshot_handle()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn is_live(&self) -> bool {
        self.application_basis.is_live()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn release(
        self,
    ) -> WorthQueryApplicationBasisReleaseReceipt {
        self.application_basis.release()
    }
}
