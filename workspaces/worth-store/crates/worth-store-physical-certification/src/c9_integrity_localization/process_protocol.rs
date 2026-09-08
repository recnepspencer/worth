use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::process_recovery_observation::ProcessRecoveryObservation;
use super::{
    ClosedStoreProcessManifest, DeclaredProcessPoison, ProcessEditorAudit, RootWireIdentity,
    RootWireRole,
};

pub(crate) const SUBJECT_REQUEST_ENV: &str = "WORTH_C9_SUBJECT_REQUEST";
const PROCESS_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProcessSubjectRequest {
    version: u16,
    role: RootWireRole,
    scenario_identity: [u8; 32],
    run_identity: [u8; 32],
    store_root: PathBuf,
    report_path: PathBuf,
    store_identity: Option<[u8; 16]>,
    manifest: Option<ClosedStoreProcessManifest>,
    poison: Option<DeclaredProcessPoison>,
    profile: super::production_profile::ProductionWorldProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProcessSubjectReport {
    version: u16,
    wire: RootWireIdentity,
    process: ProcessIdentity,
    payload: ProcessReportPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessIdentity {
    process_id: u32,
    executable_sha256: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ProcessReportPayload {
    Produced(ClosedStoreProcessManifest),
    Edited(ProcessEditorAudit),
    Recovered(ProcessRecoveryObservation),
}

impl ProcessSubjectRequest {
    pub(crate) fn producer(
        scenario_identity: [u8; 32],
        run_identity: [u8; 32],
        store_root: PathBuf,
        report_path: PathBuf,
    ) -> Self {
        Self {
            version: PROCESS_PROTOCOL_VERSION,
            role: RootWireRole::Producer,
            scenario_identity,
            run_identity,
            store_root,
            report_path,
            store_identity: None,
            manifest: None,
            poison: None,
            profile: super::production_profile::ProductionWorldProfile::Primary16KiB,
        }
    }

    pub(crate) fn editor(
        scenario_identity: [u8; 32],
        run_identity: [u8; 32],
        store_root: PathBuf,
        report_path: PathBuf,
        manifest: ClosedStoreProcessManifest,
        poison: DeclaredProcessPoison,
    ) -> Self {
        Self {
            version: PROCESS_PROTOCOL_VERSION,
            role: RootWireRole::ArtifactEditor,
            scenario_identity,
            run_identity,
            store_root,
            report_path,
            store_identity: Some(manifest.store_identity()),
            manifest: Some(manifest),
            poison: Some(poison),
            profile: super::production_profile::ProductionWorldProfile::Primary16KiB,
        }
    }

    pub(crate) fn recovery(
        scenario_identity: [u8; 32],
        run_identity: [u8; 32],
        store_root: PathBuf,
        report_path: PathBuf,
        store_identity: [u8; 16],
    ) -> Self {
        Self {
            version: PROCESS_PROTOCOL_VERSION,
            role: RootWireRole::Recovery,
            scenario_identity,
            run_identity,
            store_root,
            report_path,
            store_identity: Some(store_identity),
            manifest: None,
            poison: None,
            profile: super::production_profile::ProductionWorldProfile::Primary16KiB,
        }
    }

    pub(crate) const fn role(&self) -> RootWireRole {
        self.role
    }
    pub(crate) const fn profile(&self) -> super::production_profile::ProductionWorldProfile {
        self.profile
    }
    pub(crate) fn with_profile(
        mut self,
        profile: super::production_profile::ProductionWorldProfile,
    ) -> Self {
        self.profile = profile;
        self
    }
    pub(crate) const fn scenario_identity(&self) -> [u8; 32] {
        self.scenario_identity
    }
    pub(crate) const fn run_identity(&self) -> [u8; 32] {
        self.run_identity
    }
    pub(crate) fn store_root(&self) -> &Path {
        &self.store_root
    }
    pub(crate) fn report_path(&self) -> &Path {
        &self.report_path
    }
    pub(crate) const fn store_identity(&self) -> Option<[u8; 16]> {
        self.store_identity
    }
    pub(crate) fn manifest(&self) -> Option<&ClosedStoreProcessManifest> {
        self.manifest.as_ref()
    }
    pub(crate) fn poison(&self) -> Option<&DeclaredProcessPoison> {
        self.poison.as_ref()
    }
    pub(crate) fn require_version(&self) -> Result<(), String> {
        (self.version == PROCESS_PROTOCOL_VERSION)
            .then_some(())
            .ok_or_else(|| "process request protocol substitution".to_owned())
    }
}

impl ProcessSubjectReport {
    pub(crate) fn new(
        wire: RootWireIdentity,
        payload: ProcessReportPayload,
    ) -> Result<Self, String> {
        Ok(Self {
            version: PROCESS_PROTOCOL_VERSION,
            wire,
            process: ProcessIdentity::current()?,
            payload,
        })
    }

    pub(crate) fn require(
        &self,
        role: RootWireRole,
        scenario: [u8; 32],
        run: [u8; 32],
        store: [u8; 16],
        observed_process: u32,
        executable_sha256: [u8; 32],
    ) -> Result<(), String> {
        if self.version != PROCESS_PROTOCOL_VERSION {
            return Err("process report protocol substitution".to_owned());
        }
        self.wire
            .require_binding(role, scenario, run, store)
            .map_err(|denial| format!("process report binding denied: {denial:?}"))?;
        if self.process.process_id != observed_process {
            return Err("process identity substitution".to_owned());
        }
        if self.process.executable_sha256 != executable_sha256 {
            return Err("process executable substitution".to_owned());
        }
        Ok(())
    }

    pub(crate) fn payload(&self) -> &ProcessReportPayload {
        &self.payload
    }
}

impl ProcessIdentity {
    fn current() -> Result<Self, String> {
        Ok(Self {
            process_id: std::process::id(),
            executable_sha256: executable_sha256(
                &std::env::current_exe()
                    .map_err(|error| format!("resolve current executable: {error}"))?,
            )?,
        })
    }
}

pub(crate) fn write_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes =
        bincode::serialize(value).map_err(|error| format!("encode process wire: {error}"))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("create process wire {}: {error}", path.display()))?;
    output
        .write_all(&bytes)
        .and_then(|()| output.sync_all())
        .map_err(|error| format!("write process wire {}: {error}", path.display()))
}

pub(crate) fn read_wire<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|mut input| input.read_to_end(&mut bytes))
        .map_err(|error| format!("read process wire {}: {error}", path.display()))?;
    bincode::deserialize(&bytes).map_err(|error| format!("decode process wire: {error}"))
}

pub(crate) fn executable_sha256(path: &Path) -> Result<[u8; 32], String> {
    std::fs::read(path)
        .map(|bytes| Sha256::digest(bytes).into())
        .map_err(|error| format!("digest executable {}: {error}", path.display()))
}
