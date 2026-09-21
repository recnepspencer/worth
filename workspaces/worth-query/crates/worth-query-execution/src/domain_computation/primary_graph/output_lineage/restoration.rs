use std::any::TypeId;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{
    RecordedOutput, RecordedSourceIdentity, SemanticSource,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
};

impl WorthQueryApplicationOutputLineage {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn record_restoration(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: RecordedSourceIdentity,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
        observed_source_facts: Arc<
            [super::super::application_attempt::WorthQueryApplicationObservedFact],
        >,
    ) {
        let source = SemanticSource {
            runtime_authority,
            schema,
            scope,
            output_binding,
        };
        let generation = self
            .by_source
            .entry(source)
            .or_default()
            .entry(observation.lifecycle_incarnation())
            .or_default()
            .entry(observation.reference_generation().get())
            .or_default();
        if let Some(recorded) = generation
            .iter_mut()
            .find(|recorded| recorded.source_partition_identity == Some(source_partition_identity))
        {
            assert!(
                recorded.source_identity == Some(source_identity)
                    && recorded.producer_dependency_identity == producer_dependency_identity
                    && recorded.idempotency_key_identity == idempotency_key_identity
                    && Arc::ptr_eq(&recorded.correspondence, &correspondence),
                "one restored output partition keeps one exact identity"
            );
            recorded.observed_source_facts = Some(observed_source_facts);
            self.live_occurrences
                .insert(observation.lifecycle_incarnation());
            return;
        }
        generation.push(RecordedOutput {
            correspondence,
            source_identity: Some(source_identity),
            source_partition_identity: Some(source_partition_identity),
            producer_dependency_identity,
            idempotency_key_identity,
            observed_source_facts: Some(observed_source_facts),
        });
        self.live_occurrences
            .insert(observation.lifecycle_incarnation());
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn record_recovered_prior_output(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: RecordedSourceIdentity,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
    ) {
        let source = SemanticSource {
            runtime_authority,
            schema,
            scope,
            output_binding,
        };
        let generation = self
            .by_source
            .entry(source)
            .or_default()
            .entry(occurrence)
            .or_default()
            .entry(generation)
            .or_default();
        if generation
            .iter()
            .any(|recorded| recorded.source_partition_identity == Some(source_partition_identity))
        {
            return;
        }
        generation.push(RecordedOutput {
            correspondence,
            source_identity: Some(source_identity),
            source_partition_identity: Some(source_partition_identity),
            producer_dependency_identity,
            idempotency_key_identity,
            observed_source_facts: None,
        });
        self.live_occurrences.insert(occurrence);
    }
}
