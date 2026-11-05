//! Read-only archive of the real recorder's members and canonical commit artifacts.
use crate::{
    durability::{data::*, medium::*},
    publication::patch::data::PatchStreamPosition,
    runtime::RelationalRuntime,
    storage::data::RelationalStorageUnavailableCause as Cause,
};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Debug)]
pub(super) struct Source {
    members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
}
impl Source {
    pub(super) fn capture(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
    ) -> (RelationalNativeCheckpoint, Arc<Self>) {
        let native = runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                execution,
                execution.allocation_policy(),
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &[],
                    source: None,
                },
            )
            .expect("the actual private branch records");
        Self::archive(execution, runtime, native)
    }
    pub(super) fn capture_corrupted(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        publication: crate::history::data::CommitId,
        change: u8,
    ) -> (RelationalNativeCheckpoint, Arc<Self>) {
        let original = runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                execution,
                execution.allocation_policy(),
                RelationalCheckpointArtifactPolicy::Embedded,
            )
            .expect("actual Query checkpoint");
        let native = original
            .corrupt_current_publication_roster_external_for_test(execution, publication, change)
            .expect(
                "the real recorder emits damaged external current members without admitting them",
            );
        Self::archive(execution, runtime, native)
    }

    fn archive(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        native: RelationalNativeCheckpoint,
    ) -> (RelationalNativeCheckpoint, Arc<Self>) {
        let members = native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
                _ => panic!("first external capture owns its actual members"),
            })
            .collect();
        let commits = runtime
            .durability()
            .canonical_commit_artifacts(execution.allocation_policy())
            .map(|artifact| {
                let artifact = artifact.expect("the actual retained commit encodes");
                (artifact.position(), artifact.as_bytes().to_vec())
            })
            .collect();
        (native, Arc::new(Self { members, commits }))
    }
}
impl RelationalDurableColdSource for Source {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        key: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        bounded(self.members.get(&key), limit)
    }
    fn read_commit(&self, key: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        bounded(self.commits.get(&key), limit)
    }
}
fn bounded(bytes: Option<&Vec<u8>>, limit: u64) -> Result<Vec<u8>, Cause> {
    let bytes = bytes.ok_or(Cause::MissingMember)?;
    if bytes.len() as u64 > limit {
        return Err(Cause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    Ok(bytes.clone())
}
