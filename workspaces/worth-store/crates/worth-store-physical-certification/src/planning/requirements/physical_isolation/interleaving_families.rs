use crate::PhysicalScenarioActor;

use super::super::super::capabilities::{
    PhysicalSimulationCapability, PhysicalSimulationCapabilitySet,
};
use super::super::super::counter_contracts::{
    CounterContractKind, PhysicalCounterContract, RequiredCounterContractSet,
};
use super::super::{
    FixtureClassKind, ObserverKind, OracleFamilyKind, PhysicalDriverKind, RequiredActorSet,
    RequiredFixtureClassSet, RequiredObserverSet, RequiredOracleFamilySet,
    RequiredPhysicalDriverSet, RequiredSimulationPlanShape,
};

pub(crate) fn physical_isolation_compaction_interlock_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: RequiredActorSet::from_actors([
            PhysicalScenarioActor::foreground_reader("reader"),
            PhysicalScenarioActor::compaction_driver("compactor"),
            PhysicalScenarioActor::maintenance_reclaimer("reclaimer"),
        ]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
            PhysicalDriverKind::MemoryPressureBoundary,
        ]),
        observers: RequiredObserverSet::from_observers([ObserverKind::IndependentPhysicalTrace]),
        oracle_families: physical_isolation_oracle_families(),
        counter_contracts: compaction_counter_contracts(actor_step_count),
        fixture_classes: physical_isolation_fixture_classes(),
    }
}

pub(crate) fn physical_isolation_checkpoint_publication_interlock_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: RequiredActorSet::from_actors([
            PhysicalScenarioActor::foreground_reader("reader"),
            PhysicalScenarioActor::checkpoint_driver("checkpoint"),
            PhysicalScenarioActor::recovery_driver("recovery"),
        ]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
            PhysicalDriverKind::FreshRuntimeRecovery,
        ]),
        observers: RequiredObserverSet::from_observers([ObserverKind::IndependentPhysicalTrace]),
        oracle_families: physical_isolation_oracle_families(),
        counter_contracts: checkpoint_counter_contracts(actor_step_count),
        fixture_classes: physical_isolation_fixture_classes(),
    }
}

pub(crate) fn physical_isolation_reclaim_reachability_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: RequiredActorSet::from_actors([
            PhysicalScenarioActor::foreground_reader("reader"),
            PhysicalScenarioActor::maintenance_reclaimer("reclaimer"),
        ]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
        ]),
        observers: RequiredObserverSet::from_observers([ObserverKind::IndependentPhysicalTrace]),
        oracle_families: physical_isolation_oracle_families(),
        counter_contracts: reclaim_counter_contracts(actor_step_count),
        fixture_classes: physical_isolation_fixture_classes(),
    }
}

pub(crate) fn physical_isolation_tier_movement_stability_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: RequiredActorSet::from_actors([
            PhysicalScenarioActor::foreground_reader("reader"),
            PhysicalScenarioActor::maintenance_reclaimer("tier-movement"),
        ]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
        ]),
        observers: RequiredObserverSet::from_observers([ObserverKind::IndependentPhysicalTrace]),
        oracle_families: physical_isolation_oracle_families(),
        counter_contracts: stability_counter_contracts(actor_step_count),
        fixture_classes: physical_isolation_fixture_classes(),
    }
}

pub(crate) fn physical_isolation_future_chunk_stability_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: RequiredActorSet::from_actors([
            PhysicalScenarioActor::foreground_reader("reader"),
            PhysicalScenarioActor::future_extension_slot("future-chunk"),
        ]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
        ]),
        observers: RequiredObserverSet::from_observers([ObserverKind::IndependentPhysicalTrace]),
        oracle_families: physical_isolation_oracle_families(),
        counter_contracts: stability_counter_contracts(actor_step_count),
        fixture_classes: physical_isolation_fixture_classes(),
    }
}

pub(crate) fn physical_isolation_restart_during_cutover_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: RequiredActorSet::from_actors([
            PhysicalScenarioActor::foreground_reader("reader"),
            PhysicalScenarioActor::foreground_writer("writer"),
            PhysicalScenarioActor::recovery_driver("recovery"),
        ]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
            PhysicalDriverKind::FreshRuntimeRecovery,
        ]),
        observers: RequiredObserverSet::from_observers([ObserverKind::IndependentPhysicalTrace]),
        oracle_families: physical_isolation_oracle_families(),
        counter_contracts: restart_counter_contracts(actor_step_count),
        fixture_classes: physical_isolation_fixture_classes(),
    }
}

fn baseline_capabilities() -> PhysicalSimulationCapabilitySet {
    PhysicalSimulationCapabilitySet::from_capabilities([
        PhysicalSimulationCapability::ProductionBoundaryDriver,
        PhysicalSimulationCapability::IndependentObserver,
        PhysicalSimulationCapability::CertificationOracleFamily,
        PhysicalSimulationCapability::CounterContracts,
        PhysicalSimulationCapability::FixtureClassAdmission,
        PhysicalSimulationCapability::EvidencePolicy,
        PhysicalSimulationCapability::ForbiddenShortcutDenial,
    ])
}

fn physical_isolation_oracle_families() -> RequiredOracleFamilySet {
    RequiredOracleFamilySet::from_oracles([
        OracleFamilyKind::TranscriptReplayEvidence,
        OracleFamilyKind::PhysicalIsolationInterleaving,
    ])
}

fn compaction_counter_contracts(actor_step_count: u64) -> RequiredCounterContractSet {
    let mut contracts = base_physical_isolation_counter_contracts(actor_step_count);
    contracts.extend([
        PhysicalCounterContract::exact(
            CounterContractKind::CompactionPublicationPlanCompletions,
            1,
        ),
        positive_contract(CounterContractKind::BlockedReclaimAttempts),
        positive_contract(CounterContractKind::CompactionCandidateRanges),
        positive_contract(CounterContractKind::CopiedPages),
    ]);
    RequiredCounterContractSet::from_contracts(contracts)
}

fn checkpoint_counter_contracts(actor_step_count: u64) -> RequiredCounterContractSet {
    RequiredCounterContractSet::from_contracts(base_physical_isolation_counter_contracts(
        actor_step_count,
    ))
}

fn reclaim_counter_contracts(actor_step_count: u64) -> RequiredCounterContractSet {
    let mut contracts = base_physical_isolation_counter_contracts(actor_step_count);
    contracts.extend([
        positive_contract(CounterContractKind::BlockedReclaimAttempts),
        positive_contract(CounterContractKind::CompactionCandidateRanges),
    ]);
    RequiredCounterContractSet::from_contracts(contracts)
}

fn stability_counter_contracts(actor_step_count: u64) -> RequiredCounterContractSet {
    let mut contracts = base_physical_isolation_counter_contracts(actor_step_count);
    contracts.push(monotonic_contract(
        CounterContractKind::FutureS5SpecificCounters,
    ));
    RequiredCounterContractSet::from_contracts(contracts)
}

fn restart_counter_contracts(actor_step_count: u64) -> RequiredCounterContractSet {
    let mut contracts = checkpoint_counter_contracts(actor_step_count)
        .iter()
        .copied()
        .collect::<Vec<_>>();
    contracts.push(monotonic_contract(CounterContractKind::Retries));
    RequiredCounterContractSet::from_contracts(contracts)
}

fn base_physical_isolation_counter_contracts(
    actor_step_count: u64,
) -> Vec<PhysicalCounterContract> {
    vec![
        PhysicalCounterContract::exact(CounterContractKind::ActorStepExact, actor_step_count),
        PhysicalCounterContract::exact(CounterContractKind::ReplayIdentityExact, 1),
        PhysicalCounterContract::profile_scoped(CounterContractKind::ProfileResourceEnvelope),
        bounded_contract(CounterContractKind::AllocationBytes, 64 * 1024),
        bounded_contract(CounterContractKind::ResidentBytes, 64 * 1024),
        bounded_contract(CounterContractKind::PagePins, 8),
        monotonic_contract(CounterContractKind::LatchWaits),
        monotonic_contract(CounterContractKind::EpochRetries),
        positive_contract(CounterContractKind::ProtectedReferences),
    ]
}

fn physical_isolation_fixture_classes() -> RequiredFixtureClassSet {
    RequiredFixtureClassSet::from_fixture_classes([FixtureClassKind::AspectNativeBoundaryFact])
}

fn bounded_contract(kind: CounterContractKind, maximum: u64) -> PhysicalCounterContract {
    PhysicalCounterContract::bounded(kind, maximum)
        .expect("static bounded counter contract is valid")
}

fn positive_contract(kind: CounterContractKind) -> PhysicalCounterContract {
    PhysicalCounterContract::positive(kind).expect("static positive counter contract is valid")
}

fn monotonic_contract(kind: CounterContractKind) -> PhysicalCounterContract {
    PhysicalCounterContract::monotonic(kind).expect("static monotonic counter contract is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_compaction_completion_is_not_a_checkpoint_or_restart_requirement() {
        let kind = CounterContractKind::CompactionPublicationPlanCompletions;
        assert!(compaction_counter_contracts(3)
            .iter()
            .any(|contract| contract.kind() == kind));
        for contracts in [
            checkpoint_counter_contracts(3),
            restart_counter_contracts(3),
        ] {
            assert!(!contracts.iter().any(|contract| contract.kind() == kind));
        }
    }
}
