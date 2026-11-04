use std::path::PathBuf;
use worth_foundational::CanonicalDigestId;

use worth_store_operations::{OperationalControlSessionObservation, OperationalControlStore};

use crate::certification_child_process::decode_hex_32;
use crate::process_probe::write_current_process_observation;
use crate::{
    admit_current_process_probe, AdmittedProcessProbe, OperationalRecoveryCrashCutDenial,
    OperationalRecoveryCrashCutEvidence, OperationalRecoveryDriverTrace,
    OperationalRecoveryYieldpoint, ProcessProbeEvidenceDenial, ProcessProbeExecution, ProcessRole,
};

const CUT_ROLE: &str = "cut";
const REOPEN_ROLE: &str = "reopen";
mod request;
mod runner;
mod wire;
pub use request::OperationalRecoveryControlCutRequest;
use wire::{read_report, write_report, ProcessObservationReport};
pub const PROCESS_CRASH_ROLE_ENV: &str = "WORTH_STORE_S10_CRASH_ROLE";
pub const PROCESS_CRASH_REPORT_ENV: &str = "WORTH_STORE_S10_CRASH_REPORT";
pub const PROCESS_CRASH_CHALLENGE_ENV: &str = "WORTH_STORE_S10_CRASH_CHALLENGE";
pub const PROCESS_CRASH_YIELDPOINT_ENV: &str = "WORTH_STORE_S10_CRASH_YIELDPOINT";

#[derive(Debug, Clone)]
pub struct OperationalRecoveryProcessCrashConfig {
    yieldpoint: OperationalRecoveryYieldpoint,
    report_path: PathBuf,
    challenge: CanonicalDigestId,
    admission: AdmittedProcessProbe,
}

#[derive(Debug)]
pub struct OperationalRecoveryFreshProcessRunner {
    evidence_directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalRecoveryProcessCrashEvidence {
    crash_cut: OperationalRecoveryCrashCutEvidence,
    cut_process: ProcessProbeExecution,
    reopen_process: ProcessProbeExecution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationalRecoveryProcessCrashDenial {
    InvalidEnvironment,
    CutProcessLaunch,
    ReopenProcessLaunch,
    CutProcessDidNotCrash(Option<i32>),
    CutProcessExitedBeforeParentKill(Option<i32>),
    CutProcessReadinessTimedOut,
    ReopenProcessFailed(Option<i32>),
    MissingOrMalformedReport,
    ChallengeMismatch,
    YieldpointMismatch,
    TraceMismatch,
    ExecutableMismatch,
    InputArtifactMismatch,
    ProcessProbe(ProcessProbeEvidenceDenial),
    CrashCut(OperationalRecoveryCrashCutDenial),
}

impl OperationalRecoveryProcessCrashConfig {
    pub fn from_environment() -> Result<Option<Self>, OperationalRecoveryProcessCrashDenial> {
        if std::env::var(PROCESS_CRASH_ROLE_ENV).ok().as_deref() != Some(CUT_ROLE) {
            return Ok(None);
        }
        let report_path = std::env::var_os(PROCESS_CRASH_REPORT_ENV)
            .map(PathBuf::from)
            .ok_or(OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?;
        let challenge = decode_hex_32(
            &std::env::var(PROCESS_CRASH_CHALLENGE_ENV)
                .map_err(|_| OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?,
        )
        .ok_or(OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?;
        let token = std::env::var(PROCESS_CRASH_YIELDPOINT_ENV)
            .map_err(|_| OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?;
        let yieldpoint = OperationalRecoveryYieldpoint::from_token(&token)
            .ok_or(OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?;
        let admission = admit_current_process_probe(ProcessRole::Writer)?;
        Ok(Some(Self {
            yieldpoint,
            report_path,
            challenge: CanonicalDigestId::new(challenge),
            admission,
        }))
    }

    pub const fn yieldpoint(&self) -> OperationalRecoveryYieldpoint {
        self.yieldpoint
    }

    pub(crate) fn crash_with_control_observation(
        &self,
        observation: OperationalControlSessionObservation,
        trace: &OperationalRecoveryDriverTrace,
    ) -> ! {
        let report = ProcessObservationReport {
            challenge: CanonicalDigestId::new(*self.challenge.bytes()),
            yieldpoint: self.yieldpoint,
            observation,
            trace_identity: CanonicalDigestId::new(trace.evidence_identity()),
            operations: trace.operation_identities().to_vec(),
        };
        write_report(&self.report_path, &report).expect("write durable S10 crash-cut report");
        write_current_process_observation(
            &self.admission,
            Some(observation.process().fingerprint()),
        )
        .expect("write S10 writer process observation");
        loop {
            std::thread::park();
        }
    }
}

pub fn write_reopen_observation_from_environment(
    admission: &AdmittedProcessProbe,
    store: &OperationalControlStore,
) -> Result<bool, OperationalRecoveryProcessCrashDenial> {
    if std::env::var(PROCESS_CRASH_ROLE_ENV).ok().as_deref() != Some(REOPEN_ROLE) {
        return Ok(false);
    }
    let report_path = std::env::var_os(PROCESS_CRASH_REPORT_ENV)
        .map(PathBuf::from)
        .ok_or(OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?;
    let challenge = decode_hex_32(
        &std::env::var(PROCESS_CRASH_CHALLENGE_ENV)
            .map_err(|_| OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?,
    )
    .ok_or(OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?;
    let token = std::env::var(PROCESS_CRASH_YIELDPOINT_ENV)
        .map_err(|_| OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?;
    let yieldpoint = OperationalRecoveryYieldpoint::from_token(&token)
        .ok_or(OperationalRecoveryProcessCrashDenial::InvalidEnvironment)?;
    let observation = store
        .session_observation()
        .map_err(|_| OperationalRecoveryProcessCrashDenial::MissingOrMalformedReport)?;
    write_report(
        &report_path,
        &ProcessObservationReport {
            challenge: CanonicalDigestId::new(challenge),
            yieldpoint,
            observation,
            trace_identity: CanonicalDigestId::new([0; 32]),
            operations: Vec::new(),
        },
    )
    .map_err(|_| OperationalRecoveryProcessCrashDenial::MissingOrMalformedReport)?;
    write_current_process_observation(admission, Some(observation.process().fingerprint()))?;
    Ok(true)
}

impl OperationalRecoveryFreshProcessRunner {
    pub fn new(evidence_directory: impl Into<PathBuf>) -> Self {
        Self {
            evidence_directory: evidence_directory.into(),
        }
    }
}

impl From<ProcessProbeEvidenceDenial> for OperationalRecoveryProcessCrashDenial {
    fn from(value: ProcessProbeEvidenceDenial) -> Self {
        Self::ProcessProbe(value)
    }
}

impl From<String> for OperationalRecoveryProcessCrashDenial {
    fn from(_: String) -> Self {
        Self::InvalidEnvironment
    }
}
