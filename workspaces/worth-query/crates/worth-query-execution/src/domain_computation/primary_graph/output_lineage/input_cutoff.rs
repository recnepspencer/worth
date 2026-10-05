//! A prior performed output selected through its retained partition locator.

mod generation_append;
mod selected_basis;
mod stable_cutover;
mod stable_publication;
mod verification;
pub(in crate::domain_computation::primary_graph) use stable_cutover::{
    PublishedStableLineage, StablePublicationStop,
};
pub(super) use stable_publication::performed_fact_sequence;
pub(in crate::domain_computation::primary_graph) use stable_publication::{
    prepare_stable_address, PreparedStableLineageAddress, StableEqualityConsequence,
};

pub(in crate::domain_computation::primary_graph) use selected_basis::PreparedInputCutoffBasis;
pub(in crate::domain_computation::primary_graph) use verification::{
    cutoff_declines, InputCutoffDecision, InputCutoffVerificationStop, VerifiedInputCutoff,
};

use std::{
    any::TypeId,
    sync::{Arc, OnceLock},
};

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::{ProductBranchIncarnation, ProductBranchObservation};

use super::{
    invalidation::{FullVerificationReason, InvalidationEditAdmission},
    prepared_slot::tree_work,
    PreparedInputReuseKey, ProductCoordinate, RecordedOutput, RecordedSettlementIdentity,
    SealedNativeOutputWitness, SemanticSource, WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    invariant_projection::ConsumedOutputEvidence,
};

/// Pins the exact owner-selected record, including its retained capacity and
/// source evidence. A restored row may lack the proofs needed for input reuse.
pub(in crate::domain_computation::primary_graph) struct RetainedInputCutoffCandidate {
    cell: Arc<OnceLock<RecordedOutput>>,
}

impl RetainedInputCutoffCandidate {
    pub(super) fn from_exact_cell(cell: Arc<OnceLock<RecordedOutput>>) -> Self {
        Self { cell }
    }

    pub(super) fn recorded(&self) -> &RecordedOutput {
        self.cell
            .get()
            .expect("a selected partition cell stays published while pinned")
    }

    /// A stable alias retains the original performed record directly. Its
    /// current settlement and facts remain on the selected alias itself.
    fn originating_recorded(&self) -> Option<&RecordedOutput> {
        let selected = self.recorded();
        match &selected.performed_origin {
            None => Some(selected),
            Some(origin) => origin
                .get()
                .filter(|recorded| recorded.performed_origin.is_none()),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn prepared_input_key(
        &self,
    ) -> Option<&PreparedInputReuseKey> {
        self.recorded().prepared_input_reuse_key.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn completed_handler_fact_count(
        &self,
    ) -> Option<usize> {
        self.originating_recorded()?
            .completed_handler_facts
            .as_ref()
            .map(|boundary| boundary.handler_fact_count())
    }

    pub(in crate::domain_computation::primary_graph) fn completed_decision_reuse(
        &self,
    ) -> Option<&super::CompletedDecisionReuseProof> {
        self.originating_recorded()?
            .completed_decision_reuse
            .as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn native_output_witness(
        &self,
    ) -> Option<&SealedNativeOutputWitness> {
        self.originating_recorded()?.native_output_witness()
    }

    pub(in crate::domain_computation::primary_graph) fn settlement_identity(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.recorded().settlement_identity
    }

    pub(in crate::domain_computation::primary_graph) fn consumed_outputs(
        &self,
    ) -> &[ConsumedOutputEvidence] {
        &self.recorded().consumed_outputs
    }

    pub(in crate::domain_computation::primary_graph) fn verification_requirement(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<FullVerificationReason>, CompanionPreflightStop> {
        admission.charge_external_work(1)?;
        Ok(self.recorded().verification_requirement())
    }

    pub(in crate::domain_computation::primary_graph) fn observed_source_facts(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<[WorthQueryApplicationObservedFact]>>, CompanionPreflightStop> {
        admission.charge_external_work(1)?;
        Ok(self.recorded().observed_source_facts())
    }
}

impl WorthQueryApplicationOutputLineage {
    /// Follow the same product partition and fork ancestry as `producer_head`.
    /// Every index descent, ancestor lookup and Arc pin consumes the caller's
    /// existing admission before the read, including on a later denial.
    pub(in crate::domain_computation::primary_graph) fn prior_input_cutoff_candidate<
        Binding: 'static,
    >(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &ProductBranchObservation,
        source_partition_identity: [u8; 32],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<RetainedInputCutoffCandidate>, CompanionPreflightStop> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: TypeId::of::<Binding>(),
        };
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        loop {
            if let Some((generation, slot)) = self.partition_index.latest_admitted(
                &source,
                coordinate,
                source_partition_identity,
                admission,
            )? {
                let cell = self.recorded_cell_at_partition_slot_admitted(
                    &source,
                    coordinate.occurrence,
                    generation,
                    source_partition_identity,
                    slot,
                    admission,
                )?;
                return Ok(Some(RetainedInputCutoffCandidate { cell }));
            }
            let ancestry_work = tree_work::<ProductBranchIncarnation>(self.origins.len())
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            admission.charge_external_work(ancestry_work)?;
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
        }
    }
}
