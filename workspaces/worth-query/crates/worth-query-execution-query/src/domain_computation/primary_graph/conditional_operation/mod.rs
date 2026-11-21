use worth_query_execution_lineage::facade::ports::CommittedProductPublicationInternalPort as _;
pub mod authored_predicate;
pub mod authored_provider_source;
pub mod definition_declarations;
pub mod predicate_admission;
pub mod predicate_observation;

pub mod installation {
    pub mod clock_handle;
    pub mod denial;
    pub mod route_binding;
    pub use clock_handle::{ConditionalClockLease, WorthQueryConditionalClockHandle};
    pub use denial::{
        WorthQueryConditionalRuntimeInstallationDenial,
        WorthQueryConditionalRuntimeInstallationDenialKind,
    };
}

pub mod signal_decision_reentry {
    use super::super::primary_truth_branch_identity;
    use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
    use worth_runtime_bridge::facade::{TruthBranchIdentity, TruthSnapshotIdentity};

    pub mod admitted;
    pub mod selected_projections;
    pub use selected_projections::WorthQuerySelectedSignalProjections;

    pub struct WorthQueryConditionalTruthBasis {
        product: crate::basis::WorthQueryProductBranchLease,
        _application_basis:
            super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
        observation: std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>,
        _branch: TruthBranchIdentity,
        snapshot: TruthSnapshotIdentity,
        branch_projection: worth_runtime_bridge::facade::BridgeIdentityEvidence,
        snapshot_projection: worth_runtime_bridge::facade::BridgeIdentityEvidence,
    }

    impl WorthQueryConditionalTruthBasis {
        pub fn from_selected<Schema>(
            selected: crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
                '_,
                Schema,
            >,
        ) -> Self {
            let (_, product, application_basis) = selected.into_parts();
            let observation = product.bridge_source();
            let snapshot = product.bridge_snapshot_identity().clone();
            let branch = primary_truth_branch_identity();
            Self {
                product,
                _application_basis: application_basis,
                observation,
                branch_projection: branch.bridge_admission_evidence(),
                snapshot_projection: snapshot.bridge_admission_evidence(),
                _branch: branch,
                snapshot,
            }
        }

        pub fn retain_selected<Schema>(
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
                '_,
                Schema,
            >,
        ) -> Result<Self, crate::basis::WorthQueryProductBranchAdmissionDenial>
        where
            Schema: worth_query_installation::facade::ApplicationSchema,
        {
            Ok(Self::from_selected(selected.retain_selection(execution)?))
        }

        pub fn is_product_publication(
            &self,
            publication: &super::super::WorthQueryCommittedProductPublication,
        ) -> bool {
            worth_query_execution_lineage::facade::ports::CommittedProductPublicationInternalPort::is_selected_at(
                publication,
                self.product.observation(),
            )
        }

        pub fn snapshot(&self) -> &TruthSnapshotIdentity {
            &self.snapshot
        }

        pub fn signal_basis(&self) -> &worth_signal::facade::branch::AdmittedSignalBranchBasis {
            self.product.signal_basis()
        }

        pub fn owner_identity(&self) -> worth_runtime_world::facade::RuntimeWorldOwnerIdentity {
            self.product
                .observation()
                .branch_identity()
                .owner_identity()
        }

        pub fn product(&self) -> &crate::basis::WorthQueryProductBranchLease {
            &self.product
        }

        pub fn granular_source_read_basis(
            &self,
        ) -> crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis {
            crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis::new(
                self.snapshot.clone(),
                self._branch.clone(),
                std::sync::Arc::clone(&self.observation),
            )
        }

        pub fn snapshot_projection(&self) -> &str {
            self.snapshot_projection.terminal_projection_for_reporting()
        }

        pub fn branch_projection(&self) -> &str {
            self.branch_projection.terminal_projection_for_reporting()
        }
    }
}

pub use authored_predicate::QueryAuthoredPredicateProvider;
pub use installation::{
    WorthQueryConditionalClockHandle, WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
pub use predicate_admission::{
    prepare_dependency_conditional_installation, QueryConditionalComputeContext,
};
pub use signal_decision_reentry::{
    WorthQueryConditionalTruthBasis, WorthQuerySelectedSignalProjections,
};
pub use worth_query_execution_lineage::facade::primary_graph::conditional_operation::{
    classify_bridge_signal, WorthQueryConditionalSignalDecision,
};
