//! Crate-boundary internal port for `WorthQueryPrimaryGraphProvider` backing
//! fields. Never exported on `worth-query-execution::facade` or Host.

use std::sync::{Arc, Mutex};

use super::{
    application_attempt_state, application_attempt_work, branch_commit_coordination, fault_port,
    inbound_terminal_index, managed_cleanup_control, outstanding_dispatch,
    pending_application_publication, publication_fault_control, resource_support, session_commit,
    unpublished_idempotency, WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

pub trait PrimaryGraphProviderInternalPort {
    fn managed_cleanup_control_ref(&self) -> &Arc<managed_cleanup_control::ManagedCleanupControl>;
    fn graph_ref(&self) -> &WorthQueryPrimaryGraphIntegrationHandle;
    fn installed_operations_ref(
        &self,
    ) -> &Arc<worth_query_installation::facade::InstalledOperationCatalog>;
    fn resource_support_ref(&self) -> &resource_support::WorthQueryPrimaryGraphResourceSupport;
    fn branch_commit_coordination_ref(
        &self,
    ) -> &branch_commit_coordination::WorthQueryApplicationBranchCommitCoordinator;
    fn live_delivery_ref(&self) -> &super::super::live_delivery::WorthQueryLiveDeliverySource;
    fn attempts_ref(&self) -> &application_attempt_state::SharedApplicationAttemptStore;
    fn application_attempt_operation_control_ref(
        &self,
    ) -> &super::super::application_runtime::WorthQueryApplicationAttemptOperationControl;
    fn application_attempt_work_ref(
        &self,
    ) -> &application_attempt_work::WorthQueryApplicationAttemptWorkLedger;
    fn completed_commit_evidence_ref(
        &self,
    ) -> &Mutex<session_commit::WorthQueryCompletedCommitEvidenceStore>;
    fn unpublished_idempotency_ref(
        &self,
    ) -> &unpublished_idempotency::SharedUnpublishedIdempotencyStore;
    fn receipt_basis_retention_ref(
        &self,
    ) -> &Arc<Mutex<session_commit::WorthQueryReceiptBasisRetentionStore>>;
    fn outstanding_dispatch_ref(&self) -> &outstanding_dispatch::OutstandingDispatchOwner;
    fn inbound_terminal_index_ref(&self)
        -> &inbound_terminal_index::WorthQueryInboundTerminalIndex;
    fn pending_application_publications_ref(
        &self,
    ) -> &pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistryOwner;
    fn fault_port_ref(&self) -> &Arc<dyn fault_port::WorthQueryPrimaryGraphFaultPort>;
    fn publication_fault_control_ref(
        &self,
    ) -> &Arc<publication_fault_control::PublicationFaultControl>;
    fn correction_material_ref(
        &self,
    ) -> &std::sync::OnceLock<super::super::retained_correction::CorrectionMaterialOwner>;
    fn original_meaning_support_ref(
        &self,
    ) -> &std::sync::OnceLock<
        std::sync::Weak<super::super::query_obligations::history::QueryObligationHistory>,
    >;
    fn world_history_ref(
        &self,
    ) -> &std::sync::OnceLock<worth_runtime_world::facade::RuntimeWorldRecoveryPort>;
    fn teardown_sentinel_ref(&self) -> &Mutex<Option<Box<dyn std::any::Any + Send + Sync>>>;
}

impl PrimaryGraphProviderInternalPort for WorthQueryPrimaryGraphProvider {
    fn managed_cleanup_control_ref(&self) -> &Arc<managed_cleanup_control::ManagedCleanupControl> {
        &self.managed_cleanup_control
    }
    fn graph_ref(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.graph
    }
    fn installed_operations_ref(
        &self,
    ) -> &Arc<worth_query_installation::facade::InstalledOperationCatalog> {
        &self.installed_operations
    }
    fn resource_support_ref(&self) -> &resource_support::WorthQueryPrimaryGraphResourceSupport {
        &self.resource_support
    }
    fn branch_commit_coordination_ref(
        &self,
    ) -> &branch_commit_coordination::WorthQueryApplicationBranchCommitCoordinator {
        &self.branch_commit_coordination
    }
    fn live_delivery_ref(&self) -> &super::super::live_delivery::WorthQueryLiveDeliverySource {
        &self.live_delivery
    }
    fn attempts_ref(&self) -> &application_attempt_state::SharedApplicationAttemptStore {
        &self.attempts
    }
    fn application_attempt_operation_control_ref(
        &self,
    ) -> &super::super::application_runtime::WorthQueryApplicationAttemptOperationControl {
        &self.application_attempt_operation_control
    }
    fn application_attempt_work_ref(
        &self,
    ) -> &application_attempt_work::WorthQueryApplicationAttemptWorkLedger {
        &self.application_attempt_work
    }
    fn completed_commit_evidence_ref(
        &self,
    ) -> &Mutex<session_commit::WorthQueryCompletedCommitEvidenceStore> {
        &self.completed_commit_evidence
    }
    fn unpublished_idempotency_ref(
        &self,
    ) -> &unpublished_idempotency::SharedUnpublishedIdempotencyStore {
        &self.unpublished_idempotency
    }
    fn receipt_basis_retention_ref(
        &self,
    ) -> &Arc<Mutex<session_commit::WorthQueryReceiptBasisRetentionStore>> {
        &self.receipt_basis_retention
    }
    fn outstanding_dispatch_ref(&self) -> &outstanding_dispatch::OutstandingDispatchOwner {
        &self.outstanding_dispatch
    }
    fn inbound_terminal_index_ref(
        &self,
    ) -> &inbound_terminal_index::WorthQueryInboundTerminalIndex {
        &self.inbound_terminal_index
    }
    fn pending_application_publications_ref(
        &self,
    ) -> &pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistryOwner
{
        &self.pending_application_publications
    }
    fn fault_port_ref(&self) -> &Arc<dyn fault_port::WorthQueryPrimaryGraphFaultPort> {
        &self.fault_port
    }
    fn publication_fault_control_ref(
        &self,
    ) -> &Arc<publication_fault_control::PublicationFaultControl> {
        &self.publication_fault_control
    }
    fn correction_material_ref(
        &self,
    ) -> &std::sync::OnceLock<super::super::retained_correction::CorrectionMaterialOwner> {
        &self.correction_material
    }
    fn original_meaning_support_ref(
        &self,
    ) -> &std::sync::OnceLock<
        std::sync::Weak<super::super::query_obligations::history::QueryObligationHistory>,
    > {
        &self.original_meaning_support
    }
    fn world_history_ref(
        &self,
    ) -> &std::sync::OnceLock<worth_runtime_world::facade::RuntimeWorldRecoveryPort> {
        &self.world_history
    }
    fn teardown_sentinel_ref(&self) -> &Mutex<Option<Box<dyn std::any::Any + Send + Sync>>> {
        &self.teardown_sentinel
    }
}
