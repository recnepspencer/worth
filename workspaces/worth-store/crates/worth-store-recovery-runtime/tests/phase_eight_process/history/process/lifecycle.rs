use std::path::{Path, PathBuf};
use std::process::Command;

use super::super::super::child_lifecycle::ProcessChildGuard;
use super::super::writer_process::{c8_writer_binary_path, wait_for_marker};
use super::super::{
    ExpectedWriterHistory, MutationCrashWorkload, ParentPhysicalHistory, SubmittedOperationProgram,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KilledProductionWriter {
    pub(crate) root: PathBuf,
    pub(crate) history: ParentPhysicalHistory,
    pub(crate) expected: ExpectedWriterHistory,
    pub(crate) process_id: u32,
    pub(crate) runtime_identity: u64,
}

pub(crate) struct WriterLaunch {
    pub(crate) root: PathBuf,
    pub(crate) stage: String,
    pub(crate) mutation_crash: Option<MutationCrashLaunch>,
    pub(crate) operation_program: SubmittedOperationProgram,
    pub(crate) start: PathBuf,
    pub(crate) reached: PathBuf,
    pub(crate) durable_before_ack: bool,
    pub(crate) allow_unresolved_current_record: bool,
}

pub(crate) struct MutationCrashLaunch {
    stage: &'static str,
    workload: MutationCrashWorkload,
}

impl MutationCrashLaunch {
    pub(crate) const fn new(stage: &'static str, workload: MutationCrashWorkload) -> Self {
        Self { stage, workload }
    }
}

pub(crate) fn launch(mut declaration: WriterLaunch) -> Result<KilledProductionWriter, String> {
    let mut command = Command::new(c8_writer_binary_path());
    command
        .args(["--root"])
        .arg(&declaration.root)
        .args(["--start-marker"])
        .arg(&declaration.start)
        .args(["--reached-marker"])
        .arg(&declaration.reached)
        .args(["--checkpoint-stage"])
        .arg(&declaration.stage)
        .args(["--writer-durability-profile"])
        .arg(
            declaration
                .operation_program
                .writer_profile_selection
                .cli_name(),
        )
        .args(["--operation-program"])
        .arg(&declaration.operation_program.path)
        .args(
            declaration
                .durable_before_ack
                .then_some("--durable-before-ack"),
        );
    if let Some(crash) = declaration.mutation_crash {
        command.args(["--mutation-crash-stage", crash.stage]);
        command.args(["--mutation-crash-workload", crash.workload.cli_name()]);
    }
    let mut child = ProcessChildGuard::new(
        command
            .spawn()
            .map_err(|error| format!("spawn {}: {error}", declaration.stage))?,
    );
    let process_id = child.id();
    wait_for_marker(
        &mut child,
        &declaration.start.with_extension("ready"),
        "writer ready",
    )?;
    let runtime_identity = read_runtime_identity(&declaration.start)?;
    declaration
        .operation_program
        .expected
        .bind_persisted_operation_identities(&declaration.root)?;
    std::fs::write(&declaration.start, b"release")
        .map_err(|error| format!("release {}: {error}", declaration.stage))?;
    wait_for_marker(&mut child, &declaration.reached, "writer effect")?;
    let status = child
        .kill_and_wait()
        .map_err(|error| format!("wait for {}: {error}", declaration.stage))?;
    if status.success() {
        return Err(format!(
            "{} unexpectedly exited successfully",
            declaration.stage
        ));
    }
    let history = if declaration.allow_unresolved_current_record {
        ParentPhysicalHistory::capture_with_unresolved_record(
            &declaration.root,
            &declaration.operation_program.expected,
        )?
    } else {
        ParentPhysicalHistory::capture(&declaration.root, &declaration.operation_program.expected)?
    };
    Ok(KilledProductionWriter {
        root: declaration.root,
        history,
        expected: declaration.operation_program.expected,
        process_id,
        runtime_identity,
    })
}

fn read_runtime_identity(start: &Path) -> Result<u64, String> {
    let path = start.with_extension("runtime");
    std::fs::read_to_string(&path)
        .map_err(|error| format!("read writer runtime identity {}: {error}", path.display()))?
        .trim()
        .parse()
        .map_err(|error| format!("parse writer runtime identity {}: {error}", path.display()))
}
