use std::ffi::OsString;
use std::process::Command;
use worth_foundational::CanonicalDigestId;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{wire_encoding, ProcessProbeEvidenceDenial};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessEnvironmentBindingEvidence {
    pub name: String,
    pub value_sha256: CanonicalDigestId,
}

pub(super) fn inherited_runtime_bindings() -> Vec<(String, OsString)> {
    runtime_environment_names()
        .iter()
        .filter_map(|name| std::env::var_os(name).map(|value| ((*name).to_owned(), value)))
        .collect()
}

pub(super) fn admitted_name(name: &str) -> bool {
    name.starts_with("WORTH_STORE_") || runtime_environment_names().contains(&name)
}

pub(super) fn command_bindings(
    command: &Command,
    keys: &[String],
) -> Result<Vec<ProcessEnvironmentBindingEvidence>, ProcessProbeEvidenceDenial> {
    keys.iter()
        .map(|key| {
            let value = command
                .get_envs()
                .find(|(name, _)| *name == std::ffi::OsStr::new(key))
                .and_then(|(_, value)| value)
                .ok_or(ProcessProbeEvidenceDenial::UnadmittedEnvironment)?;
            Ok(binding(key, value))
        })
        .collect()
}

pub(super) fn current_bindings(
    keys: &[String],
) -> Result<Vec<ProcessEnvironmentBindingEvidence>, ProcessProbeEvidenceDenial> {
    if keys.iter().any(|key| !admitted_name(key)) {
        return Err(ProcessProbeEvidenceDenial::UnadmittedEnvironment);
    }
    keys.iter()
        .map(|key| {
            let value =
                std::env::var_os(key).ok_or(ProcessProbeEvidenceDenial::InvalidChildObservation)?;
            Ok(binding(key, &value))
        })
        .collect()
}

pub(super) fn identity(
    bindings: &[ProcessEnvironmentBindingEvidence],
) -> Result<[u8; 32], ProcessProbeEvidenceDenial> {
    wire_encoding::encode(bindings)
        .map(|bytes| Sha256::digest(bytes).into())
        .map_err(|_| ProcessProbeEvidenceDenial::InvalidChildObservation)
}

pub(super) fn is_admitted(bindings: &[ProcessEnvironmentBindingEvidence]) -> bool {
    !bindings.is_empty()
        && bindings
            .iter()
            .all(|binding| admitted_name(&binding.name) && *binding.value_sha256.bytes() != [0; 32])
        && bindings.windows(2).all(|pair| pair[0].name < pair[1].name)
}

fn binding(name: &str, value: &std::ffi::OsStr) -> ProcessEnvironmentBindingEvidence {
    ProcessEnvironmentBindingEvidence {
        name: name.to_owned(),
        value_sha256: CanonicalDigestId::new(
            Sha256::digest(value.to_string_lossy().as_bytes()).into(),
        ),
    }
}

#[cfg(windows)]
const fn runtime_environment_names() -> &'static [&'static str] {
    &["SYSTEMROOT", "TEMP", "TMP", "WINDIR"]
}

#[cfg(not(windows))]
const fn runtime_environment_names() -> &'static [&'static str] {
    &["TMPDIR"]
}
