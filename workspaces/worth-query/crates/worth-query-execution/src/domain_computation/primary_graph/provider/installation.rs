use std::sync::Arc;

use super::{WorthQueryPrimaryGraphProvider, WorthQueryPrimaryLogicalGraph};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn install(
        graph: super::WorthQueryPrimaryGraphIntegrationHandle,
        fault_port: Arc<dyn super::fault_port::WorthQueryPrimaryGraphFaultPort>,
        maximum_concurrent_graph_work: std::num::NonZeroUsize,
    ) -> (
        Arc<crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor>,
        Arc<Self>,
    ){
        let provider = Arc::new(Self {
            graph,
            resource_support: super::resource_support::WorthQueryPrimaryGraphResourceSupport::install(
                maximum_concurrent_graph_work,
            ),
            branch_commit_coordination: Default::default(),
            live_delivery: crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveDeliverySource::default(),
            attempts: std::sync::Arc::new(std::sync::Mutex::new(
                super::application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore::default(),
            )),
            #[cfg(feature = "test-world-operation-control")]
            application_attempt_operation_control: Default::default(),
            application_attempt_work: Default::default(),
            completed_commit_evidence: std::sync::Mutex::new(
                super::session_commit::WorthQueryCompletedCommitEvidenceStore::default(),
            ),
            unpublished_idempotency: std::sync::Arc::new(std::sync::Mutex::new(
                super::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore::new(
                    super::resource_support::UNPUBLISHED_IDEMPOTENCY_CAPACITY,
                ),
            )),
            receipt_basis_retention: std::sync::Mutex::new(Default::default()),
            pending_application_publications: std::sync::Arc::new(std::sync::Mutex::new(
                super::pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistry::new(
                    maximum_concurrent_graph_work.get(),
                ),
            )),
            conditional_commit_journal: std::sync::Mutex::new(Default::default()),
            fault_port,
        });
        let anchor = Arc::new(
            crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install_invariant_capable::<
                WorthQueryPrimaryLogicalGraph,
                Arc<Self>,
            >(Arc::clone(&provider)),
        );
        (anchor, provider)
    }
}
