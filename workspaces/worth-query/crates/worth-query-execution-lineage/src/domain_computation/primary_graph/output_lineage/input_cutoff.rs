//! A prior performed output selected through its retained partition locator.

pub mod executed_adoption;
pub mod generation_append;
pub mod selected_basis;
pub mod stable_cutover;
pub mod stable_publication;
pub mod verification;
pub use stable_cutover::PublishedStableLineage;
pub use stable_cutover::StablePublicationStop;
pub use stable_publication::performed_fact_sequence;
pub use stable_publication::prepare_stable_address;
pub use stable_publication::PreparedStableLineageAddress;
pub use stable_publication::StableEqualityConsequence;

pub use selected_basis::PreparedInputCutoffBasis;
pub use verification::cutoff_declines;
pub use verification::InputCutoffVerificationStop;
pub use verification::OwnedInputCutoffDecision;
pub use verification::VerifiedInputCutoff;

use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;
use std::sync::OnceLock;

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::ProductBranchObservation;

use super::invalidation::FullVerificationReason;
use super::PreparedInputReuseKey;
use super::ProductCoordinate;
use super::RecordedOutput;
use super::RecordedSettlementIdentity;
use super::SealedNativeOutputWitness;
use super::SemanticSource;
use super::WorthQueryApplicationOutputLineage;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;

/// Pins the exact owner-selected record, including its retained capacity and
/// source evidence. A restored row may lack the proofs needed for input reuse.
#[derive(Clone)]
pub struct RetainedInputCutoffCandidate {
    cell: Arc<OnceLock<RecordedOutput>>,
}

impl RetainedInputCutoffCandidate {
    pub fn from_exact_cell(cell: Arc<OnceLock<RecordedOutput>>) -> Self {
        Self { cell }
    }

    pub fn recorded(&self) -> &RecordedOutput {
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

    pub fn required_publication_head(
        &self,
    ) -> Result<
        super::required_publication::head::SelectedRequiredPublicationHead,
        CompanionPreflightStop,
    > {
        super::required_publication::head::SelectedRequiredPublicationHead::retain(&self.cell)
    }

    pub fn prepared_input_key(&self) -> Option<&PreparedInputReuseKey> {
        self.recorded().prepared_input_reuse_key.as_ref()
    }

    /// Only an actual adopting execution replaces the completed computation's
    /// support. Ordinary cutoff aliases keep the original performed boundary.
    fn completed_computation_record(&self) -> Option<&RecordedOutput> {
        self.recorded().completed_computation_record()
    }

    pub fn completed_handler_fact_count(&self) -> Option<usize> {
        self.completed_computation_record()?
            .completed_handler_facts
            .as_ref()
            .map(|boundary| boundary.handler_fact_count())
    }

    pub fn completed_decision_reuse(&self) -> Option<&super::CompletedDecisionReuseProof> {
        self.completed_computation_record()?
            .completed_decision_reuse
            .as_ref()
    }

    pub fn native_output_witness(&self) -> Option<&SealedNativeOutputWitness> {
        self.completed_computation_record()?.native_output_witness()
    }

    pub fn native_output_witness_cell(
        &self,
    ) -> Option<&Arc<std::sync::OnceLock<SealedNativeOutputWitness>>> {
        self.completed_computation_record()?
            .native_output_witness_cell()
    }

    pub fn settlement_identity(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.recorded().settlement_identity
    }

    /// The consumed evidence that answers at `selected` (public #358).
    pub fn consumed_outputs_at(
        &self,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
    ) -> Arc<[ConsumedOutputEvidence]> {
        self.recorded().consumed_outputs_at(selected)
    }

    /// Provenance, for a performed coupled completion only, which is never
    /// transported (`historical_transport` copies performed rows alone).
    pub fn performed_consumed_outputs(&self) -> &[ConsumedOutputEvidence] {
        &self.recorded().consumed_outputs
    }

    pub fn has_consumed_outputs(&self) -> bool {
        self.recorded().has_consumed_outputs()
    }

    /// See `RecordedOutput::retains_facts_without_witness`.
    pub fn retains_facts_without_witness(&self) -> bool {
        self.recorded().retains_facts_without_witness()
    }

    /// The original performed run, not its rebased postconditions, decides this.
    pub fn requires_own_effect_continuation(&self) -> bool {
        self.recorded()
            .computation_source
            .superseded_by_own_effect()
    }

    pub fn verification_requirement(
        &self,
    ) -> Result<Option<FullVerificationReason>, CompanionPreflightStop> {
        Ok(self.recorded().verification_requirement())
    }

    pub fn observed_source_facts(
        &self,
    ) -> Result<Option<super::ComparableSourceFacts>, CompanionPreflightStop> {
        Ok(self
            .recorded()
            .observed_source_facts()
            .and_then(|facts| facts.for_comparison()))
    }
}

impl WorthQueryApplicationOutputLineage {
    /// Follow the same product partition and fork ancestry as `producer_head`.
    /// Every index descent, ancestor lookup and Arc pin consumes the caller's
    /// existing admission before the read, including on a later denial.
    pub fn prior_input_cutoff_candidate<Binding: 'static>(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &ProductBranchObservation,
        source_partition_identity: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedInputCutoffCandidate>, CompanionPreflightStop> {
        self.prior_input_cutoff_candidate_for_binding(
            TypeId::of::<Binding>(),
            scope,
            observation,
            source_partition_identity,
            execution,
        )
    }

    pub fn prior_input_cutoff_candidate_for_binding(
        &self,
        binding: TypeId,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &ProductBranchObservation,
        source_partition_identity: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedInputCutoffCandidate>, CompanionPreflightStop> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: self
                .binding_identity(binding)
                .expect("admitted binding is installed"),
        };
        self.prior_input_cutoff_candidate_in_source(
            &source,
            observation,
            source_partition_identity,
            execution,
        )
    }

    pub fn prior_input_cutoff_candidate_in_source(
        &self,
        source: &SemanticSource,
        observation: &ProductBranchObservation,
        source_partition_identity: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedInputCutoffCandidate>, CompanionPreflightStop> {
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        loop {
            if let Some((generation, slot)) = self.partition_index.latest_admitted(
                execution,
                source,
                coordinate,
                source_partition_identity,
            )? {
                let cell = self.recorded_cell_at_partition_slot_admitted(
                    source,
                    coordinate.occurrence,
                    generation,
                    source_partition_identity,
                    slot,
                )?;
                return Ok(Some(RetainedInputCutoffCandidate { cell }));
            }

            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
        }
    }
}

mod court_doors_for_test;
