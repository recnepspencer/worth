use crate::{
    CounterContractKind, PhysicalCounterContract, PhysicalScenarioActor, RequiredCounterContractSet,
};

use super::super::{
    baseline_capabilities, bounded_contract, monotonic_contract, positive_contract,
    FixtureClassKind, ObserverKind, OracleFamilyKind, PhysicalDriverKind, RequiredActorSet,
    RequiredFixtureClassSet, RequiredObserverSet, RequiredOracleFamilySet,
    RequiredPhysicalDriverSet, RequiredSimulationPlanShape,
};

pub(in crate::planning::requirements) fn physical_isolation_readiness_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: RequiredActorSet::from_actors([PhysicalScenarioActor::recovery_driver("recovery")]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
        ]),
        observers: RequiredObserverSet::from_observers([ObserverKind::IndependentPhysicalTrace]),
        oracle_families: RequiredOracleFamilySet::from_oracles([
            OracleFamilyKind::TranscriptReplayEvidence,
            OracleFamilyKind::PhysicalIsolationReadinessShape,
        ]),
        counter_contracts: RequiredCounterContractSet::from_contracts(
            base_physical_isolation_counter_contracts(actor_step_count),
        ),
        fixture_classes: RequiredFixtureClassSet::from_fixture_classes([
            FixtureClassKind::AspectNativeBoundaryFact,
        ]),
    }
}

pub(in crate::planning::requirements) fn physical_isolation_readiness_with_shortcut_rejection_shape(
    actor_step_count: u64,
) -> RequiredSimulationPlanShape {
    let mut counter_contracts = base_physical_isolation_counter_contracts(actor_step_count);
    counter_contracts.push(PhysicalCounterContract::exact(
        CounterContractKind::ForbiddenShortcutExact,
        0,
    ));
    counter_contracts.push(positive_contract(
        CounterContractKind::BlockedReclaimAttempts,
    ));
    RequiredSimulationPlanShape {
        capabilities: baseline_capabilities(),
        actors: RequiredActorSet::from_actors([]),
        drivers: RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
            PhysicalDriverKind::ShortcutRejectionBoundary,
        ]),
        observers: RequiredObserverSet::from_observers([
            ObserverKind::IndependentPhysicalTrace,
            ObserverKind::ShortcutRejectionObserver,
        ]),
        oracle_families: RequiredOracleFamilySet::from_oracles([
            OracleFamilyKind::TranscriptReplayEvidence,
            OracleFamilyKind::PhysicalIsolationReadinessShape,
            OracleFamilyKind::ForbiddenShortcutRejection,
        ]),
        counter_contracts: RequiredCounterContractSet::from_contracts(counter_contracts),
        fixture_classes: RequiredFixtureClassSet::from_fixture_classes([
            FixtureClassKind::AspectNativeBoundaryFact,
        ]),
    }
}

pub(in crate::planning::requirements) fn physical_isolation_readiness_drivers_for_yieldpoint(
    yieldpoint: &str,
) -> RequiredPhysicalDriverSet {
    match yieldpoint {
        "io-pressure-boundary" => {
            RequiredPhysicalDriverSet::from_drivers([PhysicalDriverKind::IoPressureBoundary])
        }
        _ => RequiredPhysicalDriverSet::from_drivers([
            PhysicalDriverKind::ProductionBoundaryYieldpoint,
        ]),
    }
}

fn base_physical_isolation_counter_contracts(
    actor_step_count: u64,
) -> Vec<PhysicalCounterContract> {
    vec![
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
    ]
}
