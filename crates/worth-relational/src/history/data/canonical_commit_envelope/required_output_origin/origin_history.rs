//! Request-scoped custody of validated immutable canonical origin frames.
use super::{CanonicalCommitEnvelope, CommitId, OwnedValidatedOutputOrigins, VersionId};
use crate::history::data::CodecRetirementClosureStop as Stop;
use std::{collections::HashMap, sync::Arc};
use worth_execution::MapKernelContext;

#[derive(Default)]
pub(crate) struct OutputOriginHistory {
    commits: HashMap<CommitId, OwnedValidatedOutputOrigins>,
    versions: HashMap<VersionId, CommitId>,
}

impl OutputOriginHistory {
    pub(crate) fn by_commit(
        &mut self,
        commit: CommitId,
        load: &mut dyn FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<OwnedValidatedOutputOrigins>, Stop> {
        super::super::super::codec_retirement_closure::charge(context, 1)?;
        if let Some(frame) = self.commits.get(&commit) {
            return Ok(Some(frame.clone()));
        }
        let Some(envelope) = load(commit) else {
            return Ok(None);
        };
        if envelope.commit.commit_id != commit {
            return Err("canonical origin commit differs from history coordinate".into());
        }
        self.retain(envelope, context).map(Some)
    }

    pub(crate) fn by_version(
        &mut self,
        version: VersionId,
        load: &mut dyn FnMut(VersionId) -> Option<Arc<CanonicalCommitEnvelope>>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<OwnedValidatedOutputOrigins>, Stop> {
        super::super::super::codec_retirement_closure::charge(context, 1)?;
        if let Some(commit) = self.versions.get(&version) {
            return Ok(self.commits.get(commit).cloned());
        }
        let Some(envelope) = load(version) else {
            return Ok(None);
        };
        if envelope.commit.version_id != version {
            return Err("canonical origin version differs from history coordinate".into());
        }
        self.retain(envelope, context).map(Some)
    }

    fn retain(
        &mut self,
        envelope: Arc<CanonicalCommitEnvelope>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<OwnedValidatedOutputOrigins, Stop> {
        let commit = envelope.commit.commit_id;
        let version = envelope.commit.version_id;
        let frame = OwnedValidatedOutputOrigins::validate(envelope, context)?;
        self.versions.insert(version, commit);
        self.commits.insert(commit, frame.clone());
        Ok(frame)
    }
}
