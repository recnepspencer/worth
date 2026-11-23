//! Native non-driving assessment of one actual selected publication.
use super::consumed_output::ConsumedOutputEvidence;
use super::consumed_output::ConsumedOutputVerification as ResultMeaning;
use super::consumed_output::ConsumedOutputVerificationStop as Stop;
use crate::domain_computation::primary_graph::output_lineage::WorthQueryCurrentOutputCandidate;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;
use std::sync::Arc;
pub mod dependencies;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservedOutputPosture {
    Current,
    Pending,
}

/// Retained assessment of one output, lent without its recursive posting storage.
#[derive(Clone)]
pub struct ObservedOutputCandidate(Arc<dyn ObservationBacking>);

trait ObservationBacking: Send + Sync {
    fn observation(&self) -> &OutputObservation;
}

struct OutputObservation {
    candidate: CandidateEvidence,
    expected: ObservedOutputPosture,
    dependencies: dependencies::PostingDependencies,
}
struct CandidateEvidence {
    observed_source_facts: crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts,
    settlement_identity: Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    consumed_outputs: Arc<[ConsumedOutputEvidence]>,
    verification_requirement: Option<crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason>,
    native_output_witness: Option<Arc<std::sync::OnceLock<crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness>>>,
}
impl ObservationBacking for OutputObservation {
    fn observation(&self) -> &OutputObservation {
        self
    }
}

impl ObservedOutputCandidate {
    pub fn issue(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: WorthQueryCurrentOutputCandidate,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<Self, Stop> {
        let candidate = CandidateEvidence {
            observed_source_facts: candidate.observed_source_facts,
            settlement_identity: candidate.settlement_identity,
            consumed_outputs: candidate.consumed_outputs,
            verification_requirement: candidate.verification_requirement,
            native_output_witness: candidate.native_output_witness,
        };
        let expected = Self::assess(execution, &candidate, owner, runtime, snapshot, selected)?;

        let dependencies = dependencies::PostingDependencies::capture(&candidate, owner)?;

        Ok(Self(Arc::new(OutputObservation {
            candidate,
            expected,
            dependencies,
        })))
    }
    pub fn posting_facts(
        &self,
    ) -> impl Iterator<Item = &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>
    {
        self.0.observation().dependencies.facts()
    }

    pub fn expected(&self) -> ObservedOutputPosture {
        self.0.observation().expected
    }
    pub fn remains_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<bool, Stop> {
        let observation = self.0.observation();
        Ok(Self::assess(
            execution,
            &observation.candidate,
            owner,
            runtime,
            snapshot,
            selected,
        )? == observation.expected)
    }
    fn assess(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &CandidateEvidence,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<ObservedOutputPosture, Stop> {
        let witness = candidate
            .native_output_witness
            .as_ref()
            .filter(|w| w.get().is_some())
            .ok_or(Stop::Unavailable)?;
        // A required target must still exist with its native identity/kind.
        // Authored source and produced output may share an entity: source-first
        // comparison below can establish Pending after a lawful source edit.
        // Only Current requires the complete output content witness unchanged.
        if !witness
            .get()
            .expect("checked sealed witness")
            .roles_remain_present_in(runtime, snapshot, execution)
            .map_err(super::consumed_output::map_admission_stop)?
        {
            return Err(Stop::Unavailable);
        }
        let facts = candidate
            .observed_source_facts
            .for_comparison()
            .ok_or(Stop::Unavailable)?;
        match ConsumedOutputEvidence::observe_at_observation(
            execution,
            &candidate.settlement_identity,
            &facts,
            &candidate.consumed_outputs,
            candidate.verification_requirement,
            witness,
            owner,
            runtime,
            snapshot,
            selected,
        )? {
            ResultMeaning::Current => Ok(ObservedOutputPosture::Current),
            ResultMeaning::ChangedDirectFact(_)
            | ResultMeaning::ChangedUpstream
            | ResultMeaning::PendingUpstream => Ok(ObservedOutputPosture::Pending),
            ResultMeaning::ChangedOutput | ResultMeaning::Unaligned => Err(Stop::Unavailable),
        }
    }
}
