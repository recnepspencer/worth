use std::sync::Arc;

use super::{WorthQueryPrimaryGraphProvider, WorthQueryPrimaryLogicalGraph};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn install(
        graph: super::WorthQueryPrimaryGraphIntegrationHandle,
        fault_port: Arc<dyn super::fault_port::WorthQueryPrimaryGraphFaultPort>,
    ) -> (
        Arc<crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor>,
        Arc<Self>,
    ){
        let provider = Arc::new(Self {
            graph,
            resource_support: super::resource_support::WorthQueryPrimaryGraphResourceSupport::install(),
            commit_serialization: std::sync::Mutex::new(()),
            live_delivery: crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveDeliverySource::default(),
            attempts: std::sync::Mutex::new(
                super::application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore::default(),
            ),
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
            pending_application_publication: std::sync::Mutex::new(None),
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
