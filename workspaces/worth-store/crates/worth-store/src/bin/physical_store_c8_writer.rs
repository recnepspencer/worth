#[path = "physical_store_c8_writer/admission.rs"]
mod admission;
#[path = "physical_store_c8_writer/checkpoint.rs"]
mod checkpoint;
#[path = "physical_store_c8_writer/configuration.rs"]
mod configuration;
#[path = "physical_store_c8_writer/dirty_mutation.rs"]
mod dirty_mutation;
#[path = "physical_store_c8_writer/durability_profile.rs"]
mod durability_profile;
#[path = "physical_store_c8_writer/durable_before_ack.rs"]
mod durable_before_ack;
#[path = "physical_store_c8_writer/history.rs"]
mod history;
#[path = "physical_store_c8_writer/initialization.rs"]
mod initialization;
#[path = "physical_store_c8_writer/lifecycle.rs"]
mod lifecycle;
#[path = "physical_store_c8_writer/markers.rs"]
mod markers;
#[path = "physical_store_c8_writer/mutation_crash.rs"]
mod mutation_crash;
#[path = "physical_store_c8_writer/mutation_material.rs"]
mod mutation_material;
#[path = "physical_store_c8_writer/mutation_submission.rs"]
mod mutation_submission;
#[path = "physical_store_c8_writer/no_effect.rs"]
mod no_effect;
#[path = "physical_store_c8_writer/operation_program.rs"]
mod operation_program;

use std::ffi::OsString;
use std::path::PathBuf;

use durability_profile::WriterDurabilityProfile;
use lifecycle::CheckpointStage;

struct Invocation {
    root: PathBuf,
    operation_program: PathBuf,
    start_marker: PathBuf,
    reached_marker: PathBuf,
    stage: CheckpointStageWithSeed,
    mutation_crash: Option<mutation_crash::MutationCrashInvocation>,
    durable_before_ack: bool,
    writer_durability_profile: WriterDurabilityProfile,
}

fn main() {
    if let Err(failure) = run(parse(std::env::args_os().skip(1))) {
        eprintln!("C8_PRODUCTION_WRITER_DENIED {failure}");
        std::process::exit(2);
    }
}

fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Invocation, String> {
    let mut arguments = arguments.into_iter();
    let mut root = None;
    let mut operation_program = None;
    let mut start_marker = None;
    let mut reached_marker = None;
    let mut stage = None;
    let mut mutation_crash_stage = None;
    let mut mutation_crash_workload = None;
    let mut durable_before_ack = false;
    let mut writer_durability_profile = None;
    while let Some(option) = arguments.next() {
        let option = option
            .into_string()
            .map_err(|_| "C8 writer option was not Unicode".to_owned())?;
        if option == "--durable-before-ack" {
            durable_before_ack = true;
            continue;
        }
        let value = arguments
            .next()
            .ok_or_else(|| format!("{option} requires a value"))?;
        if option == "--writer-durability-profile" {
            let value = value
                .into_string()
                .map_err(|_| "C8 writer durability profile was not Unicode".to_owned())?;
            writer_durability_profile = Some(WriterDurabilityProfile::parse(&value)?);
            continue;
        }
        if option == "--mutation-crash-stage" {
            let value = value
                .into_string()
                .map_err(|_| "C8 mutation crash stage was not Unicode".to_owned())?;
            mutation_crash_stage = Some(value);
            continue;
        }
        if option == "--mutation-crash-workload" {
            let value = value
                .into_string()
                .map_err(|_| "C8 mutation crash workload was not Unicode".to_owned())?;
            mutation_crash_workload = Some(value);
            continue;
        }
        let value = PathBuf::from(value);
        match option.as_str() {
            "--root" => root = Some(value),
            "--operation-program" => operation_program = Some(value),
            "--start-marker" => start_marker = Some(value),
            "--reached-marker" => reached_marker = Some(value),
            "--checkpoint-stage" => {
                let value = value
                    .into_os_string()
                    .into_string()
                    .map_err(|_| "C8 checkpoint stage was not Unicode".to_owned())?;
                stage = Some(value);
            }
            _ => return Err(format!("unknown C8 writer option `{option}`")),
        }
    }
    let (stage, schedule_seed, perturbation_seed) =
        CheckpointStage::parse(&stage.ok_or_else(|| "--checkpoint-stage is required".to_owned())?)?;
    Ok(Invocation {
        root: root.ok_or_else(|| "--root is required".to_owned())?,
        operation_program: operation_program
            .ok_or_else(|| "--operation-program is required".to_owned())?,
        start_marker: start_marker.ok_or_else(|| "--start-marker is required".to_owned())?,
        reached_marker: reached_marker.ok_or_else(|| "--reached-marker is required".to_owned())?,
        stage: CheckpointStageWithSeed {
            stage,
            schedule_seed,
            perturbation_seed,
        },
        mutation_crash: mutation_crash::admit(mutation_crash_stage, mutation_crash_workload)?,
        durable_before_ack,
        writer_durability_profile: writer_durability_profile
            .ok_or_else(|| "--writer-durability-profile is required".to_owned())?,
    })
}

struct CheckpointStageWithSeed {
    stage: CheckpointStage,
    schedule_seed: u64,
    perturbation_seed: u64,
}

impl CheckpointStageWithSeed {
    fn completes_after_arrival(&self) -> bool {
        matches!(
            self.stage,
            CheckpointStage::NamespaceSynchronizationComplete
        )
    }
}

fn run(invocation: Result<Invocation, String>) -> Result<(), String> {
    let invocation = invocation?;
    let writer = initialization::initialize(&invocation)?;
    history::seed_initial_history(&writer, &invocation.stage)?;
    no_effect::cancel_before_effect(
        &writer.serving,
        writer.placement,
        invocation.stage.perturbation_seed,
    )?;
    checkpoint::complete(
        &writer.serving,
        invocation.stage.perturbation_seed ^ 0xC8_00_00_03,
    )?;

    if let Some(crash) = invocation.mutation_crash {
        return mutation_crash::hold_for_process_death(&writer, crash, &invocation);
    }

    if invocation.durable_before_ack {
        return durable_before_ack::hold_at_terminal_seam(
            &writer.serving,
            writer.placement,
            &invocation,
        );
    }

    let _dirty_mutation = dirty_mutation::prepare_for_checkpoint(
        &writer.serving,
        writer.format,
        writer.placement,
        invocation.stage.perturbation_seed,
        &invocation,
    )?;
    checkpoint::hold_at_stage(&writer.serving, &invocation)
}
