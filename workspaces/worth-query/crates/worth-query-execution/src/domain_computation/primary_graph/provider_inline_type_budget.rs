//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPrimaryGraphProvider>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/provider.rs", "WorthQueryPrimaryGraphProvider", 128 * 1024);
        owner.field::<Arc<managed_cleanup_control::ManagedCleanupControl>>(
            "managed_cleanup_control",
        );
        owner.field::<WorthQueryPrimaryGraphIntegrationHandle>("graph");
        owner.field::<resource_support::WorthQueryPrimaryGraphResourceSupport>("resource_support");
        owner.field::<branch_commit_coordination::WorthQueryApplicationBranchCommitCoordinator>(
            "branch_commit_coordination",
        );
        owner.field::<super::super::live_delivery::WorthQueryLiveDeliverySource>("live_delivery");
        owner.field::<Arc<Mutex<application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore>>>("attempts");
        owner.field::<super::super::application_runtime::WorthQueryApplicationAttemptOperationControl>("application_attempt_operation_control");
        owner.field::<application_attempt_work::WorthQueryApplicationAttemptWorkLedger>(
            "application_attempt_work",
        );
        owner.field::<Mutex<session_commit::WorthQueryCompletedCommitEvidenceStore>>(
            "completed_commit_evidence",
        );
        owner.field::<Arc<Mutex<unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore>>>(
            "unpublished_idempotency",
        );
        owner.field::<Arc<Mutex<session_commit::WorthQueryReceiptBasisRetentionStore>>>(
            "receipt_basis_retention",
        );
        owner.field::<outstanding_dispatch::OutstandingDispatchOwner>("outstanding_dispatch");
        owner.field::<inbound_terminal_index::WorthQueryInboundTerminalIndex>(
            "inbound_terminal_index",
        );
        owner.field::<pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistryOwner>("pending_application_publications");
        owner.field::<Arc<dyn fault_port::WorthQueryPrimaryGraphFaultPort>>("fault_port");
        owner.field::<Arc<publication_fault_control::PublicationFaultControl>>(
            "publication_fault_control",
        );
        owner.field::<std::sync::OnceLock<worth_runtime_world::facade::RuntimeWorldRecoveryPort>>(
            "world_history",
        );
        owner.field::<Mutex<Option<Box<dyn std::any::Any + Send + Sync>>>>("teardown_sentinel");
    }
    super::application_attempt_state::inline_type_budget::collect(census);
    super::managed_application_recovery::inline_type_budget::collect(census);
}
