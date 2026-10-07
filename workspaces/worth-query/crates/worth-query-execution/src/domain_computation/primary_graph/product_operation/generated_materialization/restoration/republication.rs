//! The lineage and mark rows of a restored generated output.

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::ProductBranchObservation;

use super::super::ProducerQualification;
use crate::domain_computation::primary_graph::{
    output_lineage::{
        invalidation::{register_republished, FullVerificationReason},
        PreparedNativeOutputWitness,
    },
    WorthQueryApplicationOutputCorrespondence, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// A restoration republishes the exact retained output on the runtime
    /// that performed it. Its row therefore continues the suspended performed
    /// record: the same consumed outputs, completed read and prepared input,
    /// and a witness sealed over the re-created entities at the restoration
    /// commit. The row's first reader compares it in full, so a demand reuses
    /// the restored output instead of performing it a second time.
    ///
    /// A suspended record that retained no performed proof has nothing to
    /// continue. It is recorded as a restored row, Fresh until verified.
    pub(super) fn record_restored_generated_output(
        &self,
        observation: &ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        producer: ProducerQualification,
    ) {
        let graph = &self.primary_provider.graph;
        let owner = &graph.source_owner.invalidation_owner;
        let mut admission = owner.edit_admission();
        let sealed = graph.with_runtime_mut(|runtime| {
            let prepared = PreparedNativeOutputWitness::prepare_republication(
                &correspondence,
                &graph.layout,
                owner,
                &mut admission,
            )
            .ok()??;
            let committed = runtime
                .snapshots()
                .snapshot_for_observation(&observation.basis().relational_basis().observation())
                .ok()?;
            let witness = prepared.finish(&correspondence, runtime, &committed);
            let read_basis = runtime.read_truth().positioned_snapshot(&committed).ok();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &committed);
            witness.zip(read_basis)
        });
        let mut lineage = graph
            .output_lineage
            .lock()
            .expect("application output lineage lock is available");
        let republished = sealed.and_then(|(witness, read_basis)| {
            let republished = lineage.prepare_republication(
                producer.output_binding_type,
                producer.runtime_authority,
                &producer.schema,
                producer.scope,
                producer.output_occurrence,
                producer.output_generation,
                producer.source_partition_identity,
                &correspondence,
                &producer.observed_source_facts,
                witness,
                &mut admission,
            )?;
            Some((republished, read_basis))
        });
        let Some((republished, read_basis)) = republished else {
            lineage.record_restoration(
                producer.output_binding_type,
                producer.runtime_authority,
                producer.schema,
                producer.scope,
                observation,
                correspondence,
                producer.recorded_source_identity,
                producer.source_partition_identity,
                producer.producer_dependency_identity,
                producer.idempotency_key_identity,
                producer.observed_source_facts,
                producer.resources,
                None,
            );
            return;
        };
        let Some(record) = lineage.record_republished_restoration(
            producer.output_binding_type,
            producer.runtime_authority,
            producer.schema,
            producer.scope,
            observation,
            correspondence,
            producer.recorded_source_identity,
            producer.source_partition_identity,
            producer.producer_dependency_identity,
            producer.idempotency_key_identity,
            producer.observed_source_facts,
            producer.resources,
            republished,
        ) else {
            return;
        };
        drop(lineage);
        let Some(witness) = record.witness.get() else {
            // A row that holds no sealed witness has nothing to register:
            // its first reader verifies it in full.
            graph
                .output_lineage
                .lock()
                .expect("application output lineage lock is available")
                .require_settlement_verification(
                    &record.identity,
                    FullVerificationReason::NativeRevisionUnavailable,
                );
            return;
        };
        // The World effect is already authoritative. A row that could not be
        // registered keeps its typed full-verification requirement.
        if let Err(reason) = register_republished(
            owner,
            &record.predecessor,
            Arc::clone(&record.identity),
            record.facts,
            &record.consumed_outputs,
            witness,
            read_basis,
            &mut admission,
        ) {
            graph
                .output_lineage
                .lock()
                .expect("application output lineage lock is available")
                .require_settlement_verification(&record.identity, reason);
        }
    }
}

#[cfg(test)]
mod own_write;
