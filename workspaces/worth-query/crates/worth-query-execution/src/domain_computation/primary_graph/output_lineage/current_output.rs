mod observation_reader;
#[cfg(test)]
mod receipt_stops;
mod retained_candidates;

use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;
use std::sync::OnceLock;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::ProductCoordinate;
use super::RecordedOutput;
use super::RecordedSourceIdentity;
use super::SemanticSource;
use super::WorthQueryApplicationOutputLineage;
use super::WorthQueryProducerLineageHead;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

pub(in crate::domain_computation::primary_graph) struct RetainedOutputCurrentnessRead {
    pub(in crate::domain_computation::primary_graph) identity:
        Arc<super::RecordedSettlementIdentity>,
    pub(in crate::domain_computation::primary_graph) facts: Option<super::ComparableSourceFacts>,
    pub(in crate::domain_computation::primary_graph) consumed_outputs:
        Arc<[ConsumedOutputEvidence]>,
    pub(in crate::domain_computation::primary_graph) native_output_witness:
        Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    /// The facts and the witness are everything this output depends on.
    pub(in crate::domain_computation::primary_graph) consumed_nothing: bool,
    pub(in crate::domain_computation::primary_graph) verification_requirement:
        Option<super::invalidation::FullVerificationReason>,
}

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn producer_head<Binding: 'static>(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        source_partition_identity: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<WorthQueryProducerLineageHead>, CompanionPreflightStop> {
        let Some(output_binding) = self.binding_identity(TypeId::of::<Binding>()) else {
            return Ok(None);
        };
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        };
        if !self.by_source.contains_key(&source) {
            return Ok(None);
        }
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        loop {
            execution
                .consult()
                .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
            let recorded = self
                .partition_index
                .latest_admitted(execution, &source, coordinate, source_partition_identity)?
                .map(|(generation, slot)| {
                    self.recorded_at_partition_slot(
                        &source,
                        coordinate.occurrence,
                        generation,
                        Some(source_partition_identity),
                        slot,
                    )
                });
            if let Some(recorded) = recorded {
                let origin = recorded
                    .performed_origin
                    .as_ref()
                    .and_then(|cell| cell.get())
                    .unwrap_or(recorded);
                return Ok(Some(WorthQueryProducerLineageHead {
                    occurrence: coordinate.occurrence,
                    dependency_identity: recorded.producer_dependency_identity,
                    idempotency_key_identity: recorded.idempotency_key_identity,
                    settlement: Arc::clone(&recorded.settlement_identity),
                    may_replay_idempotency: matches!(
                        origin.source_identity,
                        Some(RecordedSourceIdentity::Runtime(_))
                    ),
                    claims_upstream: !recorded.consumed_outputs.is_empty(),
                }));
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
        }
    }

    pub(in crate::domain_computation::primary_graph) fn source_facts_for_receipt(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        receipt: &WorthQueryApplicationCommitReceipt,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedOutputCurrentnessRead>, CompanionPreflightStop> {
        let Some(output_binding) = receipt.output_correspondence().binding_type() else {
            return Ok(None);
        };
        let Some(source_identity) = receipt.idempotency_binding().source_identity() else {
            return Ok(None);
        };
        let partition = receipt.idempotency_binding().source_partition_identity();
        let Some(output_binding) = self.binding_identity(output_binding) else {
            return Ok(None);
        };
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope: receipt.principal_scope().scope(),
            output_binding,
        };
        if !self.by_source.contains_key(&source) {
            return Ok(None);
        }
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        loop {
            let matches = |recorded: &super::RecordedOutput| {
                recorded.source_identity
                        == Some(RecordedSourceIdentity::Runtime(
                            crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new(source_identity),
                        ))
                        && std::ptr::eq(
                            recorded.correspondence.as_ref(),
                            receipt.output_correspondence(),
                        )
            };
            let (recorded, _) = match partition {
                Some(partition) => self.matching_output_in_partition_budgeted(
                    &source, coordinate, partition, execution, matches,
                )?,
                None => {
                    self.matching_legacy_output_budgeted(&source, coordinate, execution, matches)?
                }
            };
            if let Some(recorded) = recorded {
                return if self.implementation_is_current(recorded) {
                    retained_currentness_read(recorded)
                } else {
                    Ok(None)
                };
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
        }
    }

    pub(in crate::domain_computation::primary_graph) fn source_facts_for_restored_output(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        settlement: &crate::domain_computation::primary_graph::WorthQueryOutputDemandSettlement,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedOutputCurrentnessRead>, CompanionPreflightStop> {
        let Some(restored) = settlement.restored_source.as_ref() else {
            return Ok(None);
        };
        let Some(output_binding) = settlement.output_correspondence.binding_type() else {
            return Ok(None);
        };
        let Some(output_binding) = self.binding_identity(output_binding) else {
            return Ok(None);
        };
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope: restored.scope,
            output_binding,
        };
        if !self.by_source.contains_key(&source) {
            return Ok(None);
        }
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        loop {
            let (recorded, _) = self.matching_output_in_partition_budgeted(
                &source,
                coordinate,
                restored.partition,
                execution,
                |recorded| {
                    recorded.source_identity
                        == Some(RecordedSourceIdentity::Checkpoint(restored.identity))
                        && std::ptr::eq(
                            recorded.correspondence.as_ref(),
                            settlement.output_correspondence.as_ref(),
                        )
                },
            )?;
            if let Some(recorded) = recorded {
                return if self.implementation_is_current(recorded) {
                    retained_currentness_read(recorded)
                } else {
                    Ok(None)
                };
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
        }
    }

    pub(in crate::domain_computation::primary_graph) fn current_output_matches_receipt(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        receipt: &WorthQueryApplicationCommitReceipt,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, CompanionPreflightStop> {
        self.source_facts_for_receipt(runtime_authority, schema, observation, receipt, execution)
            .map(|read| {
                read.and_then(|read| read.facts)
                    .is_some_and(|facts| !facts.is_empty())
            })
    }

    /// A stable authority names one immutable alias row. Selection must still
    /// find that row as the latest output in its semantic partition; a newer
    /// alias supersedes it even when both share a Product generation.
    pub(in crate::domain_computation::primary_graph) fn source_facts_for_stable_output(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        stable: &super::PublishedStableLineage,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedOutputCurrentnessRead>, CompanionPreflightStop> {
        let identity = stable.exact_settlement();
        let source = identity.source();
        if source.runtime_authority != runtime_authority || &source.schema != schema {
            return Ok(None);
        }
        let Some(partition) = stable.source_partition_identity() else {
            return Ok(None);
        };
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        loop {
            execution
                .consult()
                .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
            let recorded = self
                .partition_index
                .latest_admitted(execution, source, coordinate, partition)?
                .map(|(generation, slot)| {
                    self.recorded_at_partition_slot(
                        source,
                        coordinate.occurrence,
                        generation,
                        Some(partition),
                        slot,
                    )
                });
            if let Some(recorded) = recorded {
                return if self.implementation_is_current(recorded)
                    && Arc::ptr_eq(&recorded.settlement_identity, identity)
                {
                    retained_currentness_read(recorded)
                } else {
                    Ok(None)
                };
            }

            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
        }
    }
}

fn retained_currentness_read(
    recorded: &RecordedOutput,
) -> Result<Option<RetainedOutputCurrentnessRead>, CompanionPreflightStop> {
    let facts = recorded
        .observed_source_facts()
        .and_then(|facts| facts.for_comparison())
        .filter(|facts| !facts.is_empty());
    // Missing comparable facts must not hide the original consumed lineage.
    // With neither, preserve the own-publication fallback at the caller.
    if facts.is_none() && recorded.consumed_outputs.is_empty() {
        return Ok(None);
    }
    let origin = recorded
        .performed_origin
        .as_ref()
        .and_then(|cell| cell.get())
        .unwrap_or(recorded);
    Ok(Some(RetainedOutputCurrentnessRead {
        identity: Arc::clone(&recorded.settlement_identity),
        facts,
        consumed_outputs: Arc::clone(&recorded.consumed_outputs),
        native_output_witness: origin.native_output_witness_cell().map(Arc::clone),
        consumed_nothing: recorded.consumed_outputs.is_empty()
            && recorded.performed_origin.is_none(),
        verification_requirement: recorded.verification_requirement(),
    }))
}
