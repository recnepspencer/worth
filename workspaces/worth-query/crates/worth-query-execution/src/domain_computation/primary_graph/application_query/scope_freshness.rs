//! Descriptive causes of refused scope freshness; never snapshot authority.
use super::super::WorthQueryEntityResolutionDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryScopeFreshnessCheck {
    CurrentSecurity,
    SelectedTruth,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryScopeFreshnessDiagnostic {
    pub check: WorthQueryScopeFreshnessCheck,
    pub cause: WorthQueryEntityResolutionDenial,
    pub runtime_instance_id: u64,
    pub snapshot_id: worth_relational::facade::snapshots::SnapshotId,
    pub version_id: worth_relational::facade::identity::VersionId,
}

impl WorthQueryScopeFreshnessDiagnostic {
    pub(super) fn new(
        check: WorthQueryScopeFreshnessCheck,
        cause: WorthQueryEntityResolutionDenial,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Self {
        Self {
            check,
            cause,
            runtime_instance_id: snapshot.runtime_instance_id(),
            snapshot_id: snapshot.snapshot_id(),
            version_id: snapshot.version_id(),
        }
    }
}
