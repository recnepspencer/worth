//! Project and stamp one producer's diagnostic evidence before epoch publication.
use crate::data::error::SignalError;
use crate::data::graph::runtime::graph::RetainedNodePayload;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement, RetainedStoragePreparation,
    RetainedStoragePreparationDenial as Denial,
};
use crate::data::trace::{
    CausalityMetadata, RetainedDiagnosticArtifact, RuntimeArtifactFinalizeImage,
    RuntimeArtifactState,
};
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::lineage::LineageRecord;
use crate::diagnostics::state::DiagnosticsState;
use crate::logic::explain::RewiringSummary;
use crate::logic::planner::{ExecutionRecordId, SemanticSegmentId};

mod copy_work;
use copy_work::checkpoint_copy;

pub(crate) struct EpochSemanticSeed {
    pub(crate) node: NodeId,
    pub(crate) execution_record: ExecutionRecordId,
    pub(crate) semantic_segment: SemanticSegmentId,
    pub(crate) before_image: Option<RuntimeArtifactFinalizeImage>,
    pub(crate) rewiring: Option<RewiringSummary>,
}

/// The task classifier uses the artifact image from before lineage stamping.
#[derive(Debug, Clone)]
pub(crate) struct PreparedSemanticArtifactImage {
    pub(crate) after: Option<RuntimeArtifactFinalizeImage>,
}

pub(crate) struct PreparedEpochSemanticNode {
    pub(crate) artifacts: PreparedSemanticArtifactImage,
    pub(crate) lineage: Option<LineageRecord>,
    pub(crate) explanation: Option<ExplanationFact>,
    pub(crate) provenance: Option<ProvenanceFact>,
}

impl SignalGraph {
    /// The projected node may replace all of its old warm/cold heap with an
    /// admitted result. This is the fixed part of the same allocation envelope
    /// used when the semantic image and its two fact payloads are prepared.
    pub(crate) fn epoch_semantic_diagnostic_fixed_bound(
        &self,
        node: NodeId,
        old_node_heap: u64,
        rewiring_heap: u64,
        links: usize,
        work: &mut RetainedStoragePreparation<'_>,
    ) -> Result<u64, SignalError> {
        let contract = self.get_contract(node)?;
        let scope = contract
            .semantics
            .partition_scope
            .retained_heap_charge(work)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?
            .bytes();
        let condition = self
            .node_eval_config(node)?
            .condition
            .retained_heap_charge(work)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?
            .bytes();
        let source = old_node_heap
            .checked_add(scope)
            .and_then(|bytes| bytes.checked_add(condition))
            .and_then(|bytes| bytes.checked_add(rewiring_heap))
            .ok_or_else(|| SignalError::invalid_input("epoch diagnostic source overflow"))?;
        // One envelope is claimed while the projection is constructed and a
        // second covers the later two-fact retained writer charge.
        semantic_projection_allocation(source, links)?
            .checked_mul(2)
            .ok_or_else(|| SignalError::invalid_input("epoch diagnostic bound overflow"))
    }

    #[cfg(test)]
    pub(crate) fn fail_second_semantic_preparation_for_test(&self) {
        self.epoch_output_preparation_fault_after.store(
            (1 << (usize::BITS - 1)) | 2,
            std::sync::atomic::Ordering::SeqCst,
        );
    }

    #[cfg(test)]
    fn check_epoch_semantic_preparation_fault(&self) -> Result<(), SignalError> {
        let flag = 1 << (usize::BITS - 1);
        let previous = self.epoch_output_preparation_fault_after.fetch_update(
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
            |remaining| (remaining > flag).then(|| remaining - 1),
        );
        if matches!(previous, Ok(value) if value == flag + 1) {
            return Err(SignalError::internal(
                "injected second semantic preparation failure",
            ));
        }
        Ok(())
    }

    pub(crate) fn prepare_epoch_semantic_node(
        &self,
        payload: &mut RetainedNodePayload,
        seed: EpochSemanticSeed,
        diagnostics: &mut DiagnosticsState,
        work: &mut RetainedStoragePreparation<'_>,
        preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<PreparedEpochSemanticNode, SignalError> {
        let node = seed.node;
        let contract = self.get_contract(node)?;
        let condition = &self.node_eval_config(node)?.condition;
        let bytes = (|| {
            let mut bytes = payload
                .warm
                .runtime_artifact_state
                .retained_heap_charge(work)?;
            bytes = bytes.checked_add(payload.cold.retained_heap_charge(work)?)?;
            bytes = bytes.checked_add(
                contract
                    .semantics
                    .partition_scope
                    .retained_heap_charge(work)?,
            )?;
            bytes = bytes.checked_add(condition.retained_heap_charge(work)?)?;
            bytes.checked_add(seed.rewiring.retained_heap_charge(work)?)
        })();
        let bytes = bytes
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?
            .bytes();
        let links = seed.rewiring.as_ref().map_or(0, |rewiring| {
            rewiring.removed.len().saturating_add(rewiring.added.len())
        });
        // Projection, historical record, explanation, and provenance each own
        // copies. Include topology-link scopes, notes, and their vector backing.
        let allocation = semantic_projection_allocation(bytes, links)?;
        if let Some(budget) = preparation {
            budget.claim(allocation)?;
        }
        // Work follows copies actually performed. The allocation envelope is
        // a capacity forecast and can be much larger than these operations.
        if let Some(runtime) = payload.warm.runtime_artifact_state.as_ref() {
            let image_heap = runtime
                .hot()
                .retained_heap_charge(work)
                .and_then(|charge| {
                    charge.checked_add(optional_charge(runtime.reuse_boundary_authority(), work)?)
                })
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?
                .bytes();
            checkpoint_copy(work, image_heap)?;
        }
        let after = payload
            .warm
            .runtime_artifact_state
            .as_deref()
            .map(RuntimeArtifactFinalizeImage::from_runtime_state);
        let mut lineage = None;
        if let Some(image) = after.as_ref() {
            copy_work::checkpoint_runtime_detach(
                payload.warm.runtime_artifact_state.as_ref(),
                work,
            )?;
            let (artifact, previous, transition) =
                crate::diagnostics::recorder::derive_lineage_transition(
                    &mut || diagnostics.allocate_lineage_artifact_id(),
                    seed.before_image.as_ref(),
                    image,
                );
            let mut target = crate::data::graph::storage::NodeEvaluationMutation::draft(
                &mut payload.hot,
                &mut payload.warm,
                &mut payload.cold,
            );
            target.stamp_lineage_and_execution(
                artifact,
                seed.execution_record,
                seed.semantic_segment,
            );
            if self.captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::DescriptiveLineage,
            ) {
                lineage = Some(LineageRecord::artifact_transition(
                    diagnostics.allocate_lineage_sequence(),
                    self.observe().current_branch().id,
                    node,
                    artifact,
                    previous,
                    seed.execution_record,
                    seed.semantic_segment,
                    transition,
                ));
            }
        }
        let mut explanation = None;
        let mut provenance = None;
        let policy = self.installed_runtime_policy();
        if self.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::DescriptiveFacts,
        ) && (policy.retains_explanation_facts() || policy.retains_provenance_facts())
        {
            if let Some(runtime) = payload.warm.runtime_artifact_state.as_ref() {
                let cold = payload.cold.as_deref();
                let projection_copy = compact_projection_copy_bytes(
                    runtime,
                    cold.and_then(|cold| cold.retained_artifact.as_ref()),
                    cold.and_then(|cold| cold.causality.as_ref()),
                    &contract.semantics.partition_scope,
                    condition,
                    work,
                )?;
                checkpoint_copy(work, projection_copy)?;
                let mut projection = ExplanationFact::compact_explanation_from_runtime_projection(
                    node,
                    payload.hot.state,
                    contract.semantics.reads,
                    contract.semantics.produces,
                    contract.semantics.partition_scope.clone(),
                    contract.semantics.required_context,
                    condition.clone(),
                    runtime,
                    cold.and_then(|cold| cold.retained_artifact.as_ref()),
                    cold.and_then(|cold| cold.execution_trace),
                    cold.and_then(|cold| cold.causality.as_ref()),
                    seed.rewiring,
                );
                if let Some(rewiring) = projection.rewiring.take() {
                    checkpoint_rewiring_links(work, &rewiring)?;
                    self.attach_rewiring_topology_links(&mut projection, &rewiring);
                    projection.rewiring = Some(rewiring);
                }
                if policy.retains_explanation_facts() {
                    let heap = projection
                        .retained_heap_charge(work)
                        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?
                        .bytes();
                    checkpoint_copy(
                        work,
                        heap.checked_add(19).ok_or_else(|| {
                            SignalError::invalid_input("explanation work overflow")
                        })?,
                    )?;
                    let mut fact = ExplanationFact::from_explanation(&projection);
                    fact.compact_projection = true;
                    explanation = Some(fact);
                }
                if policy.retains_provenance_facts() {
                    let heap = projection
                        .rewiring
                        .retained_heap_charge(work)
                        .and_then(|charge| {
                            charge.checked_add(projection.causal_links.retained_heap_charge(work)?)
                        })
                        .and_then(|charge| {
                            charge.checked_add(optional_charge(
                                projection.causality.as_ref().map(|cause| &cause.kind),
                                work,
                            )?)
                        })
                        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
                    checkpoint_copy(
                        work,
                        heap.bytes().checked_add(10).ok_or_else(|| {
                            SignalError::invalid_input("provenance work overflow")
                        })?,
                    )?;
                    provenance = Some(ProvenanceFact::from_explanation(&projection));
                }
            }
        }
        #[cfg(test)]
        self.check_epoch_semantic_preparation_fault()?;
        Ok(PreparedEpochSemanticNode {
            artifacts: PreparedSemanticArtifactImage { after },
            lineage,
            explanation,
            provenance,
        })
    }
}

fn optional_charge<T: RetainedStorageMeasurement>(
    value: Option<&T>,
    work: &mut RetainedStoragePreparation<'_>,
) -> Result<Charge, Denial> {
    value.map_or(Ok(Charge::ZERO), |value| value.retained_heap_charge(work))
}

fn compact_projection_copy_bytes(
    runtime: &RuntimeArtifactState,
    retained: Option<&RetainedDiagnosticArtifact>,
    causality: Option<&CausalityMetadata>,
    scope: &impl RetainedStorageMeasurement,
    condition: &impl RetainedStorageMeasurement,
    work: &mut RetainedStoragePreparation<'_>,
) -> Result<u64, SignalError> {
    // The historical record clones the runtime, retained cold artifact and
    // causality. The compact fields below make their own selected copies.
    let mut charge = runtime
        .retained_heap_charge(work)
        .and_then(|charge| charge.checked_add(optional_charge(retained, work)?))
        .and_then(|charge| charge.checked_add(optional_charge(causality, work)?))
        .and_then(|charge| charge.checked_add(scope.retained_heap_charge(work)?))
        .and_then(|charge| charge.checked_add(condition.retained_heap_charge(work)?))
        .and_then(|charge| charge.checked_add(optional_charge(runtime.output_identity(), work)?))
        .and_then(|charge| charge.checked_add(runtime.reuse_basis().retained_heap_charge(work)?))
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
    if let Some(retained) = retained {
        charge = charge
            .checked_add(
                retained
                    .changed_regions
                    .retained_heap_charge(work)
                    .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?,
            )
            .and_then(|charge| {
                charge.checked_add(optional_charge(
                    retained.reuse_certification.as_ref(),
                    work,
                )?)
            })
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
    }
    charge = charge
        .checked_add(
            optional_charge(causality, work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?,
        )
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
    Ok(charge.bytes())
}

fn checkpoint_rewiring_links(
    work: &mut RetainedStoragePreparation<'_>,
    rewiring: &RewiringSummary,
) -> Result<(), SignalError> {
    for (edges, scope_note, event_note) in [
        (
            &rewiring.removed,
            "dependency rewired away from current topology",
            "rewiring removed this dependency during apply",
        ),
        (
            &rewiring.added,
            "dependency entered the active topology during rewiring",
            "rewiring added this dependency during apply",
        ),
    ] {
        for dependency in edges {
            let scope = dependency
                .subscription
                .retained_heap_charge(work)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?
                .bytes();
            let copies = scope
                .checked_mul(2)
                .and_then(|bytes| {
                    bytes.checked_add(
                        (std::mem::size_of::<crate::logic::explain::CausalLink>()
                            + scope_note.len()
                            + event_note.len()) as u64,
                    )
                })
                .ok_or_else(|| SignalError::invalid_input("rewiring link work overflow"))?;
            checkpoint_copy(work, copies)?;
        }
    }
    Ok(())
}

fn semantic_projection_allocation(source: u64, links: usize) -> Result<u64, SignalError> {
    source
        .checked_mul(16)
        .and_then(|bytes| {
            bytes.checked_add((links as u64).checked_mul(
                (std::mem::size_of::<crate::logic::explain::CausalLink>() + 256) as u64 * 16,
            )?)
        })
        .and_then(|bytes| {
            bytes.checked_add(
                (std::mem::size_of::<crate::logic::explain::NodeExplanation>()
                    + std::mem::size_of::<ExplanationFact>()
                    + std::mem::size_of::<ProvenanceFact>()
                    + std::mem::size_of::<crate::data::node::NodeColdData>()
                    + std::mem::size_of::<RuntimeArtifactFinalizeImage>()
                    + std::mem::size_of::<LineageRecord>()) as u64,
            )
        })
        .ok_or_else(|| SignalError::invalid_input("epoch diagnostic projection size overflow"))
}
