use std::path::{Path, PathBuf};

use crate::ProtocolFamily;

use super::ProtocolCheckBounds;

pub const PINNED_TLC_SHA256: &str =
    "936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCheckInvocation {
    protocol: ProtocolFamily,
    model_path: PathBuf,
    configuration_path: PathBuf,
    bounds: ProtocolCheckBounds,
}

impl ProtocolCheckInvocation {
    pub fn for_checked_protocol(
        protocol: ProtocolFamily,
        crate_root: impl AsRef<Path>,
        bounds: ProtocolCheckBounds,
    ) -> Self {
        let (model, configuration) = checked_artifact_paths(protocol);
        Self {
            protocol,
            model_path: crate_root
                .as_ref()
                .join(model.replace('/', std::path::MAIN_SEPARATOR_STR)),
            configuration_path: crate_root
                .as_ref()
                .join(configuration.replace('/', std::path::MAIN_SEPARATOR_STR)),
            bounds,
        }
    }

    pub const fn protocol(&self) -> ProtocolFamily {
        self.protocol
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn configuration_path(&self) -> &Path {
        &self.configuration_path
    }

    pub const fn bounds(&self) -> ProtocolCheckBounds {
        self.bounds
    }
}

const fn checked_artifact_paths(protocol: ProtocolFamily) -> (&'static str, &'static str) {
    match protocol {
        ProtocolFamily::DurabilityRecovery => (
            "src/protocols/durability_recovery/DurabilityRecovery.tla",
            "src/protocols/durability_recovery/DurabilityRecovery.cfg",
        ),
        ProtocolFamily::RecoverySourcePrecedence => (
            "src/protocols/source_precedence/SourcePrecedence.tla",
            "src/protocols/source_precedence/SourcePrecedence.cfg",
        ),
        ProtocolFamily::CompactionVisibility => (
            "src/protocols/compaction_visibility/CompactionVisibility.tla",
            "src/protocols/compaction_visibility/CompactionVisibility.cfg",
        ),
        ProtocolFamily::LeaseReclaim => (
            "src/protocols/lease_reclaim/LeaseReclaim.tla",
            "src/protocols/lease_reclaim/LeaseReclaim.cfg",
        ),
        ProtocolFamily::QuarantineReadmission => (
            "src/protocols/quarantine_readmission/QuarantineReadmission.tla",
            "src/protocols/quarantine_readmission/QuarantineReadmission.cfg",
        ),
        ProtocolFamily::ImportPublication => (
            "src/protocols/import_publication/ImportPublication.tla",
            "src/protocols/import_publication/ImportPublication.cfg",
        ),
        ProtocolFamily::ReplicationAdmission => (
            "src/protocols/replication_admission/ReplicationAdmission.tla",
            "src/protocols/replication_admission/ReplicationAdmission.cfg",
        ),
        ProtocolFamily::SharedFrontiers => (
            "src/protocols/shared_frontiers/SharedFrontiers.tla",
            "src/protocols/shared_frontiers/SharedFrontiers.cfg",
        ),
    }
}
