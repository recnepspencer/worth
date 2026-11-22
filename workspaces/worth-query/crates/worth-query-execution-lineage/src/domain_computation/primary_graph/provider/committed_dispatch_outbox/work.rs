//! Structural work established by one exact committed-outbox owner read.

/// Bounded structural cost of a successful exact-commit outbox observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedDispatchOutboxReadWork {
    owner: worth_relational::facade::runtime::RelationalRetainedCommitProjectionWork,
}

impl WorthQueryCommittedDispatchOutboxReadWork {
    pub const fn from_owner(
        owner: worth_relational::facade::runtime::RelationalRetainedCommitProjectionWork,
    ) -> Self {
        Self { owner }
    }

    pub const fn exact_commit_snapshots(self) -> usize {
        self.owner.retained_snapshot_probes()
    }

    pub const fn canonical_version_probes(self) -> usize {
        self.owner.canonical_version_probes()
    }

    pub const fn projection_views(self) -> usize {
        self.owner.projection_views()
    }

    pub const fn examined_index_entries(self) -> usize {
        self.owner.examined_index_entries()
    }

    pub const fn direct_record_probes(self) -> usize {
        self.owner.direct_record_probes()
    }

    pub const fn projected_records(self) -> usize {
        self.owner.projected_records()
    }

    pub const fn projected_fields(self) -> usize {
        self.owner.projected_fields()
    }

    pub const fn reconstruction_requests(self) -> usize {
        self.owner.reconstruction_requests()
    }
}
