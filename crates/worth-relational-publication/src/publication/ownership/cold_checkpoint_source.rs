//! Read-only archive of the real recorder's members and canonical commit artifacts.
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use std::{collections::BTreeMap, sync::Arc};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_durability::internal_port::durability::data::*;
use worth_relational_durability::internal_port::durability::published_medium::recovery_source::{
    RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
};
use worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal as Refusal;
use worth_relational_storage::internal_port::durability::medium::*;
#[derive(Debug)]
pub struct Source {
    members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
}
impl Source {
    /// The capture's own external members, without its canonical commits.
    pub fn members_of(native: &RelationalNativeCheckpoint) -> Arc<dyn RelationalDurableColdSource> {
        let members = native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
                _ => panic!("first external capture owns its actual members"),
            })
            .collect();
        Arc::new(Self {
            members,
            commits: BTreeMap::new(),
        })
    }
    pub fn archive<I>(
        native: RelationalNativeCheckpoint,
        artifacts: impl FnOnce() -> I,
    ) -> RelationalDurableRecoverySource
    where
        I: Iterator<Item = Result<
            worth_relational_durability::internal_port::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact,
            contracts::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial,
        >>,
{
        let members = native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
                _ => panic!("first external capture owns its actual members"),
            })
            .collect();
        let commits = artifacts()
            .map(|artifact| {
                let artifact = artifact.expect("the actual retained commit encodes");
                (artifact.position(), artifact.as_bytes().to_vec())
            })
            .collect();
        RelationalDurableRecoverySource::new(
            Some(RelationalDurableCheckpointSource::new(
                native.covered_position().expect("recorded position"),
                native
                    .covered_branch_transitions()
                    .expect("recorded branch coverage"),
                native,
            )),
            Vec::new(),
            Vec::new(),
            Arc::new(Self { members, commits }),
        )
    }
}
impl Source {
    fn owned(
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        bytes: Option<&Vec<u8>>,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, Refusal> {
        if !original.is::<()>() {
            return Err(Refusal::ForeignProtection);
        }
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let bytes = bounded(bytes, limit).map_err(Refusal::Storage)?;
        let mut output =
            worth_execution::ExecutionRequestArrayBuilder::<u8>::allocate(*request, bytes.len())
                .map_err(Refusal::Buffer)?;
        output.extend_from_slice(&bytes).map_err(Refusal::Buffer)?;
        output.seal_immutable_bytes().map_err(Refusal::Buffer)
    }
}
// The archive owns immutable copies only: its protection and member custody
// are unit payloads with no physical pin, like the inline checkpoint source.
impl RelationalDurableColdSource for Source {
    fn checkpoint_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(RelationalDurableReadBounds, RelationalDurableReadPayload), Refusal> {
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let canonical = self
            .commits
            .keys()
            .next_back()
            .copied()
            .unwrap_or(PatchStreamPosition(0));
        Ok((
            RelationalDurableReadBounds {
                selected_checkpoint: true,
                canonical,
                transitions:
                    contracts::branch::transition::sequence::RelationalBranchTransitionSequence(0),
            },
            Box::new(()),
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, Refusal> {
        self.checkpoint_read_protection(request)
            .map(|(_, original)| original)
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, Refusal> {
        Self::owned(request, original, self.members.get(&member), limit)
    }
    fn read_commit_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, Refusal> {
        Self::owned(request, original, self.commits.get(&position), limit)
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        if original.is::<()>() {
            Ok(())
        } else {
            Err(RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, Refusal> {
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }
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
