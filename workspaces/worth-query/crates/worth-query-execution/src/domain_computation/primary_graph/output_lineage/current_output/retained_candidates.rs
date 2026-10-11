//! Retained candidate selection pays the request's meter on every exit.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

/// The latest output one binding recorded for a source, with everything
/// selection compares before it reuses that output.
pub(in crate::domain_computation::primary_graph) struct WorthQueryRetainedOutputCandidate {
    pub(in crate::domain_computation::primary_graph) binding: TypeId,
    pub(in crate::domain_computation::primary_graph) correspondence:
        Arc<crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence>,
    pub(in crate::domain_computation::primary_graph) source_identity:
        Option<RecordedSourceIdentity>,
    pub(in crate::domain_computation::primary_graph) observed_source_facts: Option<super::super::ComparableSourceFacts>,
    /// The performed output's sealed witness: its origin's for a stable
    /// alias. A restored row has none until a full verification builds it.
    pub(in crate::domain_computation::primary_graph) native_output_witness:
        Option<Arc<OnceLock<crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness>>>,
    /// Why the row's settlement is verified in full, when it is: the row was
    /// restored, or the owner could not register it. A commit that could not
    /// rebase its facts states that here and retains none of them.
    pub(in crate::domain_computation::primary_graph) verification_requirement:
        Option<crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason>,
    /// The row's own settlement, as the owner knows it.
    pub(in crate::domain_computation::primary_graph) settlement_identity:
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    pub(in crate::domain_computation::primary_graph) resources: Option<
        crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources,
    >,
    pub(in crate::domain_computation::primary_graph) idempotency_key_identity: [u8; 32],
}

impl WorthQueryApplicationOutputLineage {
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn retained_output_candidates(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        source_partition_identity: [u8; 32],
        maximum_work: usize,
    ) -> Result<(Vec<WorthQueryRetainedOutputCandidate>, usize), ()> {
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: maximum_work as u64,
                maximum_preparation_bytes: 0,
            },
        );
        let candidates = self.retained_output_candidates_metered(
            runtime_authority,
            schema,
            scope,
            occurrence,
            generation,
            output_bindings,
            source_partition_identity,
            &mut admission,
        )?;
        Ok((candidates, admission.charged_work() as usize))
    }

    /// Each binding's latest retained output, with every lookup reserved on
    /// the request's meter before it reads. A lookup that runs out keeps its
    /// reservation spent.
    pub(in crate::domain_computation::primary_graph) fn retained_output_candidates_metered(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        source_partition_identity: [u8; 32],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Vec<WorthQueryRetainedOutputCandidate>, ()> {
        let mut candidates = Vec::new();
        for output_binding in output_bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: self
                    .binding_identity(*output_binding)
                    .expect("output family binding is installed"),
            };
            if !self.by_source.contains_key(&source) {
                admission.charge_external_work(1).map_err(|_| ())?;
                continue;
            }
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            loop {
                // The lookup returns only budget exhaustion; its scan spent
                // the whole reservation before stopping.
                let recorded = admission
                    .reserved_read(|maximum_work| {
                        self.latest_output_in_partition_budgeted(
                            &source,
                            coordinate,
                            source_partition_identity,
                            maximum_work,
                        )
                    })
                    .map_err(|_| ())??;
                if let Some(recorded) = recorded {
                    candidates.push(WorthQueryRetainedOutputCandidate {
                        binding: *output_binding,
                        correspondence: Arc::clone(&recorded.correspondence),
                        source_identity: recorded.source_identity,
                        observed_source_facts: recorded
                            .observed_source_facts()
                            .and_then(|facts| facts.for_comparison()),
                        // A stable alias seals no witness: its output is
                        // its origin's, compared by the origin's witness.
                        native_output_witness: recorded
                            .performed_origin
                            .as_ref()
                            .and_then(|origin| origin.get())
                            .unwrap_or(recorded)
                            .native_output_witness_cell()
                            .map(Arc::clone),
                        verification_requirement: recorded.verification_requirement(),
                        settlement_identity: Arc::clone(&recorded.settlement_identity),
                        resources: recorded.resources(),
                        idempotency_key_identity: recorded.idempotency_key_identity,
                    });
                    break;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        Ok(candidates)
    }
}
