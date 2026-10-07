use super::*;

pub(in crate::domain_computation::primary_graph) enum AcceptedCheckpointFactSource {
    Committed(crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt),
    Stable(crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage),
}

impl WorthQueryOutputDemandRegistry {
    /// Capture terminal accepted outputs and their exact committed receipts in
    /// one registry snapshot. The receipt is used only to join owner lineage
    /// facts; a restored record already carries its authenticated payload.
    pub(in crate::domain_computation::primary_graph) fn accepted_checkpoint_records(
        &self,
    ) -> Result<
        Vec<(
            WorthQueryAcceptedOutputCheckpointIdentity,
            Option<AcceptedCheckpointFactSource>,
        )>,
        worth_relational::facade::durability::DurabilityError,
    > {
        let state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(worth_relational::facade::durability::DurabilityError::new(
                worth_relational::facade::durability::RecoveryFailureClass::CheckpointPublicationInFlight,
                "Query accepted-output publication is in flight",
            )),
        };
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
                                posture: WorthQueryAcceptedOutputCheckpointPosture::Performed,
                                source: key.source.checkpoint_identity().bytes(),
                                scope: receipt.principal_scope().scope(),
                                source_partition: idempotency.source_partition_identity()?,
                                producer_dependency: idempotency.producer_dependency_identity(),
                                idempotency_key: *idempotency.key_identity(),
                                resources: completion.resources,
                                roles: receipt.output_correspondence().checkpoint_roles(),
                                producer_facts: None,
                                producer_fact_wire_version: 0,
                            },
                            exact_source.then(|| AcceptedCheckpointFactSource::Committed(receipt.clone())),
                        ))
                    }
                    WorthQueryAcceptedOutputAuthority::Stable(stable) => {
                        let partition = stable.source_partition_identity()?;
                        let exact_source = stable.source_identity()
                            == Some(crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Runtime(
                                crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new(
                                    key.source.runtime_idempotency_identity(),
                                ),
                            ));
                        Some((
                            WorthQueryAcceptedOutputCheckpointIdentity {
                                producer: key.producer.clone(),
                                posture: WorthQueryAcceptedOutputCheckpointPosture::StableReused,
                                source: key.source.checkpoint_identity().bytes(),
                                scope: stable.source_scope(),
                                source_partition: partition,
                                producer_dependency: stable.producer_dependency_identity(),
                                idempotency_key: stable.idempotency_key_identity(),
                                resources: completion.resources,
                                roles: stable.output_correspondence().checkpoint_roles(),
                                producer_facts: None,
                                producer_fact_wire_version: 0,
                            },
                            exact_source.then(|| AcceptedCheckpointFactSource::Stable(stable.clone())),
                        ))
                    }
                    WorthQueryAcceptedOutputAuthority::Restored(restored) => {
                        Some((restored.checkpoint.clone(), None))
                    }
                }
            })
            .collect::<Vec<_>>();
        accepted.sort_by(|left, right| left.0.canonical_cmp(&right.0));
        Ok(accepted)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn accepted_checkpoint_identities(
        &self,
    ) -> Vec<WorthQueryAcceptedOutputCheckpointIdentity> {
        self.accepted_checkpoint_records()
            .expect("the fixture checkpoint has no publication in flight")
            .into_iter()
            .map(|(identity, _)| identity)
            .collect()
    }
}
