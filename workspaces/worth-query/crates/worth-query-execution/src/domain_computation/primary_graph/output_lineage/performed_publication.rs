//! Fill the exact pre-effect lineage address from a performed World publication.

use super::*;

impl WorthQueryApplicationOutputLineage {
    pub(super) fn record_prepared(
        &mut self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        consumed_outputs: Arc<[super::super::invariant_projection::ConsumedOutputEvidence]>,
        prepared: &PreparedOutputLineageSlot,
        completed_handler_facts: Option<
            super::super::application_attempt::CompletedHandlerFactBoundary,
        >,
        completed_decision_reuse: Option<CompletedDecisionReuseProof>,
        prepared_input_reuse_key: Option<PreparedInputReuseKey>,
        native_prior_checkpoint: Option<native_prior_checkpoint::NativePriorCheckpointLocator>,
        retained_capacity: retained_capacity::RetainedLineageCapacity,
    ) -> Arc<RecordedSettlementIdentity> {
        self.record_inner(
            application,
            consumed_outputs,
            Some(prepared),
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_prior_checkpoint,
            Some(retained_capacity),
        )
        .expect("a prepared output slot has a sealed output binding")
    }

    fn record_inner(
        &mut self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        consumed_outputs: Arc<[super::super::invariant_projection::ConsumedOutputEvidence]>,
        prepared: Option<&PreparedOutputLineageSlot>,
        completed_handler_facts: Option<
            super::super::application_attempt::CompletedHandlerFactBoundary,
        >,
        completed_decision_reuse: Option<CompletedDecisionReuseProof>,
        prepared_input_reuse_key: Option<PreparedInputReuseKey>,
        native_prior_checkpoint: Option<native_prior_checkpoint::NativePriorCheckpointLocator>,
        retained_capacity: Option<retained_capacity::RetainedLineageCapacity>,
    ) -> Option<Arc<RecordedSettlementIdentity>> {
        let evidence = application.commit_evidence();
        let correspondence = evidence.output_correspondence();
        let scope = evidence.operation_scope();
        let publication = application.committed_product_publication();
        let coordinate = ProductCoordinate {
            occurrence: publication.product_incarnation(),
            generation: publication.product_generation().get(),
        };
        let output_binding = correspondence.binding_type()?;
        // The performed path borrows the pre-effect source. Constructing a
        // fresh source here could allocate after World has moved.
        let fallback_source = prepared.is_none().then(|| SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        });
        let source = prepared.map_or_else(
            || {
                fallback_source
                    .as_ref()
                    .expect("test-only record owns its source")
            },
            |prepared| &prepared.source,
        );
        let partition = evidence.idempotency().source_partition_identity();
        if let Some(prepared) = prepared {
            assert_eq!(source.runtime_authority, scope.runtime_authority());
            assert_eq!(&source.schema, scope.binding_identity());
            assert_eq!(source.scope, scope.scope());
            assert_eq!(source.output_binding, output_binding);
            assert_eq!(
                prepared.coordinate, coordinate,
                "World performed the prepared product address"
            );
            assert_eq!(
                prepared.partition, partition,
                "prepared partition matches sealed commit evidence"
            );
        } else {
            self.live_occurrences.insert(coordinate.occurrence);
        }
        if prepared.is_none() {
            if let Some(partition) = partition {
                assert!(
                    self.partition_index
                        .at_generation(
                            source,
                            coordinate.occurrence,
                            coordinate.generation,
                            partition
                        )
                        .is_none(),
                    "one product generation may publish one output binding once"
                );
            }
        }
        let generation = prepared.is_none().then(|| {
            self.by_source
                .entry(source.clone())
                .or_default()
                .entry(coordinate.occurrence)
                .or_default()
                .entry(coordinate.generation)
                .or_default()
        });
        if partition.is_none() {
            assert!(
                generation.as_ref().is_none_or(|generation| {
                    generation.iter().all(|cell| {
                        cell.get()
                            .is_some_and(|recorded| recorded.source_partition_identity.is_some())
                    })
                }),
                "one product generation may publish one output binding once"
            );
        }
        let slot = prepared.map_or_else(
            || generation.as_ref().unwrap().len(),
            |slot| slot.identity.slot(),
        );
        let settlement_identity = if let Some(prepared) = prepared {
            assert!(prepared.record_cell.get().is_none());
            Arc::clone(&prepared.identity)
        } else {
            RecordedSettlementIdentity::retain(source, coordinate, slot)
        };
        let recorded = RecordedOutput {
            native_prior_checkpoint,
            performed_origin: None,
            _retained_capacity: retained_capacity,
            consumed_outputs,
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_output_witness: prepared
                .and_then(|slot| slot.native_output_witness.as_ref().map(Arc::clone))
                .map(std::sync::OnceLock::from)
                .unwrap_or_default(),
            mutable: std::sync::Mutex::new(RecordedOutputMutable {
                verification_requirement: evidence.source_fact_verification_requirement().map(
                    |reason| match reason {
                        super::super::provider::RebaseVerificationReason::NativeRevisionUnavailable => {
                            invalidation::FullVerificationReason::NativeRevisionUnavailable
                        }
                        super::super::provider::RebaseVerificationReason::NativeFactRevisionUnavailable(ordinal) => {
                            invalidation::FullVerificationReason::NativeFactRevisionUnavailable(ordinal)
                        }
                        super::super::provider::RebaseVerificationReason::IndexedSelectionDenied(denial) => invalidation::FullVerificationReason::IndexedSelectionDenied(denial),
                        super::super::provider::RebaseVerificationReason::IndexedSelectionFactDenied(ordinal, denial) => invalidation::FullVerificationReason::IndexedSelectionFactDenied(ordinal, denial),
                        super::super::provider::RebaseVerificationReason::UnsupportedDecisionFact => {
                            invalidation::FullVerificationReason::UnsupportedFact
                        }
                        super::super::provider::RebaseVerificationReason::AdmissionDenied(stop) => {
                            invalidation::FullVerificationReason::MarkingAdmissionDenied(stop)
                        }
                    },
                ),
                observed_source_facts: evidence
                    .retain_observed_source_facts()
                    .or_else(|| evidence.retain_verification_source_facts()),
                resources: prepared.and_then(|slot| slot.actual_resources),
            }),
            settlement_identity: Arc::clone(&settlement_identity),
            correspondence: evidence.retain_output_correspondence(),
            source_identity: evidence.idempotency().source_identity().map(|identity| {
                RecordedSourceIdentity::Runtime(
                    super::super::application_query::WorthQueryRuntimeSourceIdentity::new(identity),
                )
            }),
            source_partition_identity: evidence.idempotency().source_partition_identity(),
            producer_dependency_identity: evidence.idempotency().producer_dependency_identity(),
            idempotency_key_identity: *evidence.idempotency().key_identity(),
        };
        if let Some(prepared) = prepared {
            assert!(
                prepared.record_cell.set(recorded).is_ok(),
                "prepared output fills once"
            );
            assert!(
                prepared.partition_cell.set(slot).is_ok(),
                "prepared partition fills once"
            );
        } else {
            let cell = Arc::new(OnceLock::new());
            assert!(cell.set(recorded).is_ok());
            generation.unwrap().push(cell);
            self.partition_index.insert(
                source.clone(),
                coordinate.occurrence,
                coordinate.generation,
                partition,
                slot,
            );
        }
        Some(settlement_identity)
    }
}
