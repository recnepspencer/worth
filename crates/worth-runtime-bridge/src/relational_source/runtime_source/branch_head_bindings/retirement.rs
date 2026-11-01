//! Consume this head tuple and then its own observation registration.
use super::super::{
    RelationalBridgeBasisRetirementDenial as Denial,
    RelationalBridgeBasisRetirementProgress as Progress, RelationalBridgeObservationRetirement,
    RetiredRelationalBridgeObservation, RuntimeBridgeRelationalSource,
};
use super::*;
use worth_execution::ExecutionRequest;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(in crate::relational_source) use tests::assert_wrapped_successor_survives_abandonment;

/// Move-only original head tuple and its observation retirement. Abandonment
/// joins the same tuple-matched cleanup and is explicitly unbudgeted.
#[derive(Debug)]
#[must_use]
pub struct RelationalBridgeBranchHeadRetirement {
    lease: RelationalBridgeBranchHeadLease,
    issuer: Option<Arc<RelationalBridgeBranchHeadBindings>>,
    _removed: Option<(TruthBranchIdentity, RelationalBridgeBranchHeadBinding)>,
    observation: Option<RelationalBridgeObservationRetirement>,
    retired_observation: Option<RetiredRelationalBridgeObservation>,
}

/// Original detached head and observation custody. This retains no source or
/// RuntimeHandle; destruction and unfinished task abandonment are unbudgeted.
#[derive(Debug)]
#[must_use]
pub struct RetiredRelationalBridgeBranchHead {
    _custody: RelationalBridgeBranchHeadRetirement,
}

impl RelationalBridgeBranchHeadLease {
    /// Consume the actual lease without releasing or cloning its custody.
    pub fn into_retirement(self) -> RelationalBridgeBranchHeadRetirement {
        RelationalBridgeBranchHeadRetirement {
            lease: self,
            issuer: None,
            _removed: None,
            observation: None,
            retired_observation: None,
        }
    }
}

impl RelationalBridgeBranchHeadRetirement {
    /// Retire the original head tuple, preserving any successor. Both original
    /// registry issuers are checked on every live call, before any effect.
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &RuntimeBridgeRelationalSource,
    ) -> Progress {
        if self.retired_observation.is_some() {
            return Progress::Complete;
        }
        if !self.issued_from_source(source) {
            return Progress::Refused(Denial::ForeignSource);
        }
        if let Err(cause) = request.consult() {
            return Progress::Stopped(cause);
        }
        if self.lease.bindings.is_some() {
            return self.detach_head(request);
        }
        if self.observation.is_none() {
            let original = self.lease.observation.take().expect(/* Proof: The issued non-Clone head owns its observation; only this one-time stage consumes it after head detachment. */ "original observation remains owned");
            self.observation = Some(original.into_retirement());
            return Progress::Advanced;
        }
        let observation = self.observation.as_mut().expect(/* Proof: The preceding branch installs this task and returns; subsequent stages keep that same task. */ "original observation retirement remains owned");
        let progress = observation.advance(request, source);
        if progress != Progress::Complete {
            return progress;
        }
        let observation = self.observation.take().expect(/* Proof: The preceding borrow and completed advance addressed this same installed task. */ "completed observation task remains owned");
        match observation.into_retired() {
            Ok(retired) => {
                self.retired_observation = Some(retired);
                Progress::Complete
            }
            Err(task) => {
                self.observation = Some(task);
                Progress::Refused(Denial::RegistrationMismatch)
            }
        }
    }

    /// Preserve unfinished obligations rather than issuing retired custody.
    pub fn into_retired(self) -> Result<RetiredRelationalBridgeBranchHead, Self> {
        if self.retired_observation.is_some() {
            Ok(RetiredRelationalBridgeBranchHead { _custody: self })
        } else {
            Err(self)
        }
    }

    fn issued_from_source(&self, source: &RuntimeBridgeRelationalSource) -> bool {
        let head_matches = self
            .lease
            .bindings
            .as_ref()
            .or(self.issuer.as_ref())
            .is_some_and(|issuer| Arc::ptr_eq(issuer, &source.branch_head_bindings));
        let observation_matches = match (&self.lease.observation, &self.observation) {
            (Some(lease), _) => lease.issued_from_source(source),
            (_, Some(task)) => task.issued_from_source(source),
            _ => false,
        };
        head_matches && observation_matches
    }

    fn detach_head(&mut self, request: &ExecutionRequest<'_, '_>) -> Progress {
        let issuer = self.lease.bindings.as_ref().expect(/* Proof: advance selects this stage only while the original head lease owns its issuer. */ "original head issuer remains owned");
        let mut entries = match issuer.entries.try_lock() {
            Ok(value) => value,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Progress::Busy,
        };
        let matches = entries
            .get(&self.lease.branch_identity)
            .is_some_and(|binding| {
                binding.binding_id == self.lease.binding_id
                    && &binding.snapshot_identity == self.lease.snapshot_identity()
            });
        if let Err(cause) = request.consult() {
            return Progress::Stopped(cause);
        }
        if matches {
            self._removed = entries.remove_entry(&self.lease.branch_identity);
        }
        drop(entries);
        self.issuer = self.lease.bindings.take();
        Progress::Advanced
    }
}

impl RetiredRelationalBridgeBranchHead {
    /// Destroy original allocations only outside request-controlled work.
    pub fn reclaim(self) {
        drop(self);
    }
}
