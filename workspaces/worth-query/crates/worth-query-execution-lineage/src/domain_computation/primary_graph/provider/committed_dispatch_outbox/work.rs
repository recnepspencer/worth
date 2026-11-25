//! Structural work established by one exact committed-outbox owner read.

/// Bounded structural cost of a successful exact-commit outbox observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedDispatchOutboxReadWork {
    kind: OutboxReadWorkKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutboxReadWorkKind {
    Retained(worth_relational::facade::runtime::RelationalRetainedCommitProjectionWork),
    RecoveredSource,
}

impl WorthQueryCommittedDispatchOutboxReadWork {
    pub const fn from_owner(
        owner: worth_relational::facade::runtime::RelationalRetainedCommitProjectionWork,
    ) -> Self {
        Self {
            kind: OutboxReadWorkKind::Retained(owner),
        }
    }

    pub const fn from_recovered_source() -> Self {
        Self {
            kind: OutboxReadWorkKind::RecoveredSource,
        }
    }

    pub const fn exact_commit_snapshots(self) -> usize {
        match self.kind {
            OutboxReadWorkKind::Retained(owner) => owner.retained_snapshot_probes(),
            OutboxReadWorkKind::RecoveredSource => 0,
        }
    }

    pub const fn canonical_version_probes(self) -> usize {
        match self.kind {
            OutboxReadWorkKind::Retained(owner) => owner.canonical_version_probes(),
            OutboxReadWorkKind::RecoveredSource => 1,
        }
    }

    pub const fn projection_views(self) -> usize {
        match self.kind {
            OutboxReadWorkKind::Retained(owner) => owner.projection_views(),
            OutboxReadWorkKind::RecoveredSource => 0,
        }
    }

    pub const fn examined_index_entries(self) -> usize {
        match self.kind {
            OutboxReadWorkKind::Retained(owner) => owner.examined_index_entries(),
            OutboxReadWorkKind::RecoveredSource => 0,
        }
    }

    pub const fn direct_record_probes(self) -> usize {
        match self.kind {
            OutboxReadWorkKind::Retained(owner) => owner.direct_record_probes(),
            OutboxReadWorkKind::RecoveredSource => 0,
        }
    }

    pub const fn projected_records(self) -> usize {
        match self.kind {
            OutboxReadWorkKind::Retained(owner) => owner.projected_records(),
            OutboxReadWorkKind::RecoveredSource => 0,
        }
    }

    pub const fn projected_fields(self) -> usize {
        match self.kind {
            OutboxReadWorkKind::Retained(owner) => owner.projected_fields(),
            OutboxReadWorkKind::RecoveredSource => 0,
        }
    }

    pub const fn reconstruction_requests(self) -> usize {
        match self.kind {
            OutboxReadWorkKind::Retained(owner) => owner.reconstruction_requests(),
            OutboxReadWorkKind::RecoveredSource => 0,
        }
    }
}
