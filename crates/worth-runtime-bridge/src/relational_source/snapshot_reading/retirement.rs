//! Original reader parts, ordered so abandonment releases observation before lease.
use super::*;
use worth_relational::facade::change_source::{
    RelationalRuntimeHandleAliasRetirementDenial, RetiredRelationalRuntimeHandleAlias,
};

#[derive(Debug)]
pub(in crate::relational_source) struct RuntimePublicationSnapshotReaderRetirement {
    observation: Option<RelationalBranchObservation>,
    runtime: Option<RelationalRuntimeHandle>,
    _snapshot_identity: TruthSnapshotIdentity,
    _partition: Option<PartitionId>,
    alias: Option<RetiredRelationalRuntimeHandleAlias>,
}

#[derive(Debug)]
pub(in crate::relational_source) struct RetiredRuntimePublicationSnapshotReader {
    _snapshot_identity: TruthSnapshotIdentity,
    _partition: Option<PartitionId>,
    _alias: RetiredRelationalRuntimeHandleAlias,
}

impl RuntimePublicationSnapshotReader {
    pub(in crate::relational_source) fn validate_alias_retirement(
        &self,
        owner: &RelationalRuntime,
    ) -> Result<(), RelationalRuntimeHandleAliasRetirementDenial> {
        self.runtime.validate_immutable_owner_alias(owner)
    }

    pub(in crate::relational_source) fn into_retirement_parts(
        self,
    ) -> RuntimePublicationSnapshotReaderRetirement {
        let Self {
            runtime,
            snapshot_identity,
            observation,
            partition,
        } = self;
        RuntimePublicationSnapshotReaderRetirement {
            observation: Some(observation),
            runtime: Some(runtime),
            _snapshot_identity: snapshot_identity,
            _partition: partition,
            alias: None,
        }
    }
}

impl RuntimePublicationSnapshotReaderRetirement {
    pub(in crate::relational_source) fn into_retired(
        self,
    ) -> Result<RetiredRuntimePublicationSnapshotReader, Self> {
        if self.observation.is_some() || self.runtime.is_some() || self.alias.is_none() {
            return Err(self);
        }
        let Self {
            _snapshot_identity,
            _partition,
            alias,
            ..
        } = self;
        Ok(RetiredRuntimePublicationSnapshotReader { _snapshot_identity, _partition, _alias: alias.expect(/* Proof: The preceding check requires the actual retired alias and both operational fields to be empty before consuming these original parts. */ "retired reader retains alias metadata") })
    }
    pub(in crate::relational_source) fn validate_alias_retirement(
        &self,
        owner: &RelationalRuntime,
    ) -> Result<(), RelationalRuntimeHandleAliasRetirementDenial> {
        match &self.runtime {
            Some(runtime) => runtime.validate_immutable_owner_alias(owner),
            None => Ok(()),
        }
    }

    pub(in crate::relational_source) fn alias_retired(&self) -> bool {
        self.alias.is_some()
    }

    pub(in crate::relational_source) fn retire_runtime_alias(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> Result<(), RelationalRuntimeHandleAliasRetirementDenial> {
        let Some(runtime) = self.runtime.take() else {
            return Ok(());
        };
        match runtime.try_retire_immutable_alias(request, owner) {
            Ok(alias) => {
                self.alias = Some(alias);
                Ok(())
            }
            Err((runtime, cause)) => {
                self.runtime = Some(runtime);
                Err(cause)
            }
        }
    }

    pub(in crate::relational_source) fn observation_released(&self) -> bool {
        self.observation.is_none()
    }

    /// The sole caller still owns the untouched non-Clone retained-snapshot
    /// lease. Its row anchors TWO shares of this same inner, so this Drop is
    /// nonfinal and cannot run any Native root/registry destructor.
    pub(in crate::relational_source) fn release_observation_before_lease(&mut self) {
        drop(self.observation.take());
    }
}
