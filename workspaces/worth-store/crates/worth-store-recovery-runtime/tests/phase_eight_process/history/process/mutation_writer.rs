use std::path::Path;

use super::super::schedule::{
    create_mutation_crash_operation_program_with_operation_count, MutationCrashWorkload,
};
use super::lifecycle::{launch, KilledProductionWriter, MutationCrashLaunch, WriterLaunch};

pub(crate) fn launch_killed_mutation_writer(
    parent: &Path,
    stage: &'static str,
    workload: MutationCrashWorkload,
    schedule_seed: u64,
    perturbation_seed: u64,
) -> Result<KilledProductionWriter, String> {
    launch_killed_mutation_writer_with_operation_count(
        parent,
        stage,
        workload,
        schedule_seed,
        perturbation_seed,
        super::super::schedule::OPERATION_COUNT,
    )
}

pub(crate) fn launch_killed_mutation_writer_with_operation_count(
    parent: &Path,
    stage: &'static str,
    workload: MutationCrashWorkload,
    schedule_seed: u64,
    perturbation_seed: u64,
    operation_count: usize,
) -> Result<KilledProductionWriter, String> {
    let operation_program = create_mutation_crash_operation_program_with_operation_count(
        parent,
        schedule_seed,
        perturbation_seed,
        workload,
        operation_count,
    )?;
    launch(WriterLaunch {
        root: parent.join("mutation-crash-writer-root"),
        stage: format!("namespace-synchronization-complete@{schedule_seed}:{perturbation_seed}"),
        mutation_crash: Some(MutationCrashLaunch::new(stage, workload)),
        operation_program,
        start: parent.join("mutation-crash-writer-start"),
        reached: parent.join("mutation-crash-writer-reached"),
        durable_before_ack: false,
        allow_unresolved_current_record: true,
    })
}
