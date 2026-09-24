use std::{any::TypeId, sync::Arc};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{
    ProductCoordinate, RecordedSourceIdentity, SemanticSource, WorthQueryApplicationOutputLineage,
    WorthQueryProducerLineageHead, WorthQueryRetainedOutputCandidate,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn producer_head<Binding: 'static>(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        source_partition_identity: [u8; 32],
        maximum_work: usize,
    ) -> Result<(Option<WorthQueryProducerLineageHead>, usize), ()> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: TypeId::of::<Binding>(),
        };
        if !self.by_source.contains_key(&source) {
            return (maximum_work > 0).then_some((None, 1)).ok_or(());
        }
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        let mut work = 0_usize;
        loop {
            let (recorded, lookup_work) = self.latest_output_in_partition_budgeted(
                &source,
                coordinate,
                source_partition_identity,
                maximum_work.saturating_sub(work),
            )?;
            work = work.checked_add(lookup_work).ok_or(())?;
            if work > maximum_work {
                return Err(());
            }
            if let Some(recorded) = recorded {
                return Ok((
                    Some(WorthQueryProducerLineageHead {
                        occurrence: coordinate.occurrence,
                        dependency_identity: recorded.producer_dependency_identity,
                        idempotency_key_identity: recorded.idempotency_key_identity,
                    }),
                    work,
                ));
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok((None, work));
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
        let partition = receipt.idempotency_binding().source_partition_identity();
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
        let mut work = 0_usize;
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
            let (recorded, lookup_work) = match partition {
                Some(partition) => self.matching_output_in_partition_budgeted(
                    &source,
                    coordinate,
                    partition,
                    maximum_work.saturating_sub(work),
                    matches,
                )?,
                None => self.matching_legacy_output_budgeted(
                    &source,
                    coordinate,
                    maximum_work.saturating_sub(work),
                    matches,
                )?,
            };
            work = work.checked_add(lookup_work).ok_or(())?;
            if let Some(recorded) = recorded {
                let Some(facts) = recorded
                    .observed_source_facts
                    .as_ref()
                    .filter(|facts| !facts.is_empty())
                else {
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
        if !self.by_source.contains_key(&source) {
            return Ok(None);
        }
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        let mut work = 0_usize;
        loop {
            let (recorded, lookup_work) = self.matching_output_in_partition_budgeted(
                &source,
                coordinate,
                restored.partition,
                maximum_work.saturating_sub(work),
                |recorded| {
                    recorded.source_identity
                        == Some(RecordedSourceIdentity::Checkpoint(restored.identity))
                        && std::ptr::eq(
                            recorded.correspondence.as_ref(),
                            settlement.output_correspondence.as_ref(),
                        )
                },
            )?;
            work = work.checked_add(lookup_work).ok_or(())?;
            if let Some(recorded) = recorded {
                let Some(facts) = recorded
                    .observed_source_facts
                    .as_ref()
                    .filter(|facts| !facts.is_empty())
                else {
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
        let mut candidates = Vec::new();
        let mut work = 0_usize;
        for output_binding in output_bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: *output_binding,
            };
            if !self.by_source.contains_key(&source) {
                work = work.checked_add(1).ok_or(())?;
                if work > maximum_work {
                    return Err(());
                }
                continue;
            }
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            loop {
                let (recorded, lookup_work) = self.latest_output_in_partition_budgeted(
                    &source,
                    coordinate,
                    source_partition_identity,
                    maximum_work.saturating_sub(work),
                )?;
                work = work.checked_add(lookup_work).ok_or(())?;
                if work > maximum_work {
                    return Err(());
                }
                if let Some(recorded) = recorded {
                    candidates.push(WorthQueryRetainedOutputCandidate {
                        binding: *output_binding,
                        correspondence: Arc::clone(&recorded.correspondence),
                        source_identity: recorded.source_identity,
                        observed_source_facts: recorded.observed_source_facts.clone(),
                        resources: recorded.resources,
                    });
                    break;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        Ok((candidates, work))
    }
}
