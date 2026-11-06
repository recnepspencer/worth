use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use super::*;
use crate::certification_child_process::{
    encode_hex_32, fresh_challenge, validated_current_executable,
};
use crate::process_probe::{
    configure_process_probe, observe_graceful_exit, persist_execution, read_process_observation,
    terminate_by_parent,
};
use crate::{
    ProcessArtifactPath, ProcessIsolationRequirement, ProcessProbeDeclaration, ProcessProbeIntent,
    ProcessTerminationRequirement, SealedProcessProbeInput,
};

impl OperationalRecoveryFreshProcessRunner {
    pub fn certify_control_cut(
        &self,
        request: OperationalRecoveryControlCutRequest<'_>,
    ) -> Result<OperationalRecoveryCrashCutEvidence, OperationalRecoveryProcessCrashDenial> {
        self.certify_control_cut_with_process_evidence(request)
            .map(OperationalRecoveryProcessCrashEvidence::into_crash_cut)
    }

    pub fn certify_control_cut_with_process_evidence(
        &self,
        request: OperationalRecoveryControlCutRequest<'_>,
    ) -> Result<OperationalRecoveryProcessCrashEvidence, OperationalRecoveryProcessCrashDenial>
    {
        let OperationalRecoveryControlCutRequest {
            media_root,
            scenario_identity,
            cut_command,
            reopen_command,
            scenario_environment_keys,
            yieldpoint,
            uninterrupted_trace,
        } = request;
        std::fs::create_dir_all(&self.evidence_directory)
            .map_err(|_| OperationalRecoveryProcessCrashDenial::CutProcessLaunch)?;
        require_media_binding(cut_command, media_root)?;
        require_media_binding(reopen_command, media_root)?;
        let executable_identity = validated_executable_identity(cut_command, reopen_command)?;
        let challenge_subject = format!("{scenario_identity}:{}", yieldpoint.token());
        let challenge = fresh_challenge(
            b"worth-store-s10-process-crash-challenge-v1",
            challenge_subject.as_bytes(),
            executable_identity,
        );
        let paths = ProbePaths::new(&self.evidence_directory, challenge);
        configure_crash_command(cut_command, CUT_ROLE, &paths.cut, challenge, yieldpoint);
        configure_crash_command(
            reopen_command,
            REOPEN_ROLE,
            &paths.reopen,
            challenge,
            yieldpoint,
        );
        let admitted_environment_keys = admitted_crash_environment_keys(scenario_environment_keys);
        let cut_input =
            process_input(scenario_identity, "cut", yieldpoint, media_root, &paths.cut)?;
        let cut_intent = declaration(
            cut_command,
            &cut_input,
            ProcessRole::Writer,
            ProcessIsolationRequirement::ParentTerminated,
            ProcessTerminationRequirement::ParentKill,
        )?;
        let cut_declaration = configure_process_probe(
            cut_command,
            cut_intent,
            &cut_input,
            &paths.cut_process,
            &admitted_environment_keys,
        )?;
        let cut_execution = execute_cut(cut_command, cut_declaration, &cut_input, &paths)?;
        persist_execution(&self.evidence_directory, &cut_execution)?;
        let reopen_input = process_input(
            scenario_identity,
            "reopen",
            yieldpoint,
            media_root,
            &paths.reopen,
        )?;
        let reopen_intent = declaration(
            reopen_command,
            &reopen_input,
            ProcessRole::RecoveredRuntime,
            ProcessIsolationRequirement::FreshProcess,
            ProcessTerminationRequirement::GracefulExit,
        )?;
        let reopen_declaration = configure_process_probe(
            reopen_command,
            reopen_intent,
            &reopen_input,
            &paths.reopen_process,
            &admitted_environment_keys,
        )?;
        let reopen_execution =
            execute_reopen(reopen_command, reopen_declaration, &reopen_input, &paths)?;
        persist_execution(&self.evidence_directory, &reopen_execution)?;
        let crash_cut =
            validate_semantic_reports(&paths, challenge, yieldpoint, uninterrupted_trace)?;
        paths.remove_transient();
        Ok(OperationalRecoveryProcessCrashEvidence {
            crash_cut,
            cut_process: cut_execution,
            reopen_process: reopen_execution,
        })
    }
}

fn admitted_crash_environment_keys<'a>(scenario_environment_keys: &'a [&'a str]) -> Vec<&'a str> {
    let mut admitted = CRASH_ENVIRONMENT_KEYS.to_vec();
    admitted.extend_from_slice(scenario_environment_keys);
    admitted.sort_unstable();
    admitted.dedup();
    admitted
}

impl OperationalRecoveryProcessCrashEvidence {
    pub const fn crash_cut(&self) -> &OperationalRecoveryCrashCutEvidence {
        &self.crash_cut
    }
    pub const fn cut_process(&self) -> &ProcessProbeExecution {
        &self.cut_process
    }
    pub const fn reopen_process(&self) -> &ProcessProbeExecution {
        &self.reopen_process
    }
    pub fn into_crash_cut(self) -> OperationalRecoveryCrashCutEvidence {
        self.crash_cut
    }
}

fn execute_cut(
    command: &mut Command,
    declaration: ProcessProbeDeclaration,
    input: &SealedProcessProbeInput,
    paths: &ProbePaths,
) -> Result<ProcessProbeExecution, OperationalRecoveryProcessCrashDenial> {
    let mut child = command
        .spawn()
        .map_err(|_| OperationalRecoveryProcessCrashDenial::CutProcessLaunch)?;
    wait_for_cut_readiness(&mut child, &[&paths.cut, &paths.cut_process])?;
    if let Some(status) = child
        .try_wait()
        .map_err(|_| OperationalRecoveryProcessCrashDenial::CutProcessLaunch)?
    {
        return Err(
            OperationalRecoveryProcessCrashDenial::CutProcessExitedBeforeParentKill(status.code()),
        );
    }
    let termination = terminate_by_parent(&mut child)
        .map_err(|_| OperationalRecoveryProcessCrashDenial::CutProcessDidNotCrash(None))?;
    let identity = read_process_observation(&paths.cut_process, &declaration, child.id())?;
    ProcessProbeExecution::observed(declaration, input, identity, termination, &paths.cut)
        .map_err(Into::into)
}

fn execute_reopen(
    command: &mut Command,
    declaration: ProcessProbeDeclaration,
    input: &SealedProcessProbeInput,
    paths: &ProbePaths,
) -> Result<ProcessProbeExecution, OperationalRecoveryProcessCrashDenial> {
    let mut child = command
        .spawn()
        .map_err(|_| OperationalRecoveryProcessCrashDenial::ReopenProcessLaunch)?;
    let process_id = child.id();
    let status = child
        .wait()
        .map_err(|_| OperationalRecoveryProcessCrashDenial::ReopenProcessLaunch)?;
    if !status.success() {
        return Err(OperationalRecoveryProcessCrashDenial::ReopenProcessFailed(
            status.code(),
        ));
    }
    let identity = read_process_observation(&paths.reopen_process, &declaration, process_id)?;
    ProcessProbeExecution::observed(
        declaration,
        input,
        identity,
        observe_graceful_exit(status)?,
        &paths.reopen,
    )
    .map_err(Into::into)
}

fn validate_semantic_reports(
    paths: &ProbePaths,
    challenge: [u8; 32],
    yieldpoint: OperationalRecoveryYieldpoint,
    uninterrupted_trace: &OperationalRecoveryDriverTrace,
) -> Result<OperationalRecoveryCrashCutEvidence, OperationalRecoveryProcessCrashDenial> {
    let cut = read_report(&paths.cut)?;
    let reopened = read_report(&paths.reopen)?;
    if *cut.challenge.bytes() != challenge || *reopened.challenge.bytes() != challenge {
        return Err(OperationalRecoveryProcessCrashDenial::ChallengeMismatch);
    }
    if cut.yieldpoint != yieldpoint || reopened.yieldpoint != yieldpoint {
        return Err(OperationalRecoveryProcessCrashDenial::YieldpointMismatch);
    }
    if *cut.trace_identity.bytes() == [0; 32]
        || cut.operations.is_empty()
        || cut.operations.iter().any(|operation| {
            !uninterrupted_trace
                .operation_identities()
                .contains(operation)
        })
    {
        return Err(OperationalRecoveryProcessCrashDenial::TraceMismatch);
    }
    OperationalRecoveryCrashCutEvidence::from_external_process_reopen(
        yieldpoint,
        *cut.trace_identity.bytes(),
        cut.operations,
        cut.observation,
        reopened.observation,
        challenge,
    )
    .map_err(OperationalRecoveryProcessCrashDenial::CrashCut)
}

fn declaration(
    command: &Command,
    input: &SealedProcessProbeInput,
    role: ProcessRole,
    isolation: ProcessIsolationRequirement,
    termination: ProcessTerminationRequirement,
) -> Result<ProcessProbeIntent, OperationalRecoveryProcessCrashDenial> {
    ProcessProbeIntent::for_current_executable(command, input, role, isolation, termination)
        .map_err(Into::into)
}

fn configure_crash_command(
    command: &mut Command,
    role: &str,
    report_path: &Path,
    challenge: [u8; 32],
    yieldpoint: OperationalRecoveryYieldpoint,
) {
    command
        .env(PROCESS_CRASH_ROLE_ENV, role)
        .env(PROCESS_CRASH_REPORT_ENV, report_path)
        .env(PROCESS_CRASH_CHALLENGE_ENV, encode_hex_32(&challenge))
        .env(PROCESS_CRASH_YIELDPOINT_ENV, yieldpoint.token());
}

fn process_input(
    scenario: &str,
    role: &str,
    yieldpoint: OperationalRecoveryYieldpoint,
    media_root: &Path,
    report_path: &Path,
) -> Result<SealedProcessProbeInput, OperationalRecoveryProcessCrashDenial> {
    SealedProcessProbeInput::new(
        scenario,
        format!("{role}:{}", yieldpoint.token()),
        vec![
            ProcessArtifactPath::new("physical-media", media_root)?,
            ProcessArtifactPath::output_channel("semantic-report-channel", report_path)?,
        ],
    )
    .map_err(Into::into)
}

fn require_media_binding(
    command: &Command,
    media_root: &Path,
) -> Result<(), OperationalRecoveryProcessCrashDenial> {
    let expected = normalized_absolute(media_root)
        .ok_or(OperationalRecoveryProcessCrashDenial::InputArtifactMismatch)?;
    let matches = command.get_envs().any(|(_, value)| {
        value
            .and_then(|value| normalized_absolute(Path::new(value)))
            .is_some_and(|observed| observed == expected)
    });
    if matches {
        Ok(())
    } else {
        Err(OperationalRecoveryProcessCrashDenial::InputArtifactMismatch)
    }
}

fn normalized_absolute(path: &Path) -> Option<String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    Some(absolute.to_string_lossy().replace('\\', "/"))
}

fn wait_for_cut_readiness(
    child: &mut Child,
    required_paths: &[&Path],
) -> Result<(), OperationalRecoveryProcessCrashDenial> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if required_paths.iter().all(|path| path.is_file()) {
            return Ok(());
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|_| OperationalRecoveryProcessCrashDenial::CutProcessLaunch)?
        {
            return Err(
                OperationalRecoveryProcessCrashDenial::CutProcessExitedBeforeParentKill(
                    status.code(),
                ),
            );
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(OperationalRecoveryProcessCrashDenial::CutProcessReadinessTimedOut);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn validated_executable_identity(
    cut_command: &Command,
    reopen_command: &Command,
) -> Result<[u8; 32], OperationalRecoveryProcessCrashDenial> {
    let cut = validated_current_executable(cut_command)
        .ok_or(OperationalRecoveryProcessCrashDenial::ExecutableMismatch)?;
    let reopened = validated_current_executable(reopen_command)
        .ok_or(OperationalRecoveryProcessCrashDenial::ExecutableMismatch)?;
    if cut == reopened {
        Ok(cut)
    } else {
        Err(OperationalRecoveryProcessCrashDenial::ExecutableMismatch)
    }
}

struct ProbePaths {
    cut: PathBuf,
    reopen: PathBuf,
    cut_process: PathBuf,
    reopen_process: PathBuf,
}

impl ProbePaths {
    fn new(root: &Path, challenge: [u8; 32]) -> Self {
        let stem = encode_hex_32(&challenge);
        Self {
            cut: root.join(format!("{stem}.cut")),
            reopen: root.join(format!("{stem}.reopen")),
            cut_process: root.join(format!("{stem}.cut-process.json")),
            reopen_process: root.join(format!("{stem}.reopen-process.json")),
        }
    }

    fn remove_transient(&self) {
        for path in [
            &self.cut,
            &self.reopen,
            &self.cut_process,
            &self.reopen_process,
        ] {
            let _ = std::fs::remove_file(path);
        }
    }
}

const CRASH_ENVIRONMENT_KEYS: &[&str] = &[
    PROCESS_CRASH_ROLE_ENV,
    PROCESS_CRASH_REPORT_ENV,
    PROCESS_CRASH_CHALLENGE_ENV,
    PROCESS_CRASH_YIELDPOINT_ENV,
];
