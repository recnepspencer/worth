use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;

pub enum AcceptedCheckpointFactSource {
    Recovered(super::super::WorthQueryRecoveredProducerPublication),
    Committed(crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt),
    Stable(crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage),
}

impl WorthQueryOutputDemandRegistry {
    /// Capture terminal accepted outputs and their exact committed receipts in
    /// one registry snapshot. The receipt is used only to join owner lineage
    /// facts; a restored record already carries its authenticated payload.
    pub(in crate::domain_computation::primary_graph) fn accepted_checkpoint_records(
        &self,
    ) -> Vec<(
        WorthQueryAcceptedOutputCheckpointIdentity,
        Option<AcceptedCheckpointFactSource>,
        Option<TypeId>,
    )> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut accepted = state
            .records
            .iter()
            .filter_map(|(key, record)| {
                let DemandState::Output(output) = &record.state else { return None; };
                if !matches!(output.advancement(), WorthQueryOutputAdvancement::Idle) { return None; }
                let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint else { return None; };
                match &completion.authority {
                    WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                        let idempotency = receipt.idempotency_binding();
                        let exact_source = idempotency.source_identity()
                            == Some(key.source.runtime_idempotency_identity());
                        Some((
                            WorthQueryAcceptedOutputCheckpointIdentity {
                                implementation: idempotency.producing_implementation()?,
                                producer: (key.producer.clone()).into(),
                                posture: WorthQueryAcceptedOutputCheckpointPosture::Performed,
                                source: key.source.checkpoint_identity().bytes(),
                                scope: receipt.principal_scope().scope(),
                                source_partition: idempotency.source_partition_identity()?,
                                producer_dependency: idempotency.producer_dependency_identity(),
                                idempotency_key: *idempotency.key_identity(),
                                roles: (receipt.output_correspondence().checkpoint_roles()).into(),
                                producer_facts: None,
                                producer_fact_wire_version: 0,
 metadata: None,
},
                            exact_source.then(|| AcceptedCheckpointFactSource::Committed(receipt.clone())),
                            receipt.output_correspondence().binding_type(),
                        ))
                    }
                    WorthQueryAcceptedOutputAuthority::Recovered(publication) => {
                        let receipt = publication.view();
                        let idempotency = receipt.idempotency_binding();
                        let exact_source = idempotency.source_identity()
                            == Some(key.source.runtime_idempotency_identity());
                        Some((
                            WorthQueryAcceptedOutputCheckpointIdentity {
                                implementation: idempotency.producing_implementation()?,
                                producer: (key.producer.clone()).into(),
                                posture: WorthQueryAcceptedOutputCheckpointPosture::Performed,
                                source: key.source.checkpoint_identity().bytes(),
                                scope: receipt.principal_scope().scope(),
                                source_partition: idempotency.source_partition_identity()?,
                                producer_dependency: idempotency.producer_dependency_identity(),
                                idempotency_key: *idempotency.key_identity(),
                                roles: (receipt.output_correspondence().checkpoint_roles()).into(),
                                producer_facts: None,
                                producer_fact_wire_version: 0,
 metadata: None,
},
                            exact_source.then(|| AcceptedCheckpointFactSource::Recovered(publication.clone())),
                            receipt.output_correspondence().binding_type(),
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
                                implementation: stable.producing_implementation()?,
                                producer: (key.producer.clone()).into(),
                                posture: WorthQueryAcceptedOutputCheckpointPosture::StableReused,
                                source: key.source.checkpoint_identity().bytes(),
                                scope: stable.source_scope(),
                                source_partition: partition,
                                producer_dependency: stable.producer_dependency_identity(),
                                idempotency_key: stable.idempotency_key_identity(),
                                roles: (stable.output_correspondence().checkpoint_roles()).into(),
                                producer_facts: None,
                                producer_fact_wire_version: 0,
 metadata: None,
},
                            exact_source.then(|| AcceptedCheckpointFactSource::Stable(stable.clone())),
                            stable.output_correspondence().binding_type(),
                        ))
                    }
                    WorthQueryAcceptedOutputAuthority::Restored(restored) => {
                        Some((
                            match &restored.origin {
                                RestoredOutputOrigin::Checkpoint(checkpoint) => checkpoint.clone(),
                                RestoredOutputOrigin::HistoricalSelection(origin) => WorthQueryAcceptedOutputCheckpointIdentity {
                                    implementation: origin.implementation(), producer: Arc::new(origin.publication().producer().to_owned()),
                                    posture: WorthQueryAcceptedOutputCheckpointPosture::StableReused,
                                    source: key.source.checkpoint_identity().bytes(), scope: restored.source_scope,
                                    source_partition: restored.origin.partition(), producer_dependency: origin.dependency(),
                                    idempotency_key: origin.key(), roles: Arc::new(restored.correspondence.checkpoint_roles()),
                                    // Original selected revisions were not rebased into checkpoint evidence.
                                    producer_facts: None, producer_fact_wire_version: 0, metadata: None,
                                },
                            },
                            None,
                            restored.correspondence.binding_type(),
                        ))
                    }
                }
            })
            .collect::<Vec<_>>();
        accepted.sort_by(|left, right| left.0.canonical_cmp(&right.0));
        accepted
    }

    pub(in crate::domain_computation::primary_graph) fn accepted_checkpoint_identities(
        &self,
    ) -> Vec<WorthQueryAcceptedOutputCheckpointIdentity> {
        self.accepted_checkpoint_records()
            .into_iter()
            .map(|(identity, _, _)| identity)
            .collect()
    }
}

mod court_doors_for_test;
