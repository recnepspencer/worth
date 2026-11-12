//! Retire the original indexed observation without destroying its payload.
use super::super::{
    RelationalBridgeBasisRetirementDenial as Denial,
    RelationalBridgeBasisRetirementProgress as Progress, RuntimeBridgeRelationalSource,
};
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::change_source::{
    RelationalRetainedObservationRetirement, RelationalRetainedObservationRetirementProgress,
    RetiredRelationalRetainedObservation,
};

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(in crate::relational_source) use tests::{
    assert_observation_busy_and_indexes, assert_observation_index_refusals,
};

/// Move-only original registration and Native custody. Refusals retain the
/// same unfinished task; abandoning it invokes unbudgeted ordinary cleanup.
#[derive(Debug)]
#[must_use]
pub struct RelationalBridgeObservationRetirement {
    lease: RelationalBridgeObservationLease,
    issuer: Option<Arc<RelationalBridgeObservationBindings>>,
    removed: Option<(SnapshotId, RelationalBridgeObservationBinding)>,
    _removed_subject: Option<(SnapshotId, VersionId, Option<CommitId>)>,
    _removed_commit: Option<(CommitId, BTreeSet<SnapshotId>)>,
    _removed_member: Option<SnapshotId>,
    native: Option<RelationalRetainedObservationRetirement>,
    retired_native: Option<RetiredRelationalRetainedObservation>,
}

/// Detached local custody; final destruction and abandonment are unbudgeted.
/// Other registrations and independently opened readers remain protected.
#[derive(Debug)]
#[must_use]
pub struct RetiredRelationalBridgeObservation {
    _custody: RelationalBridgeObservationRetirement,
}

impl RelationalBridgeObservationLease {
    /// Consume the actual registration without allocation or release.
    pub fn into_retirement(self) -> RelationalBridgeObservationRetirement {
        RelationalBridgeObservationRetirement {
            lease: self,
            issuer: None,
            removed: None,
            _removed_subject: None,
            _removed_commit: None,
            _removed_member: None,
            native: None,
            retired_native: None,
        }
    }

    pub(in crate::relational_source::runtime_source) fn issued_from_source(
        &self,
        source: &RuntimeBridgeRelationalSource,
    ) -> bool {
        self.bindings
            .as_ref()
            .is_some_and(|issuer| Arc::ptr_eq(issuer, &source.observation_bindings))
    }
}

impl RelationalBridgeObservationRetirement {
    /// Advance one original stage. No Bridge gate is held during Native cleanup.
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &RuntimeBridgeRelationalSource,
    ) -> Progress {
        if self.retired_native.is_some() {
            return Progress::Complete;
        }
        if !self.issued_from_source(source) {
            return Progress::Refused(Denial::ForeignSource);
        }
        if let Err(cause) = request.consult() {
            return Progress::Stopped(cause);
        }
        if self.lease.bindings.is_some() {
            return self.detach(request);
        }
        if let Some((snapshot, binding)) = self.removed.take() {
            self._removed_subject = Some((snapshot, binding.version_id, binding.commit_id));
            self.native = Some(binding.retained.into_retirement());
            return Progress::Advanced;
        }
        let Some(native) = self.native.as_mut() else {
            return Progress::Refused(Denial::RegistrationMismatch);
        };
        let progress = native.advance(request);
        if progress != RelationalRetainedObservationRetirementProgress::Complete {
            return progress.into();
        }
        let native = self.native.take().expect(/* Proof: The preceding mutable borrow and completed advance addressed this same installed task. */ "completed Native task remains owned");
        match native.into_retired() {
            Ok(retired) => {
                self.retired_native = Some(retired);
                Progress::Complete
            }
            Err(task) => {
                self.native = Some(task);
                Progress::Refused(Denial::RegistrationMismatch)
            }
        }
    }

    /// Preserve unfinished obligations rather than issuing retired custody.
    pub fn into_retired(self) -> Result<RetiredRelationalBridgeObservation, Self> {
        if self.retired_native.is_some() {
            Ok(RetiredRelationalBridgeObservation { _custody: self })
        } else {
            Err(self)
        }
    }

    pub(in crate::relational_source::runtime_source) fn issued_from_source(
        &self,
        source: &RuntimeBridgeRelationalSource,
    ) -> bool {
        self.lease
            .bindings
            .as_ref()
            .or(self.issuer.as_ref())
            .is_some_and(|issuer| Arc::ptr_eq(issuer, &source.observation_bindings))
    }

    fn detach(&mut self, request: &ExecutionRequest<'_, '_>) -> Progress {
        let issuer = self.lease.bindings.as_ref().expect(
            /* Proof: advance selects this stage only while the original lease owns its issuer. */
            "original issuer remains owned",
        );
        let mut index = match issuer.entries.try_lock() {
            Ok(value) => value,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Progress::Busy,
        };
        let Some(parts) = self.lease.snapshot_identity.relational_snapshot_parts() else {
            return Progress::Refused(Denial::RegistrationMismatch);
        };
        let snapshot = self.lease.snapshot_id;
        let Some(binding) = index.by_snapshot.get(&snapshot) else {
            return Progress::Refused(Denial::RegistrationMismatch);
        };
        if parts.snapshot_id() != snapshot.0
            || parts.version_id() != binding.version_id.0
            || binding.retained.snapshot_id() != snapshot
            || binding.retained.observation().version_id() != binding.version_id
            || binding.retained.observation().commit_id() != binding.commit_id
        {
            return Progress::Refused(Denial::RegistrationMismatch);
        }
        let commit = binding.commit_id;
        if let Some(commit) = commit
            && !index
                .by_commit
                .get(&commit)
                .is_some_and(|members| members.contains(&snapshot))
        {
            return Progress::Refused(Denial::CommitIndexMismatch);
        }
        if let Err(cause) = request.consult() {
            return Progress::Stopped(cause);
        }
        // Exact point removal cannot allocate or destroy the moved payloads.
        self.removed = index.by_snapshot.remove_entry(&snapshot);
        if let Some(commit) = commit {
            let members = index.by_commit.get_mut(&commit).expect(
                /* Proof: The same gate validated this exact commit membership before removal. */
                "validated commit index remains present",
            );
            self._removed_member = members.take(&snapshot);
            if members.is_empty() {
                self._removed_commit = index.by_commit.remove_entry(&commit);
            }
        }
        drop(index);
        self.issuer = self.lease.bindings.take();
        Progress::Advanced
    }
}

impl RetiredRelationalBridgeObservation {
    /// Destroy original allocations only outside request-controlled work.
    pub fn reclaim(self) {
        drop(self);
    }
}
