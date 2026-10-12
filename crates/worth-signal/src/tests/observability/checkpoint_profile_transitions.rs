use crate::facade::{DiagnosticsAvailability, SignalObservationCompletion};
use crate::tests::domains::fintech::{
    compile_financial_locality_world_with_policy, restore_lifecycle_definition,
    CompiledFinancialWorld, FinancialWorldDefinition,
};
use crate::tests::performance_profiles::throughput_definition::{
    profiles, PerformanceProfile, PERFORMANCE_SEED,
};

#[test]
fn restore_lifecycle_survives_every_installed_profile() {
    let definition = restore_lifecycle_definition(PERFORMANCE_SEED);
    for profile in profiles() {
        let mut world =
            compile_financial_locality_world_with_policy(definition.clone(), profile.policy)
                .expect("restore world compiles under the installed profile");
        world
            .certify_restore_lifecycle_for_performance()
            .expect("installed profile preserves branch and restore operational truth");
        world
            .locality_operational_digest_without_observation_work()
            .expect("branch and restore operational digest derives");
        if !profile.expects_optional_observation() {
            assert!(
                world
                    .locality_optional_observation_inventory()
                    .is_idle_zero(),
                "{} restore lifecycle must not retain optional observation",
                profile.name
            );
        }
    }
}

#[test]
fn named_restore_transitions_preserve_truth_and_typed_absence() {
    let definition = restore_lifecycle_definition(PERFORMANCE_SEED);
    let idle = profile("throughput_idle");
    let forensic = profile("introspective");
    idle_to_idle(&definition, idle);
    forensic_to_idle(&definition, forensic, idle);
    idle_to_forensic(&definition, idle, forensic);
    session_interrupted_by_checkpoint(&definition, idle);
}

fn assert_destination_policy_unchanged(
    world: &CompiledFinancialWorld,
    destination: PerformanceProfile,
) {
    assert_eq!(
        world.runtime_policy().execution_objective,
        destination.policy.execution_objective
    );
    assert_eq!(
        world.runtime_policy().observation_activation,
        destination.policy.observation_activation
    );
}

fn profile(name: &'static str) -> PerformanceProfile {
    profiles()
        .into_iter()
        .find(|profile| profile.name == name)
        .unwrap_or_else(|| panic!("{name} profile"))
}

fn idle_to_idle(definition: &FinancialWorldDefinition, idle: PerformanceProfile) {
    let mut world = compile_financial_locality_world_with_policy(definition.clone(), idle.policy)
        .expect("idle restore world compiles");
    let before = world
        .locality_committed_value_list()
        .expect("idle committed values");
    let snapshot = world
        .locality_capture_runtime_snapshot()
        .expect("idle snapshot");
    world
        .locality_restore_runtime_snapshot(&snapshot)
        .expect("idle restore");
    let after = world
        .locality_committed_value_list()
        .expect("idle committed values after restore");
    assert_eq!(before, after);
    assert!(world
        .locality_optional_observation_inventory()
        .is_idle_zero());
    assert_observation_not_activated(&world);
}

fn forensic_to_idle(
    definition: &FinancialWorldDefinition,
    forensic: PerformanceProfile,
    idle: PerformanceProfile,
) {
    let mut rich =
        compile_financial_locality_world_with_policy(definition.clone(), forensic.policy)
            .expect("forensic restore world compiles");
    rich.certify_restore_lifecycle_for_performance()
        .expect("forensic restore lifecycle");
    let rich_values = rich
        .locality_committed_value_list()
        .expect("forensic committed values");
    let snapshot = rich
        .locality_capture_runtime_snapshot()
        .expect("forensic snapshot");

    let mut idle_world =
        compile_financial_locality_world_with_policy(definition.clone(), idle.policy)
            .expect("idle target compiles");
    idle_world
        .locality_restore_runtime_snapshot_keeping_destination_policy(&snapshot)
        .expect("forensic snapshot restores into idle runtime");
    assert_destination_policy_unchanged(&idle_world, idle);
    let idle_values = idle_world
        .locality_committed_value_list()
        .expect("idle committed values after forensic restore");
    assert_eq!(rich_values, idle_values);
    let after = idle_world.locality_optional_observation_inventory();
    assert_eq!(
        after.lineage_records,
        snapshot.diagnostics.lineage_records.len(),
        "restore must not grow lineage past the captured snapshot"
    );
    assert_eq!(
        after.replay_events,
        snapshot.diagnostics.replay_frames.len(),
        "restore must not grow replay past the captured snapshot"
    );
    assert_observation_not_activated(&idle_world);
}

fn idle_to_forensic(
    definition: &FinancialWorldDefinition,
    idle: PerformanceProfile,
    forensic: PerformanceProfile,
) {
    let mut idle_world =
        compile_financial_locality_world_with_policy(definition.clone(), idle.policy)
            .expect("idle source compiles");
    idle_world
        .certify_restore_lifecycle_for_performance()
        .expect("idle restore lifecycle");
    let idle_values = idle_world
        .locality_committed_value_list()
        .expect("idle committed values");
    let snapshot = idle_world
        .locality_capture_runtime_snapshot()
        .expect("idle snapshot");

    let mut rich =
        compile_financial_locality_world_with_policy(definition.clone(), forensic.policy)
            .expect("forensic target compiles");
    let dest_before = rich.locality_optional_observation_inventory();
    rich.locality_restore_runtime_snapshot_keeping_destination_policy(&snapshot)
        .expect("idle snapshot restores into forensic runtime");
    assert_destination_policy_unchanged(&rich, forensic);
    let imported = rich.locality_optional_observation_inventory();
    let snapshot_sequences = snapshot
        .diagnostics
        .lineage_records
        .iter()
        .map(|record| record.sequence)
        .collect::<Vec<_>>();
    for sequence in dest_before.lineage_sequences {
        if !snapshot_sequences.contains(&sequence) {
            assert!(
                !imported.lineage_sequences.contains(&sequence),
                "destination compile lineage {sequence} must not survive idle snapshot restore"
            );
        }
    }
    let rich_values = rich
        .locality_committed_value_list()
        .expect("forensic committed values after idle restore");
    assert_eq!(idle_values, rich_values);
    let node = rich
        .locality_first_dependent_node()
        .expect("restore world has a dependent");
    let (artifact, availability) = rich
        .locality_materialize_explanation(node)
        .expect("explanation materialization is typed");
    assert_eq!(
        availability,
        DiagnosticsAvailability::ReconstructedAvailable,
        "richer post-restore capture may reconstruct current evidence"
    );
    assert!(
        artifact.is_some(),
        "richer post-restore profile may capture new current evidence"
    );
}

fn session_interrupted_by_checkpoint(
    definition: &FinancialWorldDefinition,
    idle: PerformanceProfile,
) {
    let mut world = compile_financial_locality_world_with_policy(definition.clone(), idle.policy)
        .expect("idle interrupt world compiles");
    let session = world
        .locality_begin_runtime_observation()
        .expect("idle OnDemand session admits when requested");
    world
        .locality_capture_runtime_snapshot()
        .expect("checkpoint interrupts the session");
    assert_eq!(
        world.locality_last_observation_completion(),
        Some(SignalObservationCompletion::InterruptedByBoundary)
    );
    assert!(world.locality_finish_runtime_observation(&session).is_err());
}

fn assert_observation_not_activated(world: &CompiledFinancialWorld) {
    let node = world
        .locality_first_dependent_node()
        .expect("restore world has a dependent");
    let (_, availability) = world
        .locality_materialize_explanation(node)
        .expect("explanation materialization is typed");
    assert_eq!(
        availability,
        DiagnosticsAvailability::ObservationNotActivated
    );
}
