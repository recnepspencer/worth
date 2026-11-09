//! Request-scoped custody of validated immutable canonical origin frames.
use super::{CanonicalCommitEnvelope, CommitId, OwnedValidatedOutputOrigins, VersionId};
use crate::history::data::CodecRetirementClosureStop as Stop;
use std::sync::Arc;
use worth_execution::MapKernelContext;
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest};

#[derive(Default)]
pub(crate) struct OutputOriginHistory {
    commits: ExecutionOrderedMap<CommitId, OwnedValidatedOutputOrigins>,
    versions: ExecutionOrderedMap<VersionId, CommitId>,
}

impl OutputOriginHistory {
    pub(crate) fn by_commit(
        &mut self,
        commit: CommitId,
        load: &mut dyn FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<OwnedValidatedOutputOrigins>, Stop> {
        super::super::super::codec_retirement_closure::charge(context, 1)?;
        if let Some(frame) = self.commits.get(&commit) {
            return Ok(Some(frame.clone()));
        }
        let Some(envelope) = load(commit)? else {
            return Ok(None);
        };
        if envelope.commit.commit_id != commit {
            return Err("canonical origin commit differs from history coordinate".into());
        }
        self.retain(envelope, execution, context).map(Some)
    }

    pub(crate) fn by_version(
        &mut self,
        version: VersionId,
        load: &mut dyn FnMut(
            VersionId,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<OwnedValidatedOutputOrigins>, Stop> {
        super::super::super::codec_retirement_closure::charge(context, 1)?;
        if let Some(commit) = self.versions.get(&version) {
            return Ok(self.commits.get(commit).cloned());
        }
        let Some(envelope) = load(version, context)? else {
            return Ok(None);
        };
        if envelope.commit.version_id != version {
            return Err("canonical origin version differs from history coordinate".into());
        }
        self.retain(envelope, execution, context).map(Some)
    }

    fn retain(
        &mut self,
        envelope: Arc<CanonicalCommitEnvelope>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<OwnedValidatedOutputOrigins, Stop> {
        let commit = envelope.commit.commit_id;
        let version = envelope.commit.version_id;
        let frame = OwnedValidatedOutputOrigins::validate(envelope, context)?;
        let mut versions = self.versions.clone();
        let mut commits = self.commits.clone();
        versions
            .insert_cloned_native_for_request(&version, &commit, *execution)
            .map_err(cache_stop)?;
        commits
            .insert_cloned_native_for_request(&commit, &frame, *execution)
            .map_err(cache_stop)?;
        self.versions = versions;
        self.commits = commits;
        Ok(frame)
    }
}

fn cache_stop(cause: ExecutionOrderedMapDenial) -> Stop {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    match cause {
        ExecutionOrderedMapDenial::Request(cause) => Stop::Request(cause.into()),
        ExecutionOrderedMapDenial::CapacityOverflow => Stop::Request(Cause::ChargedBytesOverflow),
        ExecutionOrderedMapDenial::Allocation(cause) => match cause.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
                Stop::Request(cause.into())
            }
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
                Stop::Request(Cause::AllocationFailed)
            }
        },
    }
}
