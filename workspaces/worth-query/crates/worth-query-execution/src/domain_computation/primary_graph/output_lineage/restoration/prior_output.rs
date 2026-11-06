//! One insertion owner for checkpoint priors and Native-restored producer antecedents.
use super::*;
impl WorthQueryApplicationOutputLineage {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn record_prior_output(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        origin: PriorOutputOrigin,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
    ) {
        let (source_identity, checkpoint, required_publication, producing_implementation) =
            match origin {
                PriorOutputOrigin::Recovered {
                    source_identity,
                    checkpoint,
                } => {
                    let implementation = checkpoint
                        .as_ref()
                        .map(|checkpoint| checkpoint.implementation);
                    (Some(source_identity), checkpoint, None, implementation)
                }
                PriorOutputOrigin::NativeRestored(publication) => {
                    let implementation = publication.meaning().implementation;
                    (None, None, Some(publication), Some(implementation))
                }
            };
        let source = SemanticSource {
            runtime_authority,
            schema,
            scope,
            output_binding: self
                .binding_identity(output_binding)
                .expect("recovered binding is installed"),
        };
        // Recovery supplies an initial prior correspondence, never a newer
        // publication than one already retained for this partition.
        if self
            .partition_index
            .latest(
                &source,
                super::super::ProductCoordinate {
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
            super::super::ProductCoordinate {
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
            coupled_completion: None,
            producing_implementation,
            required_publication,
            native_prior_checkpoint: checkpoint.map(
                super::super::native_prior_checkpoint::NativePriorCheckpointLocator::Recovered,
            ),
            computation_source: super::super::ComputationSourceEvidence::unavailable(),
            performed_origin: None,
            consumed_outputs: Arc::from([]),
            completed_handler_facts: None,
            completed_decision_reuse: None,
            prepared_input_reuse_key: None,
            native_output_witness: OnceLock::new(),
            mutable: Mutex::new(RecordedOutputMutable::new(
                Some(super::super::invalidation::FullVerificationReason::CheckpointRestore),
                None,
                super::super::retained_computation::RecordedComputation::Absent(
                    PriorAbsence::Restored,
                ),
            )),
            settlement_identity: super::super::RecordedSettlementIdentity::retain(
                &source,
                super::super::ProductCoordinate {
                    occurrence,
                    generation,
                },
                slot,
            ),
            correspondence,
            source_identity,
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

mod court_doors_for_test;
