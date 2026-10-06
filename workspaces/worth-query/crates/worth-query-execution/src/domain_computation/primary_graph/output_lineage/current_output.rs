mod observation_reader;
mod retained_candidates;

use std::{
    any::TypeId,
    sync::{Arc, OnceLock},
};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{
    ProductCoordinate, RecordedOutput, RecordedSourceIdentity, SemanticSource,
    WorthQueryApplicationOutputLineage, WorthQueryProducerLineageHead,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

pub(in crate::domain_computation::primary_graph) struct RetainedOutputCurrentnessRead {
    pub(in crate::domain_computation::primary_graph) identity:
        Arc<super::RecordedSettlementIdentity>,
    pub(in crate::domain_computation::primary_graph) facts:
        Arc<[crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact]>,
    pub(in crate::domain_computation::primary_graph) native_output_witness:
        Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    /// The facts and the witness are everything this output depends on.
    pub(in crate::domain_computation::primary_graph) consumed_nothing: bool,
    pub(in crate::domain_computation::primary_graph) verification_requirement:
        Option<super::invalidation::FullVerificationReason>,
    pub(in crate::domain_computation::primary_graph) work: usize,
}

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
                        settlement: Arc::clone(&recorded.settlement_identity),
                        claims_upstream: !recorded.consumed_outputs.is_empty(),
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
    ) -> Result<Option<RetainedOutputCurrentnessRead>, ()> {
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
                return retained_currentness_read(recorded, work, maximum_work);
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
    ) -> Result<Option<RetainedOutputCurrentnessRead>, ()> {
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
                return retained_currentness_read(recorded, work, maximum_work);
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

    /// A stable authority names one immutable alias row. Selection must still
    /// find that row as the latest output in its semantic partition; a newer
    /// alias supersedes it even when both share a Product generation.
    pub(in crate::domain_computation::primary_graph) fn source_facts_for_stable_output(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        stable: &super::PublishedStableLineage,
        maximum_work: usize,
    ) -> Result<Option<RetainedOutputCurrentnessRead>, ()> {
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
        let mut work = 0_usize;
        loop {
            let (recorded, lookup_work) = self.latest_output_in_partition_budgeted(
                source,
                coordinate,
                partition,
                maximum_work.saturating_sub(work),
            )?;
            work = work.checked_add(lookup_work).ok_or(())?;
            if let Some(recorded) = recorded {
                return if Arc::ptr_eq(&recorded.settlement_identity, identity) {
                    retained_currentness_read(recorded, work, maximum_work)
                } else {
                    Ok(None)
                };
            }
            let ancestry_work = super::prepared_slot::tree_work::<
                worth_runtime_world::facade::ProductBranchIncarnation,
            >(self.origins.len())
            .ok_or(())?;
            let ancestry_work = usize::try_from(ancestry_work).map_err(|_| ())?;
            work = work.checked_add(ancestry_work).ok_or(())?;
            if work > maximum_work {
                return Err(());
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
    work: usize,
    maximum_work: usize,
) -> Result<Option<RetainedOutputCurrentnessRead>, ()> {
    let work = work
        .checked_add(3)
        .filter(|work| *work <= maximum_work)
        .ok_or(())?;
    let facts = recorded
        .observed_source_facts()
        .filter(|facts| !facts.is_empty());
    let Some(facts) = facts else { return Ok(None) };
    let origin = recorded
        .performed_origin
        .as_ref()
        .and_then(|cell| cell.get())
        .unwrap_or(recorded);
    Ok(Some(RetainedOutputCurrentnessRead {
        identity: Arc::clone(&recorded.settlement_identity),
        facts,
        native_output_witness: origin.native_output_witness_cell().map(Arc::clone),
        consumed_nothing: recorded.consumed_outputs.is_empty()
            && recorded.performed_origin.is_none(),
        verification_requirement: recorded.verification_requirement(),
        work,
    }))
}
