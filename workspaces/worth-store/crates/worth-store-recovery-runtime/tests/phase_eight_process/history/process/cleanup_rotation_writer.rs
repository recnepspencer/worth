use std::path::Path;

use super::super::schedule::create_cleanup_rotation_operation_program_with_operation_count;
use super::lifecycle::{launch, KilledProductionWriter, WriterLaunch};

pub(crate) fn launch_killed_cleanup_writer_with_operation_count(
    parent: &Path,
    schedule_seed: u64,
    perturbation_seed: u64,
    operation_count: usize,
) -> Result<KilledProductionWriter, String> {
    if schedule_seed == perturbation_seed {
        return Err("schedule and perturbation seeds must be distinct".to_owned());
    }
    let operation_program = create_cleanup_rotation_operation_program_with_operation_count(
        parent,
        schedule_seed,
        perturbation_seed,
        operation_count,
    )?;
    launch(WriterLaunch {
        root: parent.join("cleanup-rotation-writer-root"),
        stage: format!("candidate-publication@{schedule_seed}:{perturbation_seed}"),
        mutation_crash: None,
        operation_program,
        start: parent.join("cleanup-rotation-writer-start"),
        reached: parent.join("cleanup-rotation-writer-reached"),
        durable_before_ack: true,
        allow_unresolved_current_record: true,
    })
}
