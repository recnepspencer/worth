use crate::domain_computation::primary_graph::application_contribution::PriorAbsence;
use std::any::TypeId;
use std::sync::{Arc, Mutex, OnceLock};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{
    RecordedOutput, RecordedOutputMutable, RecordedSourceIdentity, SemanticSource,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputLineage,
};

mod republication;

/// A recorded checkpoint output and the source facts its verification covers.
pub(in crate::domain_computation::primary_graph) struct RestoredRecord {
    pub(in crate::domain_computation::primary_graph) identity:
        Arc<super::RecordedSettlementIdentity>,
    pub(in crate::domain_computation::primary_graph) facts: super::RetainedSourceFacts,
}

impl WorthQueryApplicationOutputLineage {
    /// The settlement identity of the checkpoint output that candidate
    /// selection recorded at `observation`.
    pub(in crate::domain_computation::primary_graph) fn restored_settlement_identity(
        &self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        source_partition_identity: [u8; 32],
    ) -> Option<Arc<super::RecordedSettlementIdentity>> {
        let source = SemanticSource {
            runtime_authority,
            schema,
            scope,
            output_binding,
        };
        let occurrence = observation.lifecycle_incarnation();
        let generation = observation.reference_generation().get();
        let slot = self.partition_index.at_generation(
            &source,
            occurrence,
            generation,
            source_partition_identity,
        )?;
        let recorded = self
            .by_source
            .get(&source)?
            .get(&occurrence)?
            .get(&generation)?
            .get(slot)?
            .get()?;
        Some(Arc::clone(&recorded.settlement_identity))
    }

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
        observed_source_facts: super::RetainedSourceFacts,
        resources: Option<
            super::super::application_contribution::WorthQueryProducerDemandResources,
        >,
        verified_witness: Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    ) -> Option<RestoredRecord> {
        let computation_source = observed_source_facts.source();
        let observed_source_facts = Arc::clone(observed_source_facts.postconditions());
        self.record_restored_row(
            output_binding,
            runtime_authority,
            schema,
            scope,
            observation,
            correspondence,
            source_identity,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
            observed_source_facts,
            resources,
            verified_witness,
            &mut None,
            computation_source,
        )
    }

    /// One recorder for every restored row. A checkpoint row claims nothing
    /// upstream and is Fresh until verified. A prepared republication is taken
    /// only by a new row, which then continues the performed record instead.
    #[allow(clippy::too_many_arguments)]
    fn record_restored_row(
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
        verified_witness: Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
        republished: &mut Option<republication::RepublishedOutput>,
        computation_source: super::ComputationSourceEvidence,
    ) -> Option<RestoredRecord> {
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
        if existing_slot.is_none()
            && self.partition_index.has_address(
                &source,
                super::ProductCoordinate {
                    occurrence,
                    generation: generation_number,
                },
                source_partition_identity,
            )
        {
            // A selected World publication owns this invisible address.
            return None;
        }
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
                .get(slot)
                .and_then(|cell| cell.get())
                .expect("a restored partition locator must reference retained output authority");
            assert!(
                recorded.source_partition_identity == Some(source_partition_identity)
                    && recorded.source_identity == Some(source_identity)
                    && recorded.producer_dependency_identity == producer_dependency_identity
                    && recorded.idempotency_key_identity == idempotency_key_identity
                    && Arc::ptr_eq(&recorded.correspondence, &correspondence),
                "one restored output partition keeps one exact identity"
            );
            recorded.restore(observed_source_facts, resources, verified_witness);
            let restored = RestoredRecord {
                identity: Arc::clone(&recorded.settlement_identity),
                facts: recorded
                    .observed_source_facts()
                    .expect("a restored output retains its source facts"),
            };
            self.live_occurrences
                .insert(observation.lifecycle_incarnation());
            return Some(restored);
        }
        let slot = generation.len();
        let republished = republished.take();
        let facts = republished.as_ref().map_or_else(
            || Arc::clone(&observed_source_facts),
            |output| output.facts(),
        );
        let mut recorded = RecordedOutput {
            computation_source,
            performed_origin: None,
            _retained_capacity: None,
            consumed_outputs: Arc::from([]),
            completed_handler_facts: None,
            completed_decision_reuse: None,
            prepared_input_reuse_key: None,
            native_output_witness: verified_witness.map(OnceLock::from).unwrap_or_default(),
            mutable: Mutex::new(RecordedOutputMutable::new(
                Some(super::invalidation::FullVerificationReason::CheckpointRestore),
                (Some(observed_source_facts)).map(|facts| computation_source.retain_facts(facts)),
                resources,
                super::retained_computation::RecordedComputation::Absent(PriorAbsence::Restored),
            )),
            settlement_identity: super::RecordedSettlementIdentity::retain(
                &source,
                super::ProductCoordinate {
                    occurrence,
                    generation: generation_number,
                },
                slot,
            ),
            correspondence,
            source_identity: Some(source_identity),
            source_partition_identity: Some(source_partition_identity),
            producer_dependency_identity,
            idempotency_key_identity,
        };
        if let Some(republished) = republished {
            republished.continue_in(&mut recorded);
        }
        let recorded_source = recorded.computation_source;
        let identity = Arc::clone(&recorded.settlement_identity);
        let cell = Arc::new(OnceLock::new());
        assert!(cell.set(recorded).is_ok());
        generation.push(cell);
        self.partition_index.insert(
            source,
            occurrence,
            generation_number,
            Some(source_partition_identity),
            slot,
        );
        self.live_occurrences
            .insert(observation.lifecycle_incarnation());
        let facts = super::RetainedSourceFacts::retain(recorded_source, facts);
        Some(RestoredRecord { identity, facts })
    }

    /// Keep the witness a reader built for a restored row and compared in
    /// full. The row keeps its first proof, which is the one returned.
    pub(in crate::domain_computation::primary_graph) fn retain_restored_witness(
        &self,
        identity: &super::RecordedSettlementIdentity,
        witness: Arc<OnceLock<super::SealedNativeOutputWitness>>,
    ) -> Arc<OnceLock<super::SealedNativeOutputWitness>> {
        let coordinate = identity.coordinate();
        let Some(recorded) = self
            .by_source
            .get(identity.source())
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
            .and_then(|history| history.get(&coordinate.generation))
            .and_then(|records| records.get(identity.slot()))
            .and_then(|cell| cell.get())
        else {
            return witness;
        };
        Arc::clone(recorded.native_output_witness.get_or_init(|| witness))
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
            .latest_unbudgeted(
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
        if self.partition_index.has_address(
            &source,
            super::ProductCoordinate {
                occurrence,
                generation,
            },
            source_partition_identity,
        ) {
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
        let recorded = RecordedOutput {
            computation_source: super::ComputationSourceEvidence::unavailable(),
            performed_origin: None,
            _retained_capacity: None,
            consumed_outputs: Arc::from([]),
            completed_handler_facts: None,
            completed_decision_reuse: None,
            prepared_input_reuse_key: None,
            native_output_witness: OnceLock::new(),
            mutable: Mutex::new(RecordedOutputMutable::new(
                Some(super::invalidation::FullVerificationReason::CheckpointRestore),
                None,
                resources,
                super::retained_computation::RecordedComputation::Absent(PriorAbsence::Restored),
            )),
            settlement_identity: super::RecordedSettlementIdentity::retain(
                &source,
                super::ProductCoordinate {
                    occurrence,
                    generation,
                },
                slot,
            ),
            correspondence,
            source_identity: Some(source_identity),
            source_partition_identity: Some(source_partition_identity),
            producer_dependency_identity,
            idempotency_key_identity,
        };
        let cell = Arc::new(OnceLock::new());
        assert!(cell.set(recorded).is_ok());
        records.push(cell);
        self.partition_index.insert(
            source,
            occurrence,
            generation,
            Some(source_partition_identity),
            slot,
        );
        self.live_occurrences.insert(occurrence);
    }
}
