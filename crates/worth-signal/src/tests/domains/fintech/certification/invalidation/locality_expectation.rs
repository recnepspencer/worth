use std::collections::{BTreeMap, BTreeSet};

use crate::tests::domains::fintech::world::{
    FinancialAspect, FinancialLocalityActionTrace, FinancialLocalityDefinition,
    FinancialLocalityScenario, FinancialLocalityStagedWork, FinancialLocalityTraceIdentity,
    FinancialStructuralMutation, LocalityScaleTuple, LocalityScope, LocalitySemanticOutputId,
};

mod candidates;
mod counter_contract;
#[cfg(test)]
mod recovery_assertions;
#[cfg(test)]
mod scenario_tests;
#[cfg(test)]
mod tests;
mod trace;
pub(in crate::tests::domains::fintech) use counter_contract::{
    ExpectedLocalityCounterManifest, ExpectedLocalityCounterRow,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) struct ExpectedGraphBinding {
    pub(in crate::tests::domains::fintech) graph_instance: u64,
    pub(in crate::tests::domains::fintech) seed: u64,
    pub(in crate::tests::domains::fintech) scale: LocalityScaleTuple,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) struct ExpectedBucketKey {
    pub(in crate::tests::domains::fintech) producer: LocalitySemanticOutputId,
    pub(in crate::tests::domains::fintech) aspect: FinancialAspect,
    pub(in crate::tests::domains::fintech) scope: Option<LocalityScope>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) struct ExpectedDependencyDeclaration {
    pub(in crate::tests::domains::fintech) producer: LocalitySemanticOutputId,
    pub(in crate::tests::domains::fintech) consumer: LocalitySemanticOutputId,
    pub(in crate::tests::domains::fintech) aspect: FinancialAspect,
    pub(in crate::tests::domains::fintech) edge_scope: Option<LocalityScope>,
    pub(in crate::tests::domains::fintech) contract_scope: Option<LocalityScope>,
    pub(in crate::tests::domains::fintech) dependency_revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) struct ExpectedCandidateOccurrence {
    pub(in crate::tests::domains::fintech) query_ordinal: u32,
    pub(in crate::tests::domains::fintech) bucket: ExpectedBucketKey,
    pub(in crate::tests::domains::fintech) dependency: ExpectedDependencyDeclaration,
    pub(in crate::tests::domains::fintech) output_commit_ordinal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) struct ExpectedDependencyCause {
    pub(in crate::tests::domains::fintech) graph: ExpectedGraphBinding,
    pub(in crate::tests::domains::fintech) consumer: LocalitySemanticOutputId,
    pub(in crate::tests::domains::fintech) dependency_revision: u64,
    pub(in crate::tests::domains::fintech) producer: LocalitySemanticOutputId,
    pub(in crate::tests::domains::fintech) aspect: FinancialAspect,
    pub(in crate::tests::domains::fintech) edge_scope: Option<LocalityScope>,
    pub(in crate::tests::domains::fintech) cached_version: u64,
    pub(in crate::tests::domains::fintech) output_commit_ordinal: u64,
    pub(in crate::tests::domains::fintech) committed_version: u64,
    pub(in crate::tests::domains::fintech) changed_scopes: Vec<LocalityScope>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) enum ExpectedSealedOriginBinding {
    SourceRecompute {
        admission_generation: u64,
    },
    DependencyCommit {
        cause_set_generation: u64,
        producer_commit_ordinals: Vec<u64>,
    },
    StructuralRecompute {
        structural_generation: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) struct ExpectedWorkIdentity {
    pub(in crate::tests::domains::fintech) graph: ExpectedGraphBinding,
    pub(in crate::tests::domains::fintech) target: LocalitySemanticOutputId,
    pub(in crate::tests::domains::fintech) dependency_revision: u64,
    pub(in crate::tests::domains::fintech) readiness_epoch: u64,
    pub(in crate::tests::domains::fintech) stage_order: u32,
}

pub(in crate::tests::domains::fintech) type ExpectedCanonicalWork =
    BTreeMap<ExpectedWorkIdentity, BTreeSet<ExpectedSealedOriginBinding>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) enum ExpectedActionCheckpointKind {
    PreRewireStaged(FinancialLocalityStagedWork),
    TopologyAccepted(FinancialStructuralMutation),
    StaleWorkDenied {
        stale: FinancialLocalityStagedWork,
        current_dependency_revision: u64,
    },
    CycleRejected {
        target: LocalitySemanticOutputId,
        attempted_topology_ordinal: u64,
        retained_dependency_revision: u64,
    },
    BranchCaptured,
    CheckpointCaptured,
    DerivedStateDestroyed,
    CausesReadmitted,
    ReadyWorkReconstructed,
    DeterministicRerun,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) struct ExpectedDirectSourceBasis {
    pub(in crate::tests::domains::fintech) graph: ExpectedGraphBinding,
    pub(in crate::tests::domains::fintech) source: LocalitySemanticOutputId,
    pub(in crate::tests::domains::fintech) aspect: FinancialAspect,
    pub(in crate::tests::domains::fintech) scope: Option<LocalityScope>,
    pub(in crate::tests::domains::fintech) admission_generation: u64,
    pub(in crate::tests::domains::fintech) dependency_revision: u64,
    pub(in crate::tests::domains::fintech) runtime_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct ExpectedActionCheckpoint {
    pub(in crate::tests::domains::fintech) action_ordinal: u32,
    pub(in crate::tests::domains::fintech) kind: ExpectedActionCheckpointKind,
    pub(in crate::tests::domains::fintech) canonical_causes: BTreeSet<ExpectedDependencyCause>,
    pub(in crate::tests::domains::fintech) persisted_causes: BTreeSet<ExpectedDependencyCause>,
    pub(in crate::tests::domains::fintech) canonical_work: ExpectedCanonicalWork,
    pub(in crate::tests::domains::fintech) current_source_bases:
        BTreeSet<ExpectedDirectSourceBasis>,
    pub(in crate::tests::domains::fintech) persisted_source_bases:
        BTreeSet<ExpectedDirectSourceBasis>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct FinancialLocalityExpectationManifest {
    scenario: FinancialLocalityScenario,
    action_trace: FinancialLocalityTraceIdentity,
    queried_bucket_keys: BTreeSet<ExpectedBucketKey>,
    candidate_dependencies: Vec<ExpectedCandidateOccurrence>,
    canonical_causes: BTreeSet<ExpectedDependencyCause>,
    source_bases: BTreeSet<ExpectedDirectSourceBasis>,
    canonical_work: ExpectedCanonicalWork,
    executed_work: ExpectedCanonicalWork,
    necessary_evaluations: BTreeSet<LocalitySemanticOutputId>,
    unchanged_output_stops: BTreeSet<LocalitySemanticOutputId>,
    peak_ready_width: u64,
    duplicate_admission_attempts: u64,
    causality_rejections: u64,
    committed_output_ordinals: Vec<u64>,
    counter_manifest: ExpectedLocalityCounterManifest,
    action_checkpoints: Vec<ExpectedActionCheckpoint>,
}

impl FinancialLocalityExpectationManifest {
    pub(in crate::tests::domains::fintech) fn derive(
        definition: &FinancialLocalityDefinition,
        graph_instance: u64,
    ) -> Self {
        Self::derive_for_trace(definition, &definition.action_traces()[0], graph_instance)
    }

    pub(in crate::tests::domains::fintech) fn derive_for_trace(
        definition: &FinancialLocalityDefinition,
        action_trace: &FinancialLocalityActionTrace,
        graph_instance: u64,
    ) -> Self {
        let trace = trace::derive_expected_trace_for(definition, action_trace);
        let candidates = candidates::derive_candidate_manifest(definition, graph_instance, &trace);
        let executed_work = trace.canonical_work(definition, graph_instance);
        let canonical_work = if trace.requires_reconstruction() {
            trace.reconstructed_work(
                definition,
                graph_instance,
                trace.final_readiness_epoch,
                &candidates.canonical_causes,
            )
        } else {
            trace.canonical_work(definition, graph_instance)
        };
        let peak_ready_width = trace.peak_ready_width(definition);
        let counter_manifest =
            ExpectedLocalityCounterManifest::derive(&trace, &candidates, peak_ready_width);
        let source_bases = trace.current_source_bases(definition, graph_instance);
        let action_checkpoints = trace.action_checkpoints(
            definition,
            graph_instance,
            &candidates.canonical_causes,
            &canonical_work,
        );
        let committed_output_ordinals = trace
            .deltas
            .iter()
            .map(|delta| delta.output_commit_ordinal)
            .collect();
        Self {
            scenario: definition.scenario(),
            action_trace: action_trace.identity(),
            queried_bucket_keys: candidates.queried_bucket_keys,
            candidate_dependencies: candidates.candidate_dependencies,
            canonical_causes: candidates.canonical_causes,
            source_bases,
            canonical_work,
            executed_work,
            necessary_evaluations: trace.evaluations,
            unchanged_output_stops: trace.stops,
            peak_ready_width,
            duplicate_admission_attempts: action_trace.retry_count() as u64,
            causality_rejections: candidates.causality_rejections,
            committed_output_ordinals,
            counter_manifest,
            action_checkpoints,
        }
    }

    pub(in crate::tests::domains::fintech) const fn scenario(&self) -> FinancialLocalityScenario {
        self.scenario
    }

    pub(in crate::tests::domains::fintech) const fn action_trace(
        &self,
    ) -> FinancialLocalityTraceIdentity {
        self.action_trace
    }

    pub(in crate::tests::domains::fintech) fn queried_bucket_keys(
        &self,
    ) -> &BTreeSet<ExpectedBucketKey> {
        &self.queried_bucket_keys
    }

    pub(in crate::tests::domains::fintech) fn candidate_dependencies(
        &self,
    ) -> &[ExpectedCandidateOccurrence] {
        &self.candidate_dependencies
    }

    pub(in crate::tests::domains::fintech) fn canonical_causes(
        &self,
    ) -> &BTreeSet<ExpectedDependencyCause> {
        &self.canonical_causes
    }

    pub(in crate::tests::domains::fintech) fn canonical_work(&self) -> &ExpectedCanonicalWork {
        &self.canonical_work
    }

    pub(in crate::tests::domains::fintech) fn executed_work(&self) -> &ExpectedCanonicalWork {
        &self.executed_work
    }

    pub(in crate::tests::domains::fintech) fn source_bases(
        &self,
    ) -> &BTreeSet<ExpectedDirectSourceBasis> {
        &self.source_bases
    }

    pub(in crate::tests::domains::fintech) fn necessary_evaluations(
        &self,
    ) -> &BTreeSet<LocalitySemanticOutputId> {
        &self.necessary_evaluations
    }

    pub(in crate::tests::domains::fintech) fn unchanged_output_stops(
        &self,
    ) -> &BTreeSet<LocalitySemanticOutputId> {
        &self.unchanged_output_stops
    }

    pub(in crate::tests::domains::fintech) const fn peak_ready_width(&self) -> u64 {
        self.peak_ready_width
    }

    pub(in crate::tests::domains::fintech) const fn duplicate_admission_attempts(&self) -> u64 {
        self.duplicate_admission_attempts
    }

    pub(in crate::tests::domains::fintech) const fn counter_manifest(
        &self,
    ) -> &ExpectedLocalityCounterManifest {
        &self.counter_manifest
    }

    pub(in crate::tests::domains::fintech) const fn causality_rejections(&self) -> u64 {
        self.causality_rejections
    }

    pub(in crate::tests::domains::fintech) fn action_checkpoints(
        &self,
    ) -> &[ExpectedActionCheckpoint] {
        &self.action_checkpoints
    }

    pub(in crate::tests::domains::fintech) fn committed_output_ordinals(&self) -> &[u64] {
        &self.committed_output_ordinals
    }
}
