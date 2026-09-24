use super::*;

impl WorthQueryOutputDemandRegistry {
    /// Capture terminal accepted outputs and their exact committed receipts in
    /// one registry snapshot. The receipt is used only to join owner lineage
    /// facts; a restored record already carries its authenticated payload.
    pub(in crate::domain_computation::primary_graph) fn accepted_checkpoint_records(
        &self,
    ) -> Vec<(
        WorthQueryAcceptedOutputCheckpointIdentity,
        Option<crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt>,
    )> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut accepted = state
            .records
            .iter()
            .filter_map(|(key, record)| {
                let DemandState::Output(WorthQueryOutputProgress {
                    checkpoint: Some(WorthQueryOutputCheckpoint::Ready(completion)),
                    advancement: WorthQueryOutputAdvancement::Idle,
                    ..
                }) = &record.state
                else {
                    return None;
                };
                match &completion.authority {
                    WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                        let idempotency = receipt.idempotency_binding();
                        let exact_source = idempotency.source_identity()
                            == Some(key.source.runtime_idempotency_identity());
                        Some((
                            WorthQueryAcceptedOutputCheckpointIdentity {
                                producer: key.producer.clone(),
                                source: key.source.checkpoint_identity().bytes(),
                                scope: receipt.principal_scope().scope(),
                                source_partition: idempotency.source_partition_identity()?,
                                producer_dependency: idempotency.producer_dependency_identity(),
                                idempotency_key: *idempotency.key_identity(),
                                resources: completion.resources,
                                roles: receipt.output_correspondence().checkpoint_roles(),
                                producer_facts: None,
                            },
                            exact_source.then(|| receipt.clone()),
                        ))
                    }
                    WorthQueryAcceptedOutputAuthority::Restored(restored) => {
                        Some((restored.checkpoint.clone(), None))
                    }
                }
            })
            .collect::<Vec<_>>();
        accepted.sort_by(|left, right| left.0.canonical_cmp(&right.0));
        accepted
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn accepted_checkpoint_identities(
        &self,
    ) -> Vec<WorthQueryAcceptedOutputCheckpointIdentity> {
        self.accepted_checkpoint_records()
            .into_iter()
            .map(|(identity, _)| identity)
            .collect()
    }
}
