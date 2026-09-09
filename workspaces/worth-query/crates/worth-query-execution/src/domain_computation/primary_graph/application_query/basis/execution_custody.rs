use super::super::resource_lifecycle::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisLease,
    WorthQueryApplicationBasisReleaseReceipt,
};

/// The basis admitted by the query pipeline owns every obligation for its
/// selected truth. A product selection cannot enter through a component lease.
pub(in crate::domain_computation::primary_graph::application_query) enum WorthQueryApplicationQueryBasisCustody
{
    Relational(WorthQueryApplicationBasisLease),
    Product {
        product: crate::basis::WorthQueryProductBranchLease,
        application_basis: WorthQueryApplicationBasisLease,
    },
}

impl WorthQueryApplicationQueryBasisCustody {
    pub(in crate::domain_computation::primary_graph::application_query) fn product(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductBranchLease> {
        match self {
            Self::Relational(_) => None,
            Self::Product { product, .. } => Some(product),
        }
    }
    fn component(&self) -> &WorthQueryApplicationBasisLease {
        match self {
            Self::Relational(basis) => basis,
            Self::Product {
                application_basis, ..
            } => application_basis,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn identity(
        &self,
    ) -> &WorthQueryApplicationBasisIdentity {
        self.component().identity()
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn version_id(
        &self,
    ) -> worth_relational::facade::identity::VersionId {
        self.component().version_id()
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn snapshot_handle(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.component().snapshot_handle()
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn is_live(&self) -> bool {
        self.component().is_live()
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn preview_session_liveness(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgePreviewSessionLivenessObserver> {
        self.component().preview_session_liveness()
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn retain_for_continuation(
        &self,
    ) -> Result<
        worth_relational::facade::branch::RelationalBranchRetentionLease,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    > {
        self.component().retain_for_continuation()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn release(
        self,
    ) -> WorthQueryApplicationBasisReleaseReceipt {
        match self {
            Self::Relational(basis) => basis.release(),
            Self::Product {
                application_basis, ..
            } => application_basis.release(),
        }
    }
}
