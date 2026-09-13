use std::path::PathBuf;

use worth_store_physical_backend::{
    OfflineMediaClosureEntry, OfflineMediaConsistencyBasis, OfflineMediaConsistencyBasisDenial,
    OfflineMediaReadDenial, ReadOnlyOfflineMediaCapability,
};
use worth_store_physical_isolation::{
    BootstrapSourceArtifact, BootstrapSourceArtifactFamily, BootstrapSourceEvidenceBinding,
    BootstrapSourceFrontier, BootstrapSourceResolutionDenial, BootstrapSourceResolutionRequest,
    PhysicalIsolationBootstrapSourceOwner, ResolvedBootstrapRecoverySourceCut,
};
use worth_store_replication::DisasterRecoveryComponentFamily;

use super::IndependentlyVerifiedDisasterRecoveryBundle;

#[derive(Debug)]
pub enum BootstrapSourceCutResolutionDenial {
    RootUnavailable,
    ComponentUnavailable,
    ConsistencyBasis(OfflineMediaConsistencyBasisDenial),
    Media(OfflineMediaReadDenial),
    Recovery(BootstrapSourceResolutionDenial),
}

impl From<OfflineMediaReadDenial> for BootstrapSourceCutResolutionDenial {
    fn from(value: OfflineMediaReadDenial) -> Self {
        Self::Media(value)
    }
}

impl From<BootstrapSourceResolutionDenial> for BootstrapSourceCutResolutionDenial {
    fn from(value: BootstrapSourceResolutionDenial) -> Self {
        Self::Recovery(value)
    }
}

impl IndependentlyVerifiedDisasterRecoveryBundle {
    pub fn resolve_bootstrap_source_cut(
        &self,
        operation_identity: [u8; 32],
        resident_buffer_bytes: usize,
        maximum_owned_allocation_bytes: u64,
    ) -> Result<ResolvedBootstrapRecoverySourceCut, BootstrapSourceCutResolutionDenial> {
        let root = std::fs::canonicalize(self.materialized().root())
            .map_err(|_| BootstrapSourceCutResolutionDenial::RootUnavailable)?;
        let artifacts = self
            .materialized()
            .components()
            .iter()
            .map(|component| {
                BootstrapSourceArtifact::declare(
                    source_family(component.family()),
                    component.relative_path(),
                    component.byte_length(),
                    component.expected_digest(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let closure_entries = artifacts
            .iter()
            .map(|artifact| {
                let path = std::fs::canonicalize(root.join(artifact.relative_path()))
                    .map_err(|_| BootstrapSourceCutResolutionDenial::ComponentUnavailable)?;
                OfflineMediaClosureEntry::new(
                    path,
                    artifact.byte_length(),
                    artifact.content_digest(),
                )
                .ok_or(BootstrapSourceCutResolutionDenial::ComponentUnavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let paths = closure_entries
            .iter()
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<PathBuf>>();
        let basis = OfflineMediaConsistencyBasis::content_addressed_closure_from_owned_entries(
            closure_identity(self.materialized().manifest_identity()),
            closure_entries,
        )
        .map_err(BootstrapSourceCutResolutionDenial::ConsistencyBasis)?;
        let media = ReadOnlyOfflineMediaCapability::open_bounded_from_owned_paths(
            paths,
            basis,
            maximum_owned_allocation_bytes,
        )?;
        let frontier = self.materialized().frontier();
        let request = BootstrapSourceResolutionRequest::from_independent_verification(
            operation_identity,
            BootstrapSourceEvidenceBinding::from_independent_verification(
                self.materialized().manifest_identity(),
                self.verification_identity(),
                self.materialized().source_lineage().stable_fingerprint(),
            )?,
            root,
            BootstrapSourceFrontier::admit(
                frontier.observed_lsn(),
                frontier.durable_lsn(),
                frontier.client_acknowledged_lsn(),
                frontier.replication_acknowledged_lsn(),
                frontier.authority_epoch(),
            )?,
            artifacts,
        )?;
        PhysicalIsolationBootstrapSourceOwner::resolve(request, media, resident_buffer_bytes)
            .map_err(Into::into)
    }
}

const fn source_family(family: DisasterRecoveryComponentFamily) -> BootstrapSourceArtifactFamily {
    match family {
        DisasterRecoveryComponentFamily::Authority => BootstrapSourceArtifactFamily::Authority,
        DisasterRecoveryComponentFamily::Checkpoint => BootstrapSourceArtifactFamily::Checkpoint,
        DisasterRecoveryComponentFamily::Wal => BootstrapSourceArtifactFamily::Wal,
        DisasterRecoveryComponentFamily::Page => BootstrapSourceArtifactFamily::Page,
        DisasterRecoveryComponentFamily::Blob => BootstrapSourceArtifactFamily::Blob,
        DisasterRecoveryComponentFamily::Layout => BootstrapSourceArtifactFamily::Layout,
    }
}

fn closure_identity(identity: [u8; 32]) -> String {
    let mut encoded = String::with_capacity(64);
    for byte in identity {
        use std::fmt::Write;
        write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}
