//! Disassemble the actual closed reader before consuming its own registration.
use super::super::{
    RelationalBridgeBasisRetirementDenial, RelationalBridgeBasisRetirementProgress,
    RelationalBridgeObservationRetirement, RetiredRelationalBridgeObservation,
};
use super::*;
use crate::relational_source::snapshot_reading::{
    RetiredRuntimePublicationSnapshotReader, RuntimePublicationSnapshotReaderRetirement,
};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::change_source::RelationalRuntimeHandleAliasRetirementDenial;
use worth_relational::facade::runtime::RelationalRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBridgeRetainedSnapshotRetirementDenial {
    Binding(RelationalBridgeBasisRetirementDenial),
    RuntimeAlias(RelationalRuntimeHandleAliasRetirementDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBridgeRetainedSnapshotRetirementProgress {
    Advanced,
    Busy,
    Stopped(WorkCeilingDenial),
    Refused(RelationalBridgeRetainedSnapshotRetirementDenial),
    Complete,
}

/// The original concrete reader and lease. Abandonment is unbudgeted; field
/// order releases the reader observation BEFORE its untouched raw lease.
#[derive(Debug)]
#[must_use]
pub struct RelationalBridgeRetainedSnapshotRetirement {
    reader: RuntimePublicationSnapshotReaderRetirement,
    lease: Option<RelationalBridgeObservationLease>,
    lease_task: Option<RelationalBridgeObservationRetirement>,
    retired_lease: Option<RetiredRelationalBridgeObservation>,
}

/// Passive original reader metadata, discharged alias and retired local lease.
/// This proves neither whole-source nor Native business retirement.
#[derive(Debug)]
#[must_use]
pub struct RetiredRelationalBridgeRetainedSnapshot {
    _reader: RetiredRuntimePublicationSnapshotReader,
    _lease: RetiredRelationalBridgeObservation,
}

impl RelationalBridgeRetainedSnapshot {
    /// Pure admission to disassembly. Refusal returns this SAME usable snapshot.
    pub fn try_into_retirement(
        self,
        source: &RuntimeBridgeRelationalSource,
        owner: &RelationalRuntime,
    ) -> Result<
        RelationalBridgeRetainedSnapshotRetirement,
        (Self, RelationalBridgeRetainedSnapshotRetirementDenial),
    > {
        if !self._lease.issued_from_source(source) {
            return Err((
                self,
                RelationalBridgeRetainedSnapshotRetirementDenial::Binding(
                    RelationalBridgeBasisRetirementDenial::ForeignSource,
                ),
            ));
        }
        let valid = source
            .runtime
            .validate_immutable_owner_alias(owner)
            .and_then(|()| self.reader.validate_alias_retirement(owner));
        if let Err(cause) = valid {
            return Err((
                self,
                RelationalBridgeRetainedSnapshotRetirementDenial::RuntimeAlias(cause),
            ));
        }
        let Self { reader, _lease } = self;
        Ok(RelationalBridgeRetainedSnapshotRetirement {
            reader: reader.into_retirement_parts(),
            lease: Some(_lease),
            lease_task: None,
            retired_lease: None,
        })
    }
}

impl RelationalBridgeRetainedSnapshotRetirement {
    /// Advance one original stage, borrowing the same request and actual owners.
    /// No Source or RuntimeHandle can escape this phase.
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &RuntimeBridgeRelationalSource,
        owner: &RelationalRuntime,
    ) -> RelationalBridgeRetainedSnapshotRetirementProgress {
        use RelationalBridgeRetainedSnapshotRetirementProgress as Progress;
        if self.retired_lease.is_some() {
            return Progress::Complete;
        }
        if !self.issued_from_source(source) {
            return binding_refusal(RelationalBridgeBasisRetirementDenial::ForeignSource);
        }
        let valid = source
            .runtime
            .validate_immutable_owner_alias(owner)
            .and_then(|()| self.reader.validate_alias_retirement(owner));
        if let Err(cause) = valid {
            return alias_refusal(cause);
        }
        if let Err(cause) = request.consult() {
            return Progress::Stopped(cause);
        }
        if !self.reader.alias_retired() {
            return match self.reader.retire_runtime_alias(request, owner) {
                Ok(()) => Progress::Advanced,
                Err(cause) => alias_refusal(cause),
            };
        }
        if !self.reader.observation_released() {
            if self.lease.is_none() {
                return binding_refusal(
                    RelationalBridgeBasisRetirementDenial::RegistrationMismatch,
                );
            }
            self.reader.release_observation_before_lease();
            return Progress::Advanced;
        }
        if let Some(lease) = self.lease.take() {
            self.lease_task = Some(lease.into_retirement());
            return Progress::Advanced;
        }
        let Some(task) = self.lease_task.as_mut() else {
            return binding_refusal(RelationalBridgeBasisRetirementDenial::RegistrationMismatch);
        };
        match task.advance(request, source) {
            RelationalBridgeBasisRetirementProgress::Advanced => Progress::Advanced,
            RelationalBridgeBasisRetirementProgress::Busy => Progress::Busy,
            RelationalBridgeBasisRetirementProgress::Stopped(cause) => Progress::Stopped(cause),
            RelationalBridgeBasisRetirementProgress::Refused(cause) => binding_refusal(cause),
            RelationalBridgeBasisRetirementProgress::Complete => self.retain_completed_lease(),
        }
    }

    /// Preserve the same unfinished task rather than issue retired custody.
    pub fn into_retired(self) -> Result<RetiredRelationalBridgeRetainedSnapshot, Self> {
        if self.retired_lease.is_none() || self.lease.is_some() || self.lease_task.is_some() {
            return Err(self);
        }
        let Self {
            reader,
            lease,
            lease_task,
            retired_lease,
        } = self;
        match reader.into_retired() {
            Ok(reader) => Ok(RetiredRelationalBridgeRetainedSnapshot { _reader: reader, _lease: retired_lease.expect(/* Proof: The completed lease was checked before destructuring this same task. */ "retired lease remains owned") }),
            Err(reader) => Err(Self { reader, lease, lease_task, retired_lease }),
        }
    }

    fn issued_from_source(&self, source: &RuntimeBridgeRelationalSource) -> bool {
        match (&self.lease, &self.lease_task) {
            (Some(lease), _) => lease.issued_from_source(source),
            (_, Some(task)) => task.issued_from_source(source),
            _ => false,
        }
    }

    fn retain_completed_lease(&mut self) -> RelationalBridgeRetainedSnapshotRetirementProgress {
        let Some(task) = self.lease_task.take() else {
            return binding_refusal(RelationalBridgeBasisRetirementDenial::RegistrationMismatch);
        };
        match task.into_retired() {
            Ok(retired) => {
                self.retired_lease = Some(retired);
                RelationalBridgeRetainedSnapshotRetirementProgress::Complete
            }
            Err(task) => {
                self.lease_task = Some(task);
                binding_refusal(RelationalBridgeBasisRetirementDenial::RegistrationMismatch)
            }
        }
    }
}

impl RetiredRelationalBridgeRetainedSnapshot {
    /// Reclaim original metadata and data only outside request-controlled work.
    pub fn reclaim(self) {
        drop(self);
    }
}

fn alias_refusal(
    cause: RelationalRuntimeHandleAliasRetirementDenial,
) -> RelationalBridgeRetainedSnapshotRetirementProgress {
    match cause {
        RelationalRuntimeHandleAliasRetirementDenial::Execution(cause) => {
            RelationalBridgeRetainedSnapshotRetirementProgress::Stopped(cause)
        }
        cause => RelationalBridgeRetainedSnapshotRetirementProgress::Refused(
            RelationalBridgeRetainedSnapshotRetirementDenial::RuntimeAlias(cause),
        ),
    }
}

fn binding_refusal(
    cause: RelationalBridgeBasisRetirementDenial,
) -> RelationalBridgeRetainedSnapshotRetirementProgress {
    RelationalBridgeRetainedSnapshotRetirementProgress::Refused(
        RelationalBridgeRetainedSnapshotRetirementDenial::Binding(cause),
    )
}
