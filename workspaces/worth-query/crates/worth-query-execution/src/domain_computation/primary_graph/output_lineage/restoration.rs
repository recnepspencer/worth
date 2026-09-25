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
        resources: Option<
            super::super::application_contribution::WorthQueryProducerDemandResources,
        >,
    ) {
        let source = SemanticSource {
            runtime_authority,
            schema,
            scope,
            output_binding,
        };
        let occurrence = observation.lifecycle_incarnation();
        let generation_number = observation.reference_generation().get();
        let existing_slot = self.partition_index.at_generation(
            &source,
            occurrence,
            generation_number,
            source_partition_identity,
        );
        let generation = self
            .by_source
            .entry(source.clone())
            .or_default()
            .entry(occurrence)
            .or_default()
            .entry(generation_number)
            .or_default();
        if let Some(slot) = existing_slot {
            let recorded = generation
                .get_mut(slot)
                .expect("a restored partition locator must reference retained output authority");
            assert!(
                recorded.source_partition_identity == Some(source_partition_identity)
                    && recorded.source_identity == Some(source_identity)
                    && recorded.producer_dependency_identity == producer_dependency_identity
                    && recorded.idempotency_key_identity == idempotency_key_identity
                    && Arc::ptr_eq(&recorded.correspondence, &correspondence),
                "one restored output partition keeps one exact identity"
            );
            recorded.observed_source_facts = Some(observed_source_facts);
            recorded.resources = resources;
            self.live_occurrences
                .insert(observation.lifecycle_incarnation());
            return;
        }
        let slot = generation.len();
        generation.push(RecordedOutput {
            correspondence,
            source_identity: Some(source_identity),
            source_partition_identity: Some(source_partition_identity),
            producer_dependency_identity,
            idempotency_key_identity,
            observed_source_facts: Some(observed_source_facts),
            resources,
        });
        self.partition_index.insert(
            source,
            occurrence,
            generation_number,
            source_partition_identity,
            slot,
        );
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
        resources: Option<
            super::super::application_contribution::WorthQueryProducerDemandResources,
        >,
    ) {
        let source = SemanticSource {
            runtime_authority,
            schema,
            scope,
            output_binding,
        };
        // Recovery supplies an initial prior correspondence, never a newer
        // publication than one already retained for this partition.
        if self
            .partition_index
            .latest(
                &source,
                super::ProductCoordinate {
                    occurrence,
                    generation,
                },
                source_partition_identity,
            )
            .is_some()
        {
            return;
        }
        let records = self
            .by_source
            .entry(source.clone())
            .or_default()
            .entry(occurrence)
            .or_default()
            .entry(generation)
            .or_default();
        let slot = records.len();
        records.push(RecordedOutput {
            correspondence,
            source_identity: Some(source_identity),
            source_partition_identity: Some(source_partition_identity),
            producer_dependency_identity,
            idempotency_key_identity,
            observed_source_facts: None,
            resources,
        });
        self.partition_index.insert(
            source,
            occurrence,
            generation,
            source_partition_identity,
            slot,
        );
        self.live_occurrences.insert(occurrence);
    }
}
