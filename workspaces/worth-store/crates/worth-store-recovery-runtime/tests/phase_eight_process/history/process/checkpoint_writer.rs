use std::path::Path;

use super::super::schedule::{
    create_checkpoint_operation_program, create_checkpoint_operation_program_with_operation_count,
};
use super::lifecycle::{launch, KilledProductionWriter, WriterLaunch};

pub(crate) fn launch_killed_production_writer(
    parent: &Path,
    stage: &str,
    schedule_seed: u64,
    perturbation_seed: u64,
) -> Result<KilledProductionWriter, String> {
    launch_killed_production_writer_with_operation_count(
        parent,
        stage,
        schedule_seed,
        perturbation_seed,
        super::super::DEFAULT_OPERATION_COUNT,
    )
}

pub(crate) fn launch_killed_production_writer_with_operation_count(
    parent: &Path,
    stage: &str,
    schedule_seed: u64,
    perturbation_seed: u64,
    operation_count: usize,
) -> Result<KilledProductionWriter, String> {
    if schedule_seed == perturbation_seed {
        return Err("schedule and perturbation seeds must be distinct".to_owned());
    }
    let operation_program = if operation_count == super::super::DEFAULT_OPERATION_COUNT {
        create_checkpoint_operation_program(parent, schedule_seed, perturbation_seed)?
    } else {
        create_checkpoint_operation_program_with_operation_count(
            parent,
            schedule_seed,
            perturbation_seed,
            operation_count,
        )?
    };
    launch(WriterLaunch {
        root: parent.join("production-writer-root"),
        stage: format!("{stage}@{schedule_seed}:{perturbation_seed}"),
        mutation_crash: None,
        operation_program,
        start: parent.join("production-writer-start"),
        reached: parent.join("production-writer-reached"),
        durable_before_ack: false,
        allow_unresolved_current_record: false,
    })
}
