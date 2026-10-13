use crate::history::data::RelationalCommitReceipt;
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::{SnapshotHandle, SnapshotReadPolicy};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalRetainedCommitSnapshotDenialKind {
    Execution(crate::execution::RelationalExecutionDenialCause),
    ForeignRuntime,
    VersionUnavailable,
    BranchMismatch,
    CommitMismatch,
    SnapshotNotRetained,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    SnapshotBindingMismatch,
    EntityKindMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalRetainedCommitSnapshotDenial {
    kind: RelationalRetainedCommitSnapshotDenialKind,
    detail: &'static str,
}

impl RelationalRetainedCommitSnapshotDenial {
    pub(crate) fn new(
        kind: RelationalRetainedCommitSnapshotDenialKind,
        detail: &'static str,
    ) -> Self {
        Self { kind, detail }
    }

    pub const fn kind(&self) -> RelationalRetainedCommitSnapshotDenialKind {
        self.kind
    }

    pub const fn detail(&self) -> &'static str {
        self.detail
    }
}

/// Read-only evidence that one exact canonical commit still has a published
/// snapshot handle in this runtime.
///
/// This observation does not own a release lease. Its borrowed handle is for
/// immediate use with ordinary visibility APIs.
#[derive(Debug, Eq, PartialEq)]
pub struct RelationalRetainedCommitSnapshot {
    commit: RelationalCommitReceipt,
    snapshot_handle: SnapshotHandle,
    work: RelationalRetainedCommitProjectionWork,
}

/// Operation-local structural evidence for an exact retained-commit projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationalRetainedCommitProjectionWork {
    canonical_version_probes: usize,
    retained_snapshot_probes: usize,
    projection_views: usize,
    direct_record_probes: usize,
    projected_records: usize,
    projected_fields: usize,
    examined_index_entries: usize,
    reconstruction_requests: usize,
}

pub struct RelationalRetainedCommitEntityProjection<T> {
    value: Option<T>,
    work: RelationalRetainedCommitProjectionWork,
}

impl RelationalRetainedCommitSnapshot {
    pub const fn commit(&self) -> &RelationalCommitReceipt {
        &self.commit
    }

    pub const fn snapshot_handle(&self) -> &SnapshotHandle {
        &self.snapshot_handle
    }

    pub const fn work(&self) -> RelationalRetainedCommitProjectionWork {
        self.work
    }
}

impl RelationalRetainedCommitProjectionWork {
    pub(crate) const fn opened_snapshot() -> Self {
        Self {
            canonical_version_probes: 1,
            retained_snapshot_probes: 1,
            projection_views: 0,
            direct_record_probes: 0,
            projected_records: 0,
            projected_fields: 0,
            examined_index_entries: 0,
            reconstruction_requests: 0,
        }
    }

    pub const fn canonical_version_probes(self) -> usize {
        self.canonical_version_probes
    }

    pub const fn retained_snapshot_probes(self) -> usize {
        self.retained_snapshot_probes
    }

    pub const fn projection_views(self) -> usize {
        self.projection_views
    }

    pub const fn direct_record_probes(self) -> usize {
        self.direct_record_probes
    }

    pub const fn projected_records(self) -> usize {
        self.projected_records
    }

    pub const fn projected_fields(self) -> usize {
        self.projected_fields
    }

    pub const fn examined_index_entries(self) -> usize {
        self.examined_index_entries
    }

    pub const fn reconstruction_requests(self) -> usize {
        self.reconstruction_requests
    }

    pub(crate) const fn record_projection(
        mut self,
        projected_records: usize,
        projected_fields: usize,
    ) -> Self {
        self.projection_views += 1;
        self.direct_record_probes += 1;
        self.projected_records += projected_records;
        self.projected_fields += projected_fields;
        self
    }
}

impl<T> RelationalRetainedCommitEntityProjection<T> {
    pub(crate) const fn new(
        value: Option<T>,
        work: RelationalRetainedCommitProjectionWork,
    ) -> Self {
        Self { value, work }
    }

    pub fn into_parts(self) -> (Option<T>, RelationalRetainedCommitProjectionWork) {
        (self.value, self.work)
    }
}

pub(crate) fn open_retained_commit_snapshot(
    runtime: &RelationalRuntime,
    expected_runtime_instance_id: u64,
    requested_commit: &RelationalCommitReceipt,
) -> Result<RelationalRetainedCommitSnapshot, RelationalRetainedCommitSnapshotDenial> {
    if expected_runtime_instance_id != runtime.runtime_instance_id() {
        return Err(denial(
            RelationalRetainedCommitSnapshotDenialKind::ForeignRuntime,
            "retained commit snapshot belongs to a different runtime instance",
        ));
    }

    let Some(committed) = runtime
        .history()
        .committed_version(requested_commit.version_id)
    else {
        return Err(denial(
            RelationalRetainedCommitSnapshotDenialKind::VersionUnavailable,
            "requested version has no canonical commit in this runtime",
        ));
    };
    let canonical_commit = committed.commit();
    if canonical_commit.branch_id != requested_commit.branch_id {
        return Err(denial(
            RelationalRetainedCommitSnapshotDenialKind::BranchMismatch,
            "requested branch does not own the canonical commit version",
        ));
    }
    if canonical_commit != requested_commit {
        return Err(denial(
            RelationalRetainedCommitSnapshotDenialKind::CommitMismatch,
            "requested commit reference differs from canonical history",
        ));
    }

    let Some((snapshot_id, binding)) = runtime
        .visibility
        .published_snapshot_binding_for_version(requested_commit.version_id)
    else {
        return Err(denial(
            RelationalRetainedCommitSnapshotDenialKind::SnapshotNotRetained,
            "canonical commit no longer has an already-retained published snapshot",
        ));
    };
    if binding.branch_id != requested_commit.branch_id
        || binding.version_id != requested_commit.version_id
        || binding.read_policy != SnapshotReadPolicy::ImmutablePinnedNoLazyMutation
    {
        return Err(denial(
            RelationalRetainedCommitSnapshotDenialKind::SnapshotBindingMismatch,
            "published snapshot binding disagrees with canonical commit identity",
        ));
    }

    Ok(RelationalRetainedCommitSnapshot {
        commit: canonical_commit.clone(),
        snapshot_handle: SnapshotHandle {
            runtime_instance_id: runtime.runtime_instance_id(),
            branch_id: binding.branch_id,
            snapshot_id,
            version_id: binding.version_id,
            read_policy: binding.read_policy,
        },
        work: RelationalRetainedCommitProjectionWork::opened_snapshot(),
    })
}

pub(crate) fn projection_binding_denial() -> RelationalRetainedCommitSnapshotDenial {
    denial(
        RelationalRetainedCommitSnapshotDenialKind::SnapshotBindingMismatch,
        "retained commit snapshot could not open its bound projection view",
    )
}

fn denial(
    kind: RelationalRetainedCommitSnapshotDenialKind,
    detail: &'static str,
) -> RelationalRetainedCommitSnapshotDenial {
    RelationalRetainedCommitSnapshotDenial::new(kind, detail)
}
