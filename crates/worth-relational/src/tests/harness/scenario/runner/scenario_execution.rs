use crate::facade::snapshots::SnapshotHandle;
use crate::tests::harness::fixtures::runtime::build_runtime;
use crate::tests::harness::scenario::operation::ScenarioOperation;
use crate::tests::harness::scenario::seed::DeterministicGenerator;
use crate::tests::support::{
    checkpoint_for_schema_version, create_entity_in_partition, SchemaVersionId,
};

use super::operation_application::apply_operation;
use super::operation_selection::choose_operation;
use super::property_runtime::build_property_runtime;
use super::{
    scenario_branch_main, ActiveRelation, ScenarioTrace, SeededScenarioConfig, SeededScenarioWorld,
};
use crate::tests::harness::fixtures::runtime::RuntimeHarnessMode;

pub(crate) fn run_seeded_scenario(config: SeededScenarioConfig) -> SeededScenarioWorld {
    let mut runtime = build_runtime(config.runtime_mode);
    let mut entities = seed_initial_entities(&mut runtime, "seed");
    let baseline_checkpoint = checkpoint_for_schema_version(
        runtime
            .publication()
            .artifacts()
            .latest_patch()
            .unwrap()
            .position,
        SchemaVersionId(1),
    );
    let mut checkpoints = vec![baseline_checkpoint.clone()];
    let mut snapshots: Vec<SnapshotHandle> = Vec::new();
    let mut relations: Vec<ActiveRelation> = Vec::new();
    let mut branches = vec![scenario_branch_main()];
    let mut generator = DeterministicGenerator::new(config.seed);
    let mut operations = Vec::with_capacity(config.steps);
    let mut name_counter = 0_u64;

    for step in 0..config.steps {
        let operation = choose_operation(
            &mut generator,
            &entities,
            &snapshots,
            &relations,
            &branches,
            &config,
        );
        apply_operation(
            &runtime,
            &mut entities,
            &mut snapshots,
            &mut relations,
            &mut branches,
            &mut generator,
            &mut name_counter,
            config.seed,
            step,
            &operation,
        );
        operations.push(operation);
        run_periodic_authority_work(&mut runtime, &config, step, &mut operations);
        maybe_record_checkpoint(&runtime, &config, &mut checkpoints);
    }
    record_final_checkpoint(&runtime, &mut checkpoints);

    SeededScenarioWorld {
        runtime,
        baseline_checkpoint,
        checkpoints,
        trace: ScenarioTrace {
            seed: config.seed,
            operations,
        },
    }
}

pub(crate) fn run_property_scenario(
    operations: Vec<ScenarioOperation>,
    runtime_mode: RuntimeHarnessMode,
) -> SeededScenarioWorld {
    let mut runtime = build_property_runtime(runtime_mode);
    let mut entities = seed_initial_entities(&mut runtime, "property");
    let baseline_checkpoint = checkpoint_for_schema_version(
        runtime
            .publication()
            .artifacts()
            .latest_patch()
            .unwrap()
            .position,
        SchemaVersionId(1),
    );
    let mut checkpoints = vec![baseline_checkpoint.clone()];
    let mut snapshots: Vec<SnapshotHandle> = Vec::new();
    let mut relations: Vec<ActiveRelation> = Vec::new();
    let mut branches = vec![scenario_branch_main()];
    let mut generator = DeterministicGenerator::new(0xC0FFEE);
    let mut name_counter = 0_u64;

    for (step, operation) in operations.iter().enumerate() {
        apply_operation(
            &runtime,
            &mut entities,
            &mut snapshots,
            &mut relations,
            &mut branches,
            &mut generator,
            &mut name_counter,
            0xC0FFEE,
            step,
            operation,
        );
        // Rebuild from authoritative records after every operation, including
        // retention and rejected mutations. Do not trust the carried digest cache.
        let symbols = runtime.services.symbols.interner_snapshot();
        for branch in &branches {
            let root = runtime
                .history
                .branch_cell(branch)
                .and_then(|cell| cell.root())
                .unwrap();
            assert!(root.is_complete(&symbols),
                "step {step}, {operation:?}, branch {branch:?}: incremental root differs from cold truth");
            root.assert_derived_inventory_matches_cold();
        }
        record_any_new_checkpoint(&runtime, &mut checkpoints);
    }

    SeededScenarioWorld {
        runtime,
        baseline_checkpoint,
        checkpoints,
        trace: ScenarioTrace {
            seed: 0xC0FFEE,
            operations,
        },
    }
}

fn seed_initial_entities(
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    label: &str,
) -> Vec<crate::facade::identity::EntityId> {
    vec![
        create_entity_in_partition(
            runtime,
            &format!("{label}-left"),
            crate::facade::identity::PartitionId(7),
        ),
        create_entity_in_partition(
            runtime,
            &format!("{label}-right"),
            crate::facade::identity::PartitionId(11),
        ),
        create_entity_in_partition(
            runtime,
            &format!("{label}-center"),
            crate::facade::identity::PartitionId(29),
        ),
    ]
}

fn run_periodic_authority_work(
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    config: &SeededScenarioConfig,
    step: usize,
    operations: &mut Vec<ScenarioOperation>,
) {
    if config
        .durable_checkpoint_every
        .map(|interval| (step + 1).is_multiple_of(interval))
        .unwrap_or(false)
    {
        runtime.durability_authority().checkpoint().unwrap();
        operations.push(ScenarioOperation::DurableCheckpoint);
    }

    if config
        .durable_compact_every
        .map(|interval| (step + 1).is_multiple_of(interval))
        .unwrap_or(false)
    {
        let _ = runtime.durability_authority().compact_store().unwrap();
        operations.push(ScenarioOperation::CompactDurableStore);
    }

    if config
        .retention_pass_every
        .map(|interval| (step + 1).is_multiple_of(interval))
        .unwrap_or(false)
    {
        let _ = runtime.retention().run_pass();
        operations.push(ScenarioOperation::RunRetentionPass);
    }
}

fn maybe_record_checkpoint(
    runtime: &crate::facade::runtime::RelationalRuntime,
    config: &SeededScenarioConfig,
    checkpoints: &mut Vec<crate::facade::publication::SubscriberCheckpoint>,
) {
    let latest_position = runtime
        .publication()
        .artifacts()
        .latest_patch()
        .unwrap()
        .position;
    if latest_position.0 > checkpoints.last().unwrap().position().0
        && (latest_position.0 as usize).is_multiple_of(config.checkpoint_stride)
    {
        checkpoints.push(checkpoint_for_schema_version(
            latest_position,
            SchemaVersionId(1),
        ));
    }
}

fn record_any_new_checkpoint(
    runtime: &crate::facade::runtime::RelationalRuntime,
    checkpoints: &mut Vec<crate::facade::publication::SubscriberCheckpoint>,
) {
    let latest_position = runtime
        .publication()
        .artifacts()
        .latest_patch()
        .unwrap()
        .position;
    if latest_position.0 > checkpoints.last().unwrap().position().0 {
        checkpoints.push(checkpoint_for_schema_version(
            latest_position,
            SchemaVersionId(1),
        ));
    }
}

fn record_final_checkpoint(
    runtime: &crate::facade::runtime::RelationalRuntime,
    checkpoints: &mut Vec<crate::facade::publication::SubscriberCheckpoint>,
) {
    let latest_position = runtime
        .publication()
        .artifacts()
        .latest_patch()
        .unwrap()
        .position;
    if checkpoints
        .last()
        .map(|checkpoint| checkpoint.position() != latest_position)
        .unwrap_or(true)
    {
        checkpoints.push(checkpoint_for_schema_version(
            latest_position,
            SchemaVersionId(1),
        ));
    }
}
