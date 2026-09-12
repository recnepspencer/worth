use super::{
    baseline_capabilities, bounded_contract, monotonic_contract, positive_contract,
    CounterContractKind, FixtureClassKind, ObserverKind, OracleFamilyKind, PhysicalCounterContract,
    PhysicalDriverKind, RequiredCounterContractSet, RequiredFixtureClassSet, RequiredObserverSet,
    RequiredOracleFamilySet, RequiredPhysicalDriverSet, RequiredSimulationPlanShape,
};

pub(super) fn recovery_shape(actor_step_count: u64) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: super::RequiredActorSet::from_actors([]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::FreshRuntimeRecovery,
        ]),
        observers: RequiredObserverSet::from_observers([ObserverKind::RecoveryOutcomeObserver]),
        oracle_families: RequiredOracleFamilySet::from_oracles([
            OracleFamilyKind::TranscriptReplayEvidence,
            OracleFamilyKind::RecoveryDogfood,
        ]),
        counter_contracts: RequiredCounterContractSet::from_contracts([
            PhysicalCounterContract::exact(CounterContractKind::ActorStepExact, actor_step_count),
            PhysicalCounterContract::exact(CounterContractKind::ReplayIdentityExact, 1),
            PhysicalCounterContract::profile_scoped(CounterContractKind::ProfileResourceEnvelope),
            bounded_contract(CounterContractKind::AllocationBytes, 64 * 1024),
            bounded_contract(CounterContractKind::ReplayedPages, 128),
            monotonic_contract(CounterContractKind::Retries),
        ]),
        fixture_classes: RequiredFixtureClassSet::from_fixture_classes([
            FixtureClassKind::AspectNativeBoundaryFact,
            FixtureClassKind::RecoveryArtifacts,
        ]),
    }
}

pub(super) fn physical_isolation_checkpoint_publication_crash_replay_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: super::RequiredActorSet::from_actors([]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
            PhysicalDriverKind::FreshRuntimeRecovery,
        ]),
        observers: RequiredObserverSet::from_observers([
            ObserverKind::IndependentPhysicalTrace,
            ObserverKind::RecoveryOutcomeObserver,
        ]),
        oracle_families: RequiredOracleFamilySet::from_oracles([
            OracleFamilyKind::TranscriptReplayEvidence,
            OracleFamilyKind::PhysicalIsolationReadinessShape,
            OracleFamilyKind::RecoveryDogfood,
        ]),
        counter_contracts: RequiredCounterContractSet::from_contracts([
            PhysicalCounterContract::exact(CounterContractKind::ActorStepExact, actor_step_count),
            PhysicalCounterContract::exact(CounterContractKind::ReplayIdentityExact, 1),
            PhysicalCounterContract::profile_scoped(CounterContractKind::ProfileResourceEnvelope),
            bounded_contract(CounterContractKind::AllocationBytes, 64 * 1024),
            bounded_contract(CounterContractKind::PagePins, 8),
            bounded_contract(CounterContractKind::IoQueueDepth, 4),
            monotonic_contract(CounterContractKind::LatchWaits),
            monotonic_contract(CounterContractKind::EpochRetries),
            positive_contract(CounterContractKind::ProtectedReferences),
            positive_contract(CounterContractKind::CompactionCandidateRanges),
            positive_contract(CounterContractKind::CopiedPages),
            monotonic_contract(CounterContractKind::Retries),
        ]),
        fixture_classes: RequiredFixtureClassSet::from_fixture_classes([
            FixtureClassKind::AspectNativeBoundaryFact,
            FixtureClassKind::RecoveryArtifacts,
        ]),
    }
}
