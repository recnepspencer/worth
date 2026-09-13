use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::external_report_paths::resolve_prospective_path;
use super::{CleanRootArtifactManifest, ExternalReportPaths, RootLocalizationCounters};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RootSliceScenario {
    clean_store_root: PathBuf,
    reports: ExternalReportPaths,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FreshRootArtifactRowDenial {
    DestinationExists,
    ReportPathCollision,
    CopyFailed,
    CopiedArtifactMismatch,
}

#[derive(Debug)]
pub(crate) struct FreshRootArtifactRow {
    root: PathBuf,
    baseline_identity: [u8; 32],
}

impl RootSliceScenario {
    pub(crate) fn new(clean_store_root: PathBuf, reports: ExternalReportPaths) -> Self {
        Self {
            clean_store_root,
            reports,
        }
    }

    pub(crate) fn clean_store_root(&self) -> &Path {
        &self.clean_store_root
    }
    pub(crate) const fn reports(&self) -> &ExternalReportPaths {
        &self.reports
    }
}

impl FreshRootArtifactRow {
    pub(crate) fn copy_from(
        scenario: &RootSliceScenario,
        manifest: &CleanRootArtifactManifest,
        destination: PathBuf,
        counters: &mut RootLocalizationCounters,
    ) -> Result<Self, FreshRootArtifactRowDenial> {
        if destination.exists() {
            return Err(FreshRootArtifactRowDenial::DestinationExists);
        }
        let destination = resolve_prospective_path(&destination)
            .map_err(|_| FreshRootArtifactRowDenial::CopyFailed)?;
        if scenario.reports.runtime().starts_with(&destination)
            || scenario.reports.offline().starts_with(&destination)
        {
            return Err(FreshRootArtifactRowDenial::ReportPathCollision);
        }
        std::fs::create_dir_all(&destination)
            .map_err(|_| FreshRootArtifactRowDenial::CopyFailed)?;
        let mut copied_paths = BTreeSet::new();
        let mut copied_bytes = 0_u64;
        for record in manifest.records() {
            for relative in [record.relative_path(), record.substitution_source_path()] {
                if !copied_paths.insert(relative.to_path_buf()) {
                    continue;
                }
                let source = scenario.clean_store_root.join(relative);
                let target = destination.join(relative);
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|_| FreshRootArtifactRowDenial::CopyFailed)?;
                }
                copied_bytes += std::fs::copy(source, target)
                    .map_err(|_| FreshRootArtifactRowDenial::CopyFailed)?;
            }
        }
        for (relative, _) in manifest.supporting_artifacts() {
            if !copied_paths.insert(relative.to_path_buf()) {
                continue;
            }
            let source = scenario.clean_store_root.join(relative);
            let target = destination.join(relative);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|_| FreshRootArtifactRowDenial::CopyFailed)?;
            }
            copied_bytes += std::fs::copy(source, target)
                .map_err(|_| FreshRootArtifactRowDenial::CopyFailed)?;
        }
        let observed_identity = copied_manifest_identity(&destination, manifest)?;
        if observed_identity != manifest.identity() {
            return Err(FreshRootArtifactRowDenial::CopiedArtifactMismatch);
        }
        counters.record_world_copy(copied_paths.len() as u64, copied_bytes);
        Ok(Self {
            root: destination,
            baseline_identity: observed_identity,
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) const fn baseline_identity(&self) -> [u8; 32] {
        self.baseline_identity
    }
}

fn copied_manifest_identity(
    root: &Path,
    manifest: &CleanRootArtifactManifest,
) -> Result<[u8; 32], FreshRootArtifactRowDenial> {
    for record in manifest.records() {
        let bytes = std::fs::read(root.join(record.relative_path()))
            .map_err(|_| FreshRootArtifactRowDenial::CopiedArtifactMismatch)?;
        if Sha256::digest(bytes).as_slice() != record.content_sha256() {
            return Err(FreshRootArtifactRowDenial::CopiedArtifactMismatch);
        }
        let donor = std::fs::read(root.join(record.substitution_source_path()))
            .map_err(|_| FreshRootArtifactRowDenial::CopiedArtifactMismatch)?;
        if Sha256::digest(donor).as_slice() != record.substitution_source_sha256() {
            return Err(FreshRootArtifactRowDenial::CopiedArtifactMismatch);
        }
    }
    for (relative, expected_digest) in manifest.supporting_artifacts() {
        let bytes = std::fs::read(root.join(relative))
            .map_err(|_| FreshRootArtifactRowDenial::CopiedArtifactMismatch)?;
        if Sha256::digest(bytes).as_slice() != expected_digest {
            return Err(FreshRootArtifactRowDenial::CopiedArtifactMismatch);
        }
    }
    Ok(manifest.identity())
}
