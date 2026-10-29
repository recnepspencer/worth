pub(crate) use {std::path::Path};

pub(crate) use {crate::phase_eight_process::history::schedule::create_mutation_crash_operation_program_with_operation_count, crate::history::schedule::in_flight_mutation::MutationCrashWorkload};
pub(crate) use {crate::phase_eight_process::history::process::lifecycle::launch, crate::phase_eight_process::history::process::lifecycle::launch_root, crate::phase_eight_process::history::process::lifecycle::KilledProductionWriter, crate::phase_eight_process::history::process::lifecycle::MutationCrashLaunch, crate::phase_eight_process::history::process::lifecycle::WriterLaunch};

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
        crate::phase_eight_process::history::schedule::OPERATION_COUNT,
    )
}

pub(crate) fn launch_multi_page_rewrite_root(
    parent: &Path,
    schedule_seed: u64,
    perturbation_seed: u64,
) -> Result<std::path::PathBuf, String> {
    let operation_program = create_mutation_crash_operation_program_with_operation_count(
        parent,
        schedule_seed,
        perturbation_seed,
        crate::history::schedule::in_flight_mutation::MutationCrashWorkload::MultiPageSegmentRewrite,
        crate::phase_eight_process::history::schedule::OPERATION_COUNT,
    )?;
    launch_root(WriterLaunch {
        root: parent.join("mutation-crash-writer-root"),
        stage: format!("namespace-synchronization-complete@{schedule_seed}:{perturbation_seed}"),
        mutation_crash: Some(crate::phase_eight_process::history::process::lifecycle::MutationCrashLaunch::new(
            "after-wal-durability",
            crate::history::schedule::in_flight_mutation::MutationCrashWorkload::MultiPageSegmentRewrite,
        )),
        operation_program,
        start: parent.join("mutation-crash-writer-start"),
        reached: parent.join("mutation-crash-writer-reached"),
        durable_before_ack: false,
        allow_unresolved_current_record: true,
    })
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
        mutation_crash: Some(crate::phase_eight_process::history::process::lifecycle::MutationCrashLaunch::new(stage, workload)),
        operation_program,
        start: parent.join("mutation-crash-writer-start"),
        reached: parent.join("mutation-crash-writer-reached"),
        durable_before_ack: false,
        allow_unresolved_current_record: true,
    })
}