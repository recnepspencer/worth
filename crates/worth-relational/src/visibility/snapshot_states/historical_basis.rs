use std::sync::Arc;

use crate::branch::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::runtime::RelationalRuntime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoricalVisibilityDenial {
    UnknownVersion,
    AuthoringBranchUnavailable,
    MvccIntervalUnavailable,
    CertificationReconstructionRequired,
    RetentionUnavailable,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
}

#[derive(Clone, Debug)]
pub(crate) struct HistoricalVisibilityBasis {
    branch_id: BranchId,
    version_id: VersionId,
    root: Option<Arc<crate::history::retention::RelationalRetainedHistoricalRoot>>,
    coverage: HistoricalVisibilityCoverage,
}

#[derive(Clone, Debug)]
enum HistoricalVisibilityCoverage {
    EmptyGenesis,
    RetainedInterval {
        source_root_id: u64,
        source_version: VersionId,
    },
}

impl HistoricalVisibilityBasis {
    pub(crate) fn resolve_retained_commit(
        runtime: &RelationalRuntime,
        commit_id: crate::history::data::CommitId,
        branch_id: BranchId,
        version_id: VersionId,
    ) -> Result<Self, HistoricalVisibilityDenial> {
        let retained = runtime
            .history
            .retain_historical_root(commit_id)
            .map_err(historical_retention_denial)?
            .ok_or(HistoricalVisibilityDenial::CertificationReconstructionRequired)?;
        let source_root_id = retained.root().id();
        if retained.root().commit_id() != Some(commit_id) {
            return Err(HistoricalVisibilityDenial::MvccIntervalUnavailable);
        }
        Ok(Self {
            branch_id,
            version_id,
            root: Some(Arc::new(retained)),
            coverage: HistoricalVisibilityCoverage::RetainedInterval {
                source_root_id,
                source_version: version_id,
            },
        })
    }

    pub(crate) fn resolve(
        runtime: &RelationalRuntime,
        version_id: VersionId,
    ) -> Result<Self, HistoricalVisibilityDenial> {
        let branch_id = crate::visibility::branch_scope::branch_for_version(runtime, version_id)
            .ok_or(HistoricalVisibilityDenial::UnknownVersion)?;
        let commit_id = runtime
            .history
            .commit_artifact_for_version(version_id)
            .map(|artifact| artifact.commit_id());
        if let Some(root) = commit_id
            .map(|commit_id| runtime.history.retain_historical_root(commit_id))
            .transpose()
            .map_err(historical_retention_denial)?
            .flatten()
        {
            let source_root_id = root.root().id();
            return Ok(Self {
                branch_id,
                version_id,
                coverage: HistoricalVisibilityCoverage::RetainedInterval {
                    source_root_id,
                    source_version: version_id,
                },
                root: Some(Arc::new(root)),
            });
        }
        let cell = runtime
            .history
            .branch_cell(&branch_id)
            .ok_or(HistoricalVisibilityDenial::AuthoringBranchUnavailable)?;
        let Some(root) = cell.root() else {
            if version_id.is_zero() && runtime.history().latest_commit().is_none() {
                return Ok(Self {
                    branch_id,
                    version_id,
                    root: None,
                    coverage: HistoricalVisibilityCoverage::EmptyGenesis,
                });
            }
            return Err(HistoricalVisibilityDenial::CertificationReconstructionRequired);
        };
        let mut source_version = match root.axes() {
            Some(axes) => VersionId(axes.storage_version),
            None if version_id.is_zero() && root.id() == 0 && root.descriptor().is_none() => {
                VersionId(0)
            }
            None => return Err(HistoricalVisibilityDenial::MvccIntervalUnavailable),
        };
        if source_version.as_u64() < version_id.as_u64() {
            let metadata_aliases_root = runtime
                .history
                .commit_artifact_for_version(version_id)
                .is_some_and(|artifact| envelope_selects_root(artifact.envelope(), &root));
            if !metadata_aliases_root {
                return Err(HistoricalVisibilityDenial::MvccIntervalUnavailable);
            }
            source_version = version_id;
        }
        let source_root_id = root.id();
        let root = crate::history::retention::RelationalRetainedHistoricalRoot::acquire(
            &runtime.history.retention_binding(),
            root,
        )
        .map_err(historical_retention_denial)?;
        Ok(Self {
            branch_id,
            version_id,
            root: Some(Arc::new(root)),
            coverage: HistoricalVisibilityCoverage::RetainedInterval {
                source_root_id,
                source_version,
            },
        })
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub(crate) const fn version_id(&self) -> VersionId {
        self.version_id
    }

    pub(crate) fn root(&self) -> Option<&Arc<RelationalBranchRoot>> {
        self.root.as_ref().map(|retained| retained.root())
    }

    pub(crate) fn source_root_id(&self) -> Option<u64> {
        match self.coverage {
            HistoricalVisibilityCoverage::EmptyGenesis => None,
            HistoricalVisibilityCoverage::RetainedInterval { source_root_id, .. } => {
                Some(source_root_id)
            }
        }
    }

    pub(crate) fn schema_commitment(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|retained| {
            retained
                .root()
                .schema_authority()
                .registry()
                .authority_digest_bytes()
        })
    }

    pub(crate) fn source_version(&self) -> VersionId {
        match self.coverage {
            HistoricalVisibilityCoverage::EmptyGenesis => VersionId(0),
            HistoricalVisibilityCoverage::RetainedInterval { source_version, .. } => source_version,
        }
    }
}

fn historical_retention_denial(
    denial: crate::history::retention::RelationalRetentionAcquisitionDenial,
) -> HistoricalVisibilityDenial {
    match denial {
        crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
            HistoricalVisibilityDenial::RetentionCapacityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
            HistoricalVisibilityDenial::RetentionIdentityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable
        | crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge => {
            HistoricalVisibilityDenial::RetentionUnavailable
        }
    }
}

fn envelope_selects_root(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    root: &RelationalBranchRoot,
) -> bool {
    let Some(descriptor) = root.descriptor() else {
        return false;
    };
    envelope
        .branch_cell_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.observation.target().as_basis())
        .is_some_and(|target| target.roots() == descriptor)
}
