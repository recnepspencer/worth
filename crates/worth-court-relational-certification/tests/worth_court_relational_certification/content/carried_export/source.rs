//! Actual checkpoint member/journal provider for carried export.
use super::*;
use worth_testkit_relational::durable_medium::bytes::MemberBytes;
#[derive(Debug)]
pub(super) struct Source {
    members: MemberBytes,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub(super) fault: AtomicUsize,
    pub(super) reads: AtomicUsize,
}
impl Source {
    pub(super) fn from_checkpoint(
        checkpoint: &RelationalNativeCheckpoint,
        commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    ) -> Self {
        Self {
            members: captured_members(checkpoint),
            commits,
            fault: AtomicUsize::new(0),
            reads: AtomicUsize::new(0),
        }
    }
}
use std::any::Any;
use worth_execution::ExecutionImmutableBytes;
use worth_relational::facade::durability::RelationalDurableOwnedReadRefusal as Refusal;
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

impl RelationalDurableColdSource for Source {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let bytes = self
            .members
            .get(locator.sha256, locator.length)
            .ok_or(Cause::MissingMember)?;
        bounded(bytes, limit)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        let mut bytes = bounded(
            self.commits
                .get(&position)
                .ok_or(Cause::SourceUnavailable)?,
            limit,
        )?;
        if position == PatchStreamPosition(1) {
            self.reads.fetch_add(1, Ordering::SeqCst);
            match self.fault.load(Ordering::SeqCst) {
                0 => {}
                1 => bytes[8..10].copy_from_slice(&2_u16.to_le_bytes()),
                2 => bytes[58..66].copy_from_slice(&(256_u64 * 1024 * 1024 - 65).to_le_bytes()),
                3 => *bytes.last_mut().unwrap() ^= 1,
                value => panic!("unrecognized court fault {value}"),
            }
        }
        Ok(bytes)
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
        let bytes = snapshot
            .canonical(request, position.0, limit)
            .map_err(|cause| snapshot_failure(cause, Cause::SourceUnavailable))?;
        if position == PatchStreamPosition(1) {
            self.reads.fetch_add(1, Ordering::SeqCst);
        }
        if position == PatchStreamPosition(1) {
            corrupted_original(request, bytes, self.fault.load(Ordering::SeqCst))
        } else {
            Ok(bytes)
        }
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

fn bounded(bytes: &[u8], limit: u64) -> Result<Vec<u8>, Cause> {
    if bytes.len() as u64 > limit {
        return Err(Cause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    Ok(bytes.to_vec())
}
use worth_execution::ExecutionRequestArrayBuilder;

fn captured_members(checkpoint: &RelationalNativeCheckpoint) -> MemberBytes {
    let mut members = MemberBytes::default();
    for member in checkpoint.members() {
        match member {
            RelationalCheckpointMember::New { locator, bytes } => {
                members.insert(locator.sha256, locator.length, bytes.clone());
            }
            RelationalCheckpointMember::Carried { .. } => {
                panic!("initial capture owns its actual member bytes");
            }
        }
    }
    members
}

// Apply only at the carried-export original read, after the canonical limit
// and original opaque-snapshot checks. The existing fault/observation counters
// stay on that exact body-read seam, never on prefix/protection acquisition.
fn corrupted_original(
    request: &ExecutionRequest<'_, '_>,
    original: ExecutionImmutableBytes,
    fault: usize,
) -> Result<ExecutionImmutableBytes, Refusal> {
    if fault == 0 {
        return Ok(original);
    }
    let mut bytes = ExecutionRequestArrayBuilder::allocate(*request, original.len())
        .map_err(Refusal::Buffer)?;
    bytes
        .extend_from_slice(original.bytes())
        .map_err(Refusal::Buffer)?;
    match fault {
        1 => {
            for (offset, byte) in 2_u16.to_le_bytes().into_iter().enumerate() {
                bytes
                    .replace_initialized(8 + offset, byte)
                    .map_err(Refusal::Buffer)?;
            }
        }
        2 => {
            for (offset, byte) in (256_u64 * 1024 * 1024 - 65)
                .to_le_bytes()
                .into_iter()
                .enumerate()
            {
                bytes
                    .replace_initialized(58 + offset, byte)
                    .map_err(Refusal::Buffer)?;
            }
        }
        3 => {
            let index = original.len() - 1;
            bytes
                .replace_initialized(index, original.bytes()[index] ^ 1)
                .map_err(Refusal::Buffer)?;
        }
        value => panic!("unrecognized court fault {value}"),
    }
    bytes.seal_immutable_bytes().map_err(Refusal::Buffer)
}
