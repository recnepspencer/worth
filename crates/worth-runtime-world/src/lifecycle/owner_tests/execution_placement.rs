use std::{num::NonZeroUsize, sync::Arc};

use worth_execution::{ConstructionDenial, ExecutionAuthority, ExecutionAuthorityConfig};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy,
};

use super::*;
use crate::branch::reference_test_fixture::{real_fixture, RealReferenceFixture};
use crate::lifecycle::{
    RuntimeWorldBuildDenial, RuntimeWorldExecutionPlacement, RuntimeWorldOwner,
    RuntimeWorldOwnerBuilder, RuntimeWorldOwnerInputs,
};

fn policy() -> ExecutionRequestPolicy {
    policy_of(DeterminismContract::CanonicalBitwise, 2, 2048)
}

fn policy_of(
    determinism: DeterminismContract,
    workers: usize,
    memory: u64,
) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        determinism,
        ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), memory, 1_000),
    )
}

type Ready<D, I, E, Ctx, T> = RuntimeWorldOwnerBuilder<
    crate::facade::RuntimeWorldCorrespondencePort,
    worth_relational::facade::branch::RelationalOwnerServicePorts,
    worth_signal::facade::branch::SignalOwnerServicePorts<D, I, E, Ctx, T>,
    worth_signal::facade::branch::SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>,
    RuntimeWorldBudgets,
    RuntimeWorldClock,
>;

fn ready(inputs: RuntimeWorldOwnerInputs<(), (), (), (), ()>) -> Ready<(), (), (), (), ()> {
    RuntimeWorldOwner::builder()
        .with_relational_services(inputs.relational)
        .with_signal_services(inputs.signal)
        .with_signal_definition_publication(inputs.signal_definition_publication)
        .with_bridge_correspondence(inputs.bridge)
        .with_budgets(inputs.budgets)
        .with_clock(inputs.clock)
}

/// The fixture owns the runtimes the World's services reach, so it lives as
/// long as the World built from it.
fn builder() -> (RealReferenceFixture, Ready<(), (), (), (), ()>) {
    let mut fixture = real_fixture(4, 4);
    let inputs = fixture.owner_inputs(
        bootstrap_budgets(),
        RuntimeWorldClock::from_source(FixedClock),
    );
    (fixture, ready(inputs))
}

#[test]
fn a_world_runs_unbounded_without_a_policy_and_serial_with_one() {
    let (_fixture, world) = builder();
    let unbounded = world.build().unwrap();
    assert!(matches!(
        unbounded.execution_placement(),
        RuntimeWorldExecutionPlacement::Unbounded
    ));
    assert!(unbounded.execution_authority().is_none());

    let (_fixture, world) = builder();
    let serial = world.with_execution_policy(policy()).build().unwrap();
    assert!(matches!(
        serial.execution_placement(),
        RuntimeWorldExecutionPlacement::Serial(installed) if installed == policy()
    ));
    assert!(serial.execution_authority().is_none());
}

#[test]
fn an_authority_needs_a_policy_and_a_world_leases_under_both() {
    let config = ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: 4096,
    };
    let authority = Arc::new(ExecutionAuthority::try_construct(config).unwrap());
    let (_fixture, world) = builder();
    assert!(matches!(
        world
            .with_execution_authority(Arc::clone(&authority))
            .build(),
        Err(RuntimeWorldBuildDenial::ExecutionAuthorityWithoutPolicy)
    ));
    // A policy the authority can never lease is refused here, each cause on
    // its own, not by every request it would shape.
    for (refused, denial) in [
        (
            policy_of(DeterminismContract::CanonicalBitwise, 3, 2048),
            RuntimeWorldBuildDenial::ExecutionPolicyWorkersExceedAuthority,
        ),
        (
            policy_of(DeterminismContract::CanonicalBitwise, 2, 4097),
            RuntimeWorldBuildDenial::ExecutionPolicyMemoryExceedsAuthority,
        ),
        (
            policy_of(
                DeterminismContract::ContractEquivalent(EquivalenceContractId::new(1)),
                2,
                2048,
            ),
            RuntimeWorldBuildDenial::ExecutionPolicyContractUnavailable,
        ),
    ] {
        let (_fixture, world) = builder();
        assert_eq!(
            world
                .with_execution_authority(Arc::clone(&authority))
                .with_execution_policy(refused)
                .build()
                .err(),
            Some(denial)
        );
    }
    let (_fixture, world) = builder();
    let owner = world
        .with_execution_authority(Arc::clone(&authority))
        .with_execution_policy(policy())
        .build()
        .unwrap();
    assert!(std::ptr::eq(
        owner.execution_authority().unwrap(),
        authority.as_ref()
    ));
    assert!(matches!(
        owner.execution_placement(),
        RuntimeWorldExecutionPlacement::Leased { authority: installed, policy: leased }
            if std::ptr::eq(installed, authority.as_ref()) && leased == policy()
    ));
    assert!(matches!(
        ExecutionAuthority::try_construct(config),
        Err(ConstructionDenial::AlreadyConstructed)
    ));
}
