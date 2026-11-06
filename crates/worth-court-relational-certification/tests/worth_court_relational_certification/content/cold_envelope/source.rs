//! Concrete immutable checkpoint/journal provider for this scenario.
use super::*;
use std::any::Any;
use worth_execution::ExecutionImmutableBytes;
use worth_execution::ExecutionRequestArrayBuilder;
use worth_relational::facade::durability::RelationalDurableOwnedReadRefusal as Refusal;
use worth_relational::facade::{
    runtime::RelationalRuntime, storage::RelationalStorageUnavailableCause as Cause,
};
use worth_testkit_relational::durable_medium::bytes::MemberBytes;
use worth_testkit_relational::durable_medium::{OwnedNativeReadSnapshot, OwnedSnapshotDenial};

fn original_snapshot(
    original: &(dyn Any + Send + Sync),
) -> Result<&OwnedNativeReadSnapshot, Refusal> {
    original
        .downcast_ref::<OwnedNativeReadSnapshot>()
        .ok_or(Refusal::ForeignProtection)
}

fn snapshot_failure(cause: OwnedSnapshotDenial, missing: Cause) -> Refusal {
    match cause {
        OwnedSnapshotDenial::Request(cause) => Refusal::Request(cause.into()),
        OwnedSnapshotDenial::Buffer(cause) => Refusal::Buffer(cause),
        OwnedSnapshotDenial::Missing => Refusal::Storage(missing),
        OwnedSnapshotDenial::Integrity => Refusal::Storage(Cause::IntegrityMismatch),
        OwnedSnapshotDenial::Limit { length, limit } => {
            Refusal::Storage(Cause::LimitExceeded { length, limit })
        }
        OwnedSnapshotDenial::Capacity => Refusal::Storage(Cause::MemoryExhausted),
    }
}

#[derive(Debug)]
pub(in crate::content) struct Source {
    members: MemberBytes,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    older: PatchStreamPosition,
    selected_probe: Option<RelationalDurableMemberLocator>,
    pub(super) reads: AtomicUsize,
}
impl Source {
    pub(in crate::content) fn capture(
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        checkpoint: &RelationalNativeCheckpoint,
        older_position: PatchStreamPosition,
        preceding: Option<&Self>,
    ) -> Self {
        let mut members = MemberBytes::default();
        for member in checkpoint.members() {
            match member {
                RelationalCheckpointMember::New { locator, bytes } => {
                    members.insert(locator.sha256, locator.length, bytes.clone())
                }
                RelationalCheckpointMember::Carried { locator } => members.insert(
                    locator.sha256,
                    locator.length,
                    captured_carried_member(
                        execution,
                        &preceding
                            .expect("carried members require the actual preceding source")
                            .members,
                        *locator,
                    ),
                ),
            }
        }
        let commits = runtime
            .durability()
            .canonical_commit_artifacts(execution)
            .map(|artifact| {
                let artifact = artifact.unwrap();
                (artifact.position(), artifact.as_bytes().to_vec())
            })
            .collect();
        Self {
            members,
            commits,
            older: older_position,
            selected_probe: None,
            reads: AtomicUsize::new(0),
        }
    }
}
impl Source {
    pub(super) fn insert_selected_frame(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        member: RelationalDurableMemberLocator,
        bytes: &[u8],
    ) {
        let mut body = ExecutionRequestArrayBuilder::allocate(*request, bytes.len()).unwrap();
        body.extend_from_slice(bytes).unwrap();
        self.members.insert(
            member.sha256,
            member.length,
            body.seal_immutable_bytes().unwrap(),
        );
        self.selected_probe = Some(member);
    }
}

impl RelationalDurableColdSource for Source {
    fn observe_canonical_member_read(&self, member: RelationalDurableMemberLocator) {
        if self.selected_probe == Some(member) {
            self.reads.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, RelationalStorageUnavailableCause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        bounded(
            self.members
                .get(locator.sha256, locator.length)
                .ok_or(RelationalStorageUnavailableCause::MissingMember)?,
            limit,
        )
    }
    fn read_commit(
        &self,
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        if position == self.older {
            self.reads.fetch_add(1, Ordering::SeqCst);
        }
        bounded(
            self.commits
                .get(&position)
                .ok_or(RelationalStorageUnavailableCause::SourceUnavailable)?,
            limit,
        )
    }

    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        original_snapshot(original)?
            .member(request, locator.sha256, locator.length, limit)
            .map_err(|cause| snapshot_failure(cause, Cause::MissingMember))
    }
    fn read_commit_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        let snapshot = original_snapshot(original)?;
        if position == self.older {
            self.reads.fetch_add(1, Ordering::SeqCst);
        }
        snapshot
            .canonical(request, position.0, limit)
            .map_err(|cause| snapshot_failure(cause, Cause::SourceUnavailable))
    }
    fn checkpoint_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(RelationalDurableReadBounds, RelationalDurableReadPayload), Refusal> {
        let original = OwnedNativeReadSnapshot::capture(
            request,
            self.commits
                .iter()
                .map(|(position, bytes)| (position.0, bytes.as_slice())),
            std::iter::empty(),
            &self.members,
            None,
        )
        .map_err(|cause| snapshot_failure(cause, Cause::SourceUnavailable))?;
        let [canonical, transitions] = original.bounds();
        Ok((
            RelationalDurableReadBounds {
                selected_checkpoint: true,
                canonical: PatchStreamPosition(canonical),
                transitions: worth_relational::facade::branch::RelationalBranchTransitionSequence(
                    transitions,
                ),
            },
            original,
        ))
    }

    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, Refusal> {
        OwnedNativeReadSnapshot::capture(
            request,
            std::iter::empty(),
            std::iter::empty(),
            &self.members,
            None,
        )
        .map(|original| original as RelationalDurableReadPayload)
        .map_err(|cause| snapshot_failure(cause, Cause::MissingMember))
    }

    fn read_member_prefix_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        length: usize,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        original_snapshot(original)?
            .member_prefix(request, locator.sha256, locator.length, length)
            .map_err(|cause| snapshot_failure(cause, Cause::MissingMember))
    }

    fn read_initial_inputs_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        limit: u64,
    ) -> Result<Option<ExecutionImmutableBytes>, Refusal> {
        original_snapshot(original)?
            .initial_inputs(request, limit)
            .map_err(|cause| snapshot_failure(cause, Cause::SourceUnavailable))
    }

    fn read_transition_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn Any + Send + Sync),
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        original_snapshot(original)?
            .transition(request, sequence.0, limit)
            .map_err(|cause| snapshot_failure(cause, Cause::SourceUnavailable))
    }

    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, Refusal> {
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        if original.is::<()>() || original.is::<OwnedNativeReadSnapshot>() {
            Ok(())
        } else {
            Err(RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }

    fn advance_member_custody_retirement(
        &self,
        _request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }
}

fn bounded(bytes: &[u8], limit: u64) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
    if bytes.len() as u64 > limit {
        return Err(RelationalStorageUnavailableCause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    Ok(bytes.to_vec())
}
fn captured_carried_member(
    request: &ExecutionRequest<'_, '_>,
    preceding: &MemberBytes,
    locator: RelationalDurableMemberLocator,
) -> ExecutionImmutableBytes {
    let bytes = preceding
        .get(locator.sha256, locator.length)
        .expect("the actual preceding source owns the carried member");
    let mut buffer = ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
        .expect("the actual member copy is funded before allocation");
    buffer.extend_from_slice(bytes).unwrap();
    buffer.seal_immutable_bytes().unwrap()
}
