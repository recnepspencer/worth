use worth_store::physical_runtime::{
    PhysicalResidencyCounterSnapshot, PhysicalResidencyObservation,
};
use worth_store_io_scheduler::{IoQueueCounterSnapshot, IoQueueExecutedEvidenceSource};

use crate::{
    ObservedPhysicalTrace, PhysicalInterleavingSchedule, PhysicalSimulationPlan,
    PhysicalSimulationPlanIdentity,
};

use super::{
    evidence::{require_resource_observation_within_envelope, PhysicalResourceEnvelopeObservation},
    CounterContractKind, CounterMismatchEvidence, PhysicalCounterEvidenceRow,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalCounterExecutionSources {
    plan_identity: PhysicalSimulationPlanIdentity,
    actor_step_count: u64,
    shortcut_rejection_count: u64,
    blob_chunk_count: u64,
    blob_logical_bytes: u64,
    protected_ranges: u64,
    compaction_candidate_ranges: u64,
    range_comparisons: u64,
    overlapping_ranges: u64,
    copied_pages: u64,
    publication_swaps: u64,
    blocked_reclaims: u64,
    residency: PhysicalResidencyObservation,
    io: IoQueueCounterSnapshot,
}

impl PhysicalCounterExecutionSources {
    pub fn admit_for_plan(
        plan: &PhysicalSimulationPlan,
        schedule: &PhysicalInterleavingSchedule,
        trace: &ObservedPhysicalTrace,
        residency: PhysicalResidencyObservation,
        io_queue: IoQueueExecutedEvidenceSource,
    ) -> Result<Self, CounterMismatchEvidence> {
        require_executed_sources_match_plan(plan, schedule, trace)?;
        let io = io_queue.counters();
        let resource_observation =
            resource_observation_from_sources(plan, residency.counters(), io);
        require_resource_observation_within_envelope(plan, resource_observation)?;
        let compaction = require_compaction_observation_for_contracts(plan, trace)?;
        Ok(Self {
            plan_identity: plan.identity().clone(),
            actor_step_count: schedule.actor_steps().len() as u64,
            shortcut_rejection_count: trace.shortcut_rejections().len() as u64,
            blob_chunk_count: plan
                .blob_harness_topology()
                .map(|topology| topology.chunk_count())
                .unwrap_or(0),
            blob_logical_bytes: plan
                .blob_harness_topology()
                .map(|topology| topology.logical_bytes())
                .unwrap_or(0),
            protected_ranges: compaction.protected_ranges(),
            compaction_candidate_ranges: compaction.candidate_ranges(),
            range_comparisons: compaction.range_comparisons(),
            overlapping_ranges: compaction.overlapping_ranges(),
            copied_pages: compaction.copied_pages(),
            publication_swaps: compaction.publication_swaps(),
            blocked_reclaims: compaction.blocked_reclaims(),
            residency,
            io,
        })
    }
}

fn require_compaction_observation_for_contracts(
    plan: &PhysicalSimulationPlan,
    trace: &ObservedPhysicalTrace,
) -> Result<CompactionCounterSource, CounterMismatchEvidence> {
    if requires_compaction_counter_source(plan) {
        return trace
            .compaction_interlock()
            .map(CompactionCounterSource::Observed)
            .ok_or(CounterMismatchEvidence::MissingCompactionInterlockObservation);
    }
    Ok(CompactionCounterSource::NoClaim)
}

fn requires_compaction_counter_source(plan: &PhysicalSimulationPlan) -> bool {
    plan.counter_contracts()
        .iter()
        .any(|contract| is_compaction_counter(contract.kind()))
}

const fn is_compaction_counter(kind: CounterContractKind) -> bool {
    matches!(
        kind,
        CounterContractKind::ProtectedReferences
            | CounterContractKind::BlockedReclaimAttempts
            | CounterContractKind::PublicationSwaps
            | CounterContractKind::CompactionCandidateRanges
            | CounterContractKind::CopiedPages
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompactionCounterSource {
    Observed(crate::CompactionInterlockObservation),
    NoClaim,
}

impl CompactionCounterSource {
    const fn protected_ranges(self) -> u64 {
        match self {
            Self::Observed(observation) => observation.protected_ranges(),
            Self::NoClaim => 0,
        }
    }

    const fn candidate_ranges(self) -> u64 {
        match self {
            Self::Observed(observation) => observation.candidate_ranges(),
            Self::NoClaim => 0,
        }
    }

    const fn range_comparisons(self) -> u64 {
        match self {
            Self::Observed(observation) => observation.range_comparisons(),
            Self::NoClaim => 0,
        }
    }

    const fn overlapping_ranges(self) -> u64 {
        match self {
            Self::Observed(observation) => observation.overlapping_ranges(),
            Self::NoClaim => 0,
        }
    }

    const fn copied_pages(self) -> u64 {
        match self {
            Self::Observed(observation) => observation.copied_pages(),
            Self::NoClaim => 0,
        }
    }

    const fn publication_swaps(self) -> u64 {
        match self {
            Self::Observed(observation) => observation.publication_swaps(),
            Self::NoClaim => 0,
        }
    }

    const fn blocked_reclaims(self) -> u64 {
        match self {
            Self::Observed(observation) => observation.blocked_reclaims(),
            Self::NoClaim => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalExecutedCounterEvidence {
    pub(crate) rows: Vec<PhysicalCounterEvidenceRow>,
    pub(crate) resource_observation: PhysicalResourceEnvelopeObservation,
    pub(crate) residency_source: super::PhysicalResidencyEvidenceSource,
}

impl PhysicalExecutedCounterEvidence {
    pub fn from_execution_sources(
        plan: &PhysicalSimulationPlan,
        sources: PhysicalCounterExecutionSources,
    ) -> Result<Self, CounterMismatchEvidence> {
        if sources.plan_identity != *plan.identity() {
            return Err(CounterMismatchEvidence::ExecutedEvidencePlanMismatch);
        }
        let rows = plan
            .counter_contracts()
            .iter()
            .map(|contract| {
                PhysicalCounterEvidenceRow::new(
                    contract.kind(),
                    contract.expectation().kind(),
                    observed_counter_count(contract.kind(), &sources),
                )
            })
            .collect();
        let resource_observation =
            resource_observation_from_sources(plan, sources.residency.counters(), sources.io);
        let residency_source =
            super::PhysicalResidencyEvidenceSource::from_store_observation(sources.residency);
        Ok(Self {
            rows,
            resource_observation,
            residency_source,
        })
    }
}

fn require_executed_sources_match_plan(
    plan: &PhysicalSimulationPlan,
    schedule: &PhysicalInterleavingSchedule,
    trace: &ObservedPhysicalTrace,
) -> Result<(), CounterMismatchEvidence> {
    if !schedule.replay_identity_matches_plan(plan)
        || trace.scenario_identity() != plan.scenario_identity()
        || trace.plan_identity() != plan.identity()
    {
        return Err(CounterMismatchEvidence::ExecutedEvidencePlanMismatch);
    }
    Ok(())
}

fn resource_observation_from_sources(
    plan: &PhysicalSimulationPlan,
    residency: PhysicalResidencyCounterSnapshot,
    io: IoQueueCounterSnapshot,
) -> PhysicalResourceEnvelopeObservation {
    PhysicalResourceEnvelopeObservation::new(
        plan.profile(),
        residency.active_operation_bytes(),
        residency.resident_bytes(),
        u64::from(residency.pinned_frames()),
        u64::from(residency.dirty_frames()),
        u64::from(io.peak_queue_depth()),
        u64::from(io.interference_events()),
    )
}

fn observed_counter_count(
    kind: CounterContractKind,
    sources: &PhysicalCounterExecutionSources,
) -> u64 {
    match kind {
        CounterContractKind::ActorStepExact => sources.actor_step_count,
        CounterContractKind::ReplayIdentityExact => 1,
        CounterContractKind::ForbiddenShortcutExact => 0,
        CounterContractKind::ProfileResourceEnvelope => 1,
        CounterContractKind::BlobChunkCountExact => sources.blob_chunk_count,
        CounterContractKind::BlobLogicalBytesExact => sources.blob_logical_bytes,
        CounterContractKind::AllocationBytes => {
            sources.residency.counters().active_operation_bytes()
        }
        CounterContractKind::PagePins => u64::from(sources.residency.counters().pinned_frames()),
        CounterContractKind::IoQueueDepth => u64::from(sources.io.peak_queue_depth()),
        CounterContractKind::ResidentBytes => sources.residency.counters().resident_bytes(),
        CounterContractKind::DirtyPages => u64::from(sources.residency.counters().dirty_frames()),
        CounterContractKind::IoInterferenceEvents => u64::from(sources.io.interference_events()),
        CounterContractKind::LatchWaits => 0,
        CounterContractKind::EpochRetries => 0,
        CounterContractKind::ProtectedReferences => sources.protected_ranges,
        CounterContractKind::Retries => 0,
        CounterContractKind::BlockedReclaimAttempts => sources.blocked_reclaims,
        CounterContractKind::PublicationSwaps => sources.publication_swaps,
        CounterContractKind::ReplayedPages => 0,
        CounterContractKind::CompactionCandidateRanges => sources.compaction_candidate_ranges,
        CounterContractKind::CopiedPages => sources.copied_pages,
        CounterContractKind::FutureS5SpecificCounters => 0,
    }
}
