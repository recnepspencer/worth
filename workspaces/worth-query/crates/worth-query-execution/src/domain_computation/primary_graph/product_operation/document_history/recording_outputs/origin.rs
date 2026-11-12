//! A validated single or group completion shares one immutable origin constructor.
use super::*;
impl WorthQueryHistoryRecordingOutputOrigin {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn matches_group_member(
        &self,
        meaning: WorthQueryApplicationIdempotencyBinding,
        endpoint: &WorthQueryApplicationReadObservation,
    ) -> bool {
        self.intent.canonical_key() == meaning.canonical_key()
            && self.intent.has_same_intent(meaning)
            && self.observation.product_observation() == endpoint.product_observation()
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn capture(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        model: super::super::HistoryModelIdentity,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: crate::basis::WorthQueryProductBranch,
        intent: WorthQueryApplicationIdempotencyBinding,
        source: super::super::source_evidence::WorthQueryHistorySourceEvidence<'_>,
        profile: &WorthQueryHistoryOutputProfile,
        observation: Arc<WorthQueryApplicationReadObservation>,
        root_posture: Option<WorthQueryOutputSettlementPosture>,
    ) -> Result<Arc<Self>, WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        let memory = memory::reserve(
            execution,
            execution.has_accounting(),
            (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64,
        )
        .map_err(Denial::Recording)?;
        // The authored source and its completed output endpoint can be different
        // publications. Preserve source identity from the validated receipt.
        let (intent, source) = match source {
            super::super::source_evidence::WorthQueryHistorySourceEvidence::Live(receipt) => (
                receipt.idempotency_binding(),
                receipt
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
            ),
            super::super::source_evidence::WorthQueryHistorySourceEvidence::Recorded(_) => (
                intent,
                observation.product_observation().selected_commit().clone(),
            ),
        };
        Ok(Arc::new(Self {
            recipes: super::super::discovery_recipes::HistoryDiscoveryRecipeRecorder::new(
                execution,
                reserved.record.node,
                model,
            )
            .map_err(Denial::Recording)?,
            node: reserved.record.node,
            candidate,
            expected: reserved.expected.clone(),
            intent,
            source,
            observation,
            profile: profile.clone(),
            root_posture,
            _memory: memory,
        }))
    }
}
