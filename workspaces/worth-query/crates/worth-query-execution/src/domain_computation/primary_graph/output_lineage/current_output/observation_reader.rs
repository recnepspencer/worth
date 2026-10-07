//! The outputs a reader at a retained observation selects.

use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::WorthQueryProducerDemandResources,
    invariant_projection::ConsumedOutputEvidence,
    output_lineage::invalidation::InvalidationEditAdmission,
};

/// One retained output as a retained observation selects it. Holding it
/// keeps its lineage row. The row is the latest one at or before the
/// observation that retention still holds, so its reader verifies it at
/// that observation before using it.
pub(in crate::domain_computation::primary_graph) struct ObservedRetainedOutput {
    pub(in crate::domain_computation::primary_graph) binding: TypeId,
    pub(in crate::domain_computation::primary_graph) stable: super::super::PublishedStableLineage,
    pub(in crate::domain_computation::primary_graph) facts:
        Option<super::super::ComparableSourceFacts>,
    pub(in crate::domain_computation::primary_graph) consumed_outputs:
        Arc<[ConsumedOutputEvidence]>,
    pub(in crate::domain_computation::primary_graph) verification_requirement:
        Option<super::super::invalidation::FullVerificationReason>,
    pub(in crate::domain_computation::primary_graph) native_output_witness:
        Option<Arc<OnceLock<super::super::SealedNativeOutputWitness>>>,
    pub(in crate::domain_computation::primary_graph) resources:
        Option<WorthQueryProducerDemandResources>,
}

impl WorthQueryApplicationOutputLineage {
    /// The latest output of each binding at or before `observation`, through
    /// the origins of its branch occurrence. Each lookup is reserved on the
    /// reader's meter before it reads.
    pub(in crate::domain_computation::primary_graph) fn outputs_at_observation(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        output_bindings: &[TypeId],
        source_partition_identity: [u8; 32],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Vec<ObservedRetainedOutput>, ()> {
        let mut outputs = Vec::new();
        for output_binding in output_bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: *output_binding,
            };
            if !self.by_source.contains_key(&source) {
                admission.charge_external_work(1).map_err(|_| ())?;
                continue;
            }
            let mut coordinate = ProductCoordinate {
                occurrence: observation.lifecycle_incarnation(),
                generation: observation.reference_generation().get(),
            };
            loop {
                let cell = admission
                    .reserved_read(|maximum_work| {
                        self.latest_cell_in_partition_budgeted(
                            &source,
                            coordinate,
                            source_partition_identity,
                            maximum_work,
                        )
                    })
                    .map_err(|_| ())??;
                if let Some((cell, recorded)) = cell.and_then(|cell| Some((cell, cell.get()?))) {
                    let origin = recorded
                        .performed_origin
                        .as_ref()
                        .and_then(|origin| origin.get())
                        .unwrap_or(recorded);
                    outputs.push(ObservedRetainedOutput {
                        binding: *output_binding,
                        stable: super::super::PublishedStableLineage::selected_at(
                            cell,
                            observation,
                        ),
                        facts: recorded
                            .observed_source_facts()
                            .and_then(|facts| facts.for_comparison())
                            .filter(|facts| !facts.is_empty()),
                        consumed_outputs: Arc::clone(&recorded.consumed_outputs),
                        verification_requirement: recorded.verification_requirement(),
                        native_output_witness: origin.native_output_witness_cell().map(Arc::clone),
                        resources: recorded.resources(),
                    });
                    break;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        Ok(outputs)
    }
}
