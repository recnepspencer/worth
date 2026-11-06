use std::path::PathBuf;
use std::process::Command;
use worth_foundational::CanonicalDigestId;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{wire_encoding, ProcessArtifactPath};
use crate::certification_child_process::validated_current_executable;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessRole {
    Writer,
    CrashTarget,
    RecoveredRuntime,
    OfflineVerifier,
    FormalCheckerAdapter,
    AllocatorIsolatedProbe,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessIsolationRequirement {
    FreshProcess,
    ParentTerminated,
    IndependentObserver,
    IsolatedAllocator,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessTerminationRequirement {
    GracefulExit,
    PanicUnwind,
    Abort,
    ParentKill,
    OsTermination,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SealedProcessProbeInput {
    scenario_identity: String,
    fault_schedule_identity: String,
    artifacts: Vec<ProcessArtifactPath>,
    identity: CanonicalDigestId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessProbeDeclaration {
    scenario_identity: String,
    role: ProcessRole,
    isolation: ProcessIsolationRequirement,
    required_termination: ProcessTerminationRequirement,
    input_identity: CanonicalDigestId,
    executable_identity: CanonicalDigestId,
    working_directory: String,
    environment_identity: CanonicalDigestId,
    declaration_identity: CanonicalDigestId,
}

#[derive(Debug, Clone)]
pub struct ProcessProbeIntent {
    scenario_identity: String,
    role: ProcessRole,
    isolation: ProcessIsolationRequirement,
    required_termination: ProcessTerminationRequirement,
    input_identity: CanonicalDigestId,
    executable_identity: CanonicalDigestId,
    working_directory: String,
}

impl SealedProcessProbeInput {
    pub fn new(
        scenario_identity: impl Into<String>,
        fault_schedule_identity: impl Into<String>,
        mut artifacts: Vec<ProcessArtifactPath>,
    ) -> Result<Self, String> {
        let scenario_identity = scenario_identity.into();
        let fault_schedule_identity = fault_schedule_identity.into();
        if scenario_identity.trim().is_empty() || fault_schedule_identity.trim().is_empty() {
            return Err(
                "process scenario and fault-schedule identities cannot be empty".to_owned(),
            );
        }
        artifacts.sort_by(|left, right| left.purpose().cmp(right.purpose()));
        if artifacts
            .windows(2)
            .any(|pair| pair[0].purpose() == pair[1].purpose())
        {
            return Err("process input repeats an artifact purpose".to_owned());
        }
        let identity = digest_serialized(&(
            "worth-store-process-probe-input-v1",
            &scenario_identity,
            &fault_schedule_identity,
            &artifacts,
        ))?;
        Ok(Self {
            scenario_identity,
            fault_schedule_identity,
            artifacts,
            identity: CanonicalDigestId::new(identity),
        })
    }

    pub fn scenario_identity(&self) -> &str {
        &self.scenario_identity
    }

    pub fn fault_schedule_identity(&self) -> &str {
        &self.fault_schedule_identity
    }

    pub fn artifacts(&self) -> &[ProcessArtifactPath] {
        &self.artifacts
    }

    pub const fn identity(&self) -> [u8; 32] {
        *self.identity.bytes()
    }

    pub(crate) fn admits_output_path(&self, path: &std::path::Path) -> Result<bool, String> {
        self.artifacts.iter().try_fold(false, |admitted, artifact| {
            artifact
                .admits_output_path(path)
                .map(|matches| admitted || matches)
        })
    }

    pub fn decode_untrusted(bytes: &[u8]) -> Result<Self, String> {
        Self::decode(bytes, true)
    }

    pub(crate) fn decode_declared(bytes: &[u8]) -> Result<Self, String> {
        Self::decode(bytes, false)
    }

    fn decode(bytes: &[u8], reobserve_inputs: bool) -> Result<Self, String> {
        let decoded: Self = wire_encoding::decode(bytes)?;
        if reobserve_inputs {
            for artifact in &decoded.artifacts {
                artifact.validate_child_admission()?;
            }
        }
        let checked = Self::new(
            decoded.scenario_identity.clone(),
            decoded.fault_schedule_identity.clone(),
            decoded.artifacts.clone(),
        )?;
        if decoded != checked {
            return Err("process probe input identity mismatch".to_owned());
        }
        Ok(checked)
    }
}

impl ProcessProbeIntent {
    pub fn for_current_executable(
        command: &Command,
        input: &SealedProcessProbeInput,
        role: ProcessRole,
        isolation: ProcessIsolationRequirement,
        required_termination: ProcessTerminationRequirement,
    ) -> Result<Self, String> {
        validate_process_contract(role, isolation, required_termination)?;
        let executable_identity = validated_current_executable(command).ok_or_else(|| {
            "process probe command is not the current sealed executable".to_owned()
        })?;
        let working_directory = command
            .get_current_dir()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "process probe working directory is unavailable".to_owned())?;
        let working_directory = std::fs::canonicalize(&working_directory)
            .unwrap_or(working_directory)
            .to_string_lossy()
            .replace('\\', "/");
        Ok(Self {
            scenario_identity: input.scenario_identity().to_owned(),
            role,
            isolation,
            required_termination,
            input_identity: CanonicalDigestId::new(input.identity()),
            executable_identity: CanonicalDigestId::new(executable_identity),
            working_directory,
        })
    }

    pub(crate) fn bind_environment(
        self,
        environment_identity: [u8; 32],
    ) -> Result<ProcessProbeDeclaration, String> {
        let declaration_identity = digest_serialized(&(
            "worth-store-process-probe-declaration-v2",
            &self.scenario_identity,
            self.role,
            self.isolation,
            self.required_termination,
            self.input_identity,
            self.executable_identity,
            &self.working_directory,
            CanonicalDigestId::new(environment_identity),
        ))?;
        Ok(ProcessProbeDeclaration {
            scenario_identity: self.scenario_identity,
            role: self.role,
            isolation: self.isolation,
            required_termination: self.required_termination,
            input_identity: self.input_identity,
            executable_identity: self.executable_identity,
            working_directory: self.working_directory,
            environment_identity: CanonicalDigestId::new(environment_identity),
            declaration_identity: CanonicalDigestId::new(declaration_identity),
        })
    }

    pub const fn role(&self) -> ProcessRole {
        self.role
    }

    pub const fn input_identity(&self) -> [u8; 32] {
        *self.input_identity.bytes()
    }
}

fn validate_process_contract(
    role: ProcessRole,
    isolation: ProcessIsolationRequirement,
    termination: ProcessTerminationRequirement,
) -> Result<(), String> {
    let role_admits_isolation = matches!(
        (role, isolation),
        (
            ProcessRole::Writer | ProcessRole::CrashTarget,
            ProcessIsolationRequirement::FreshProcess
                | ProcessIsolationRequirement::ParentTerminated
        ) | (
            ProcessRole::RecoveredRuntime,
            ProcessIsolationRequirement::FreshProcess
        ) | (
            ProcessRole::OfflineVerifier | ProcessRole::FormalCheckerAdapter,
            ProcessIsolationRequirement::IndependentObserver
        ) | (
            ProcessRole::AllocatorIsolatedProbe,
            ProcessIsolationRequirement::IsolatedAllocator
        )
    );
    let isolation_admits_termination = match isolation {
        ProcessIsolationRequirement::FreshProcess => {
            termination != ProcessTerminationRequirement::ParentKill
        }
        ProcessIsolationRequirement::ParentTerminated => {
            termination == ProcessTerminationRequirement::ParentKill
        }
        ProcessIsolationRequirement::IndependentObserver
        | ProcessIsolationRequirement::IsolatedAllocator => {
            termination == ProcessTerminationRequirement::GracefulExit
        }
    };
    if role_admits_isolation && isolation_admits_termination {
        Ok(())
    } else {
        Err(format!(
            "process role {role:?} cannot claim {isolation:?} with {termination:?} termination"
        ))
    }
}

impl ProcessProbeDeclaration {
    pub fn scenario_identity(&self) -> &str {
        &self.scenario_identity
    }

    pub const fn role(&self) -> ProcessRole {
        self.role
    }

    pub const fn isolation(&self) -> ProcessIsolationRequirement {
        self.isolation
    }

    pub const fn required_termination(&self) -> ProcessTerminationRequirement {
        self.required_termination
    }

    pub const fn input_identity(&self) -> [u8; 32] {
        *self.input_identity.bytes()
    }

    pub const fn executable_identity(&self) -> [u8; 32] {
        *self.executable_identity.bytes()
    }

    pub fn working_directory(&self) -> &str {
        &self.working_directory
    }

    pub const fn environment_identity(&self) -> [u8; 32] {
        *self.environment_identity.bytes()
    }

    pub const fn declaration_identity(&self) -> [u8; 32] {
        *self.declaration_identity.bytes()
    }
}

fn digest_serialized(value: &impl Serialize) -> Result<[u8; 32], String> {
    wire_encoding::encode(value).map(|bytes| Sha256::digest(bytes).into())
}
