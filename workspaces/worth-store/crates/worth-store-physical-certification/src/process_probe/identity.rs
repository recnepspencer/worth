use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use worth_foundational::CanonicalDigestId;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    environment, wire_encoding, ProcessEnvironmentBindingEvidence, ProcessProbeDeclaration,
    ProcessProbeEvidenceDenial, ProcessProbeIntent, ProcessRole, SealedProcessProbeInput,
};
use crate::certification_child_process::{decode_hex_32, encode_hex_32, publish_new_synced};

const ROLE_ENV: &str = "WORTH_STORE_PROCESS_PROBE_ROLE";
const PARENT_ENV: &str = "WORTH_STORE_PROCESS_PROBE_PARENT_PID";
const INPUT_ENV: &str = "WORTH_STORE_PROCESS_PROBE_INPUT_IDENTITY";
const SEALED_INPUT_ENV: &str = "WORTH_STORE_PROCESS_PROBE_SEALED_INPUT";
const OBSERVATION_ENV: &str = "WORTH_STORE_PROCESS_PROBE_OBSERVATION";
const ENVIRONMENT_KEYS_ENV: &str = "WORTH_STORE_PROCESS_PROBE_ENVIRONMENT_KEYS";
const ENVIRONMENT_IDENTITY_ENV: &str = "WORTH_STORE_PROCESS_PROBE_ENVIRONMENT_IDENTITY";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessIdentityEvidence {
    pub role: ProcessRole,
    pub executable_identity: CanonicalDigestId,
    pub process_id: u32,
    pub launch_parent_process_id: u32,
    pub working_directory: String,
    pub working_directory_identity: CanonicalDigestId,
    pub environment: Vec<ProcessEnvironmentBindingEvidence>,
    pub environment_identity: CanonicalDigestId,
    pub input_artifact_identity: CanonicalDigestId,
    pub runtime_identity: Option<CanonicalDigestId>,
}

#[derive(Debug, Clone)]
pub struct AdmittedProcessProbe {
    role: ProcessRole,
    input_identity: CanonicalDigestId,
}

#[derive(Debug)]
pub(crate) struct ObservedProcessIdentity(ProcessIdentityEvidence);

impl ObservedProcessIdentity {
    pub(crate) fn into_evidence(self) -> ProcessIdentityEvidence {
        self.0
    }
}

pub(crate) fn configure_process_probe(
    command: &mut Command,
    intent: ProcessProbeIntent,
    input: &SealedProcessProbeInput,
    observation_path: &Path,
    additional_environment_keys: &[&str],
) -> Result<ProcessProbeDeclaration, ProcessProbeEvidenceDenial> {
    if input.identity() != intent.input_identity() {
        return Err(ProcessProbeEvidenceDenial::InputIdentityMismatch);
    }
    let sealed_input = wire_encoding::encode_environment(input)
        .map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    let protocol_keys = [
        ROLE_ENV,
        PARENT_ENV,
        INPUT_ENV,
        SEALED_INPUT_ENV,
        OBSERVATION_ENV,
        ENVIRONMENT_KEYS_ENV,
    ];
    if additional_environment_keys
        .iter()
        .any(|key| protocol_keys.contains(key) || *key == ENVIRONMENT_IDENTITY_ENV)
    {
        return Err(ProcessProbeEvidenceDenial::UnadmittedEnvironment);
    }
    let additional_bindings = explicit_command_environment(command, additional_environment_keys)?;
    let runtime_bindings = environment::inherited_runtime_bindings();
    let mut keys = protocol_keys
        .iter()
        .chain(additional_environment_keys)
        .map(|key| (*key).to_owned())
        .collect::<Vec<_>>();
    keys.extend(runtime_bindings.iter().map(|(name, _)| name.clone()));
    keys.sort_unstable();
    keys.dedup();
    if keys
        .iter()
        .any(|key| !environment::admitted_name(key) || key.contains(';'))
    {
        return Err(ProcessProbeEvidenceDenial::UnadmittedEnvironment);
    }
    command.env_clear();
    for (name, value) in runtime_bindings {
        command.env(name, value);
    }
    for (name, value) in additional_bindings {
        command.env(name, value);
    }
    command
        .env(ROLE_ENV, role_token(intent.role()))
        .env(PARENT_ENV, std::process::id().to_string())
        .env(INPUT_ENV, encode_hex_32(&intent.input_identity()))
        .env(SEALED_INPUT_ENV, sealed_input)
        .env(OBSERVATION_ENV, observation_path)
        .env(ENVIRONMENT_KEYS_ENV, keys.join(";"));
    let bindings = environment::command_bindings(command, &keys)?;
    let identity = environment::identity(&bindings)?;
    command.env(ENVIRONMENT_IDENTITY_ENV, encode_hex_32(&identity));
    intent
        .bind_environment(identity)
        .map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)
}

pub fn admit_current_process_probe(
    expected_role: ProcessRole,
) -> Result<AdmittedProcessProbe, ProcessProbeEvidenceDenial> {
    let role = parse_role(&required_string(ROLE_ENV)?)
        .ok_or(ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    if role != expected_role {
        return Err(ProcessProbeEvidenceDenial::RoleMismatch);
    }
    let input_identity = decode_hex_32(&required_string(INPUT_ENV)?)
        .ok_or(ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    let sealed_input = wire_encoding::decode_environment::<SealedProcessProbeInput>(
        &required_string(SEALED_INPUT_ENV)?,
    )
    .and_then(|decoded| {
        SealedProcessProbeInput::decode_untrusted(&wire_encoding::encode(&decoded)?)
    })
    .map_err(|_| ProcessProbeEvidenceDenial::InputIdentityMismatch)?;
    if sealed_input.identity() != input_identity {
        return Err(ProcessProbeEvidenceDenial::InputIdentityMismatch);
    }
    validate_current_environment()?;
    Ok(AdmittedProcessProbe {
        role,
        input_identity: CanonicalDigestId::new(input_identity),
    })
}

pub(crate) fn write_current_process_observation(
    admission: &AdmittedProcessProbe,
    runtime_identity: Option<[u8; 32]>,
) -> Result<(), ProcessProbeEvidenceDenial> {
    let observation_path = required_path(OBSERVATION_ENV)?;
    let role = parse_role(&required_string(ROLE_ENV)?)
        .ok_or(ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    if role != admission.role {
        return Err(ProcessProbeEvidenceDenial::RoleMismatch);
    }
    let launch_parent_process_id = required_string(PARENT_ENV)?
        .parse()
        .map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    let input_artifact_identity = decode_hex_32(&required_string(INPUT_ENV)?)
        .ok_or(ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    let sealed_input = wire_encoding::decode_environment::<SealedProcessProbeInput>(
        &required_string(SEALED_INPUT_ENV)?,
    )
    .and_then(|decoded| SealedProcessProbeInput::decode_declared(&wire_encoding::encode(&decoded)?))
    .map_err(|_| ProcessProbeEvidenceDenial::InputIdentityMismatch)?;
    if sealed_input.identity() != input_artifact_identity
        || input_artifact_identity != *admission.input_identity.bytes()
    {
        return Err(ProcessProbeEvidenceDenial::InputIdentityMismatch);
    }
    let (environment, environment_identity) = validate_current_environment()?;
    let executable = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    let executable_identity = file_digest(&executable)?;
    let working_directory = std::env::current_dir()
        .and_then(fs::canonicalize)
        .map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)?
        .to_string_lossy()
        .replace('\\', "/");
    let evidence = ProcessIdentityEvidence {
        role,
        executable_identity: CanonicalDigestId::new(executable_identity),
        process_id: std::process::id(),
        launch_parent_process_id,
        working_directory_identity: CanonicalDigestId::new(
            Sha256::digest(working_directory.as_bytes()).into(),
        ),
        working_directory,
        environment,
        environment_identity: CanonicalDigestId::new(environment_identity),
        input_artifact_identity: CanonicalDigestId::new(input_artifact_identity),
        runtime_identity: match runtime_identity {
            Some(bytes) => Some(CanonicalDigestId::new(bytes)),
            None => None,
        },
    };
    write_new_observation(&observation_path, &evidence)
}

fn validate_current_environment(
) -> Result<(Vec<ProcessEnvironmentBindingEvidence>, [u8; 32]), ProcessProbeEvidenceDenial> {
    let declared_environment_identity = decode_hex_32(&required_string(ENVIRONMENT_IDENTITY_ENV)?)
        .ok_or(ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    let keys = required_string(ENVIRONMENT_KEYS_ENV)?
        .split(';')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let environment = environment::current_bindings(&keys)?;
    if !environment::is_admitted(&environment) {
        return Err(ProcessProbeEvidenceDenial::EnvironmentMismatch);
    }
    let environment_identity = environment::identity(&environment)?;
    if environment_identity != declared_environment_identity {
        return Err(ProcessProbeEvidenceDenial::EnvironmentMismatch);
    }
    Ok((environment, environment_identity))
}

pub(crate) fn read_process_observation(
    path: &Path,
    declaration: &ProcessProbeDeclaration,
    spawned_process_id: u32,
) -> Result<ObservedProcessIdentity, ProcessProbeEvidenceDenial> {
    let bytes = fs::read(path).map_err(|_| ProcessProbeEvidenceDenial::MissingChildObservation)?;
    let evidence: ProcessIdentityEvidence = wire_encoding::decode(&bytes)
        .map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    evidence.validate_against(declaration, spawned_process_id)?;
    Ok(ObservedProcessIdentity(evidence))
}

impl ProcessIdentityEvidence {
    pub(crate) fn validate_against(
        &self,
        declaration: &ProcessProbeDeclaration,
        spawned_process_id: u32,
    ) -> Result<(), ProcessProbeEvidenceDenial> {
        if self.role != declaration.role() {
            return Err(ProcessProbeEvidenceDenial::RoleMismatch);
        }
        if self.process_id == 0
            || self.process_id != spawned_process_id
            || self.process_id == std::process::id()
            || self.launch_parent_process_id != std::process::id()
        {
            return Err(ProcessProbeEvidenceDenial::ProcessRelationshipMismatch);
        }
        if *self.executable_identity.bytes() != declaration.executable_identity() {
            return Err(ProcessProbeEvidenceDenial::ExecutableMismatch);
        }
        if *self.input_artifact_identity.bytes() != declaration.input_identity() {
            return Err(ProcessProbeEvidenceDenial::InputIdentityMismatch);
        }
        let working_directory_identity: [u8; 32] =
            Sha256::digest(self.working_directory.as_bytes()).into();
        if self.working_directory != declaration.working_directory()
            || *self.working_directory_identity.bytes() != working_directory_identity
        {
            return Err(ProcessProbeEvidenceDenial::WorkingDirectoryMismatch);
        }
        if !environment::is_admitted(&self.environment)
            || environment::identity(&self.environment)? != *self.environment_identity.bytes()
            || *self.environment_identity.bytes() != declaration.environment_identity()
        {
            return Err(ProcessProbeEvidenceDenial::EnvironmentMismatch);
        }
        Ok(())
    }
}

fn explicit_command_environment(
    command: &Command,
    keys: &[&str],
) -> Result<Vec<(String, std::ffi::OsString)>, ProcessProbeEvidenceDenial> {
    keys.iter()
        .map(|key| {
            let value = command
                .get_envs()
                .find(|(name, _)| *name == std::ffi::OsStr::new(key))
                .and_then(|(_, value)| value)
                .ok_or(ProcessProbeEvidenceDenial::UnadmittedEnvironment)?;
            Ok(((*key).to_owned(), value.to_owned()))
        })
        .collect()
}

fn write_new_observation(
    path: &Path,
    evidence: &ProcessIdentityEvidence,
) -> Result<(), ProcessProbeEvidenceDenial> {
    let bytes =
        wire_encoding::encode(evidence).map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
    publish_new_synced(path, &bytes).map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)
}

fn file_digest(path: &Path) -> Result<[u8; 32], ProcessProbeEvidenceDenial> {
    let mut file =
        fs::File::open(path).map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

fn required_string(name: &str) -> Result<String, ProcessProbeEvidenceDenial> {
    std::env::var(name).map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)
}

fn required_path(name: &str) -> Result<PathBuf, ProcessProbeEvidenceDenial> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .ok_or(ProcessProbeEvidenceDenial::InvalidChildObservation)
}

const fn role_token(role: ProcessRole) -> &'static str {
    match role {
        ProcessRole::Writer => "writer",
        ProcessRole::CrashTarget => "crash-target",
        ProcessRole::RecoveredRuntime => "recovered-runtime",
        ProcessRole::OfflineVerifier => "offline-verifier",
        ProcessRole::FormalCheckerAdapter => "formal-checker-adapter",
        ProcessRole::AllocatorIsolatedProbe => "allocator-isolated-probe",
    }
}

fn parse_role(value: &str) -> Option<ProcessRole> {
    Some(match value {
        "writer" => ProcessRole::Writer,
        "crash-target" => ProcessRole::CrashTarget,
        "recovered-runtime" => ProcessRole::RecoveredRuntime,
        "offline-verifier" => ProcessRole::OfflineVerifier,
        "formal-checker-adapter" => ProcessRole::FormalCheckerAdapter,
        "allocator-isolated-probe" => ProcessRole::AllocatorIsolatedProbe,
        _ => return None,
    })
}
