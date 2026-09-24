use std::any::TypeId;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{
    latest_output_in_partition, latest_output_matching, ProductCoordinate, RecordedSourceIdentity,
    SemanticSource, WorthQueryApplicationOutputLineage, WorthQueryOutputSourcePosture,
    WorthQueryProducerLineageHead,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn producer_head<Binding: 'static>(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        source_partition_identity: [u8; 32],
    ) -> Option<WorthQueryProducerLineageHead> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: TypeId::of::<Binding>(),
        };
        let versions = self.by_source.get(&source)?;
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        loop {
            if let Some(recorded) = versions
                .get(&coordinate.occurrence)
                .and_then(|history| {
                    latest_output_in_partition(
                        history,
                        coordinate.generation,
                        source_partition_identity,
                    )
                })
                .map(|(_, recorded)| recorded)
            {
                return Some(WorthQueryProducerLineageHead {
                    occurrence: coordinate.occurrence,
                    dependency_identity: recorded.producer_dependency_identity,
                    idempotency_key_identity: recorded.idempotency_key_identity,
                });
            }
            coordinate = self.origins.get(&coordinate.occurrence).copied()?;
        }
    }

    pub(in crate::domain_computation::primary_graph) fn source_facts_for_receipt(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        receipt: &WorthQueryApplicationCommitReceipt,
        maximum_work: usize,
    ) -> Result<
        Option<(
            std::sync::Arc<[super::super::application_attempt::WorthQueryApplicationObservedFact]>,
            usize,
        )>,
        (),
    > {
        let Some(output_binding) = receipt.output_correspondence().binding_type() else {
            return Ok(None);
        };
        let Some(source_identity) = receipt.idempotency_binding().source_identity() else {
            return Ok(None);
        };
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope: receipt.principal_scope().scope(),
            output_binding,
        };
        let Some(versions) = self.by_source.get(&source) else {
            return Ok(None);
        };
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        let mut work = 0_usize;
        loop {
            work = work.checked_add(1).ok_or(())?;
            if work > maximum_work {
                return Err(());
            }
            if let Some(recorded) = versions.get(&coordinate.occurrence).and_then(|history| {
                latest_output_matching(history, coordinate.generation, |recorded| {
                    recorded.source_identity
                        == Some(RecordedSourceIdentity::Runtime(
                            crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new(source_identity),
                        ))
                        && std::ptr::eq(
                            recorded.correspondence.as_ref(),
                            receipt.output_correspondence(),
                        )
                })
            }) {
                let Some(facts) = recorded.observed_source_facts.as_ref() else {
                    return Ok(None);
                };
                return Ok(Some((std::sync::Arc::clone(facts), work)));
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
        maximum_work: usize,
    ) -> Result<
        Option<(
            std::sync::Arc<[super::super::application_attempt::WorthQueryApplicationObservedFact]>,
            usize,
        )>,
        (),
    > {
        let Some(restored) = settlement.restored_source.as_ref() else {
            return Ok(None);
        };
        let Some(output_binding) = settlement.output_correspondence.binding_type() else {
            return Ok(None);
        };
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope: restored.scope,
            output_binding,
        };
        let Some(versions) = self.by_source.get(&source) else {
            return Ok(None);
        };
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        let mut work = 0_usize;
        loop {
            work = work.checked_add(1).ok_or(())?;
            if work > maximum_work {
                return Err(());
            }
            if let Some(recorded) = versions.get(&coordinate.occurrence).and_then(|history| {
                latest_output_matching(history, coordinate.generation, |recorded| {
                    recorded.source_identity
                        == Some(RecordedSourceIdentity::Checkpoint(restored.identity))
                        && std::ptr::eq(
                            recorded.correspondence.as_ref(),
                            settlement.output_correspondence.as_ref(),
                        )
                })
            }) {
                let Some(facts) = recorded.observed_source_facts.as_ref() else {
                    return Ok(None);
                };
                return Ok(Some((std::sync::Arc::clone(facts), work)));
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
    ) -> bool {
        self.source_facts_for_receipt(runtime_authority, schema, observation, receipt, usize::MAX)
            .is_ok_and(|facts| facts.is_some())
    }

    pub(in crate::domain_computation::primary_graph) fn source_posture_for_any_output_binding(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        current_runtime_identity: crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity,
        current_checkpoint_identity: crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
    ) -> WorthQueryOutputSourcePosture {
        let mut retained_output = false;
        for output_binding in output_bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: *output_binding,
            };
            let Some(versions) = self.by_source.get(&source) else {
                continue;
            };
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            loop {
                if let Some(recorded) = versions
                    .get(&coordinate.occurrence)
                    .and_then(|history| history.range(..=coordinate.generation).next_back())
                    .map(|(_, recorded)| recorded)
                {
                    if recorded.iter().any(|recorded| {
                        recorded.source_identity.is_some_and(|identity| {
                            identity.matches_current(
                                current_runtime_identity,
                                current_checkpoint_identity,
                            )
                        })
                    }) {
                        return WorthQueryOutputSourcePosture::Exact(*output_binding);
                    }
                    retained_output = true;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        if retained_output {
            WorthQueryOutputSourcePosture::Drifted
        } else {
            WorthQueryOutputSourcePosture::Absent
        }
    }
}
