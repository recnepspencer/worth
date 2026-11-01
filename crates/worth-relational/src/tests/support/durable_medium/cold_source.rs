use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

impl RelationalDurableColdSource for super::TestDurableMedium {
    fn checkpoint_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            crate::durability::medium::RelationalDurableReadBounds,
            crate::durability::medium::RelationalDurableReadPayload,
        ),
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        let journal = self.journal();
        let transitions = self.transition_journal();
        let snapshot = worth_testkit_relational::durable_medium::OwnedNativeReadSnapshot::capture(
            request,
            journal
                .artifacts
                .entries
                .iter()
                .map(|(id, bytes)| (*id, bytes.as_ref())),
            transitions
                .artifacts
                .entries
                .iter()
                .map(|(id, bytes)| (*id, bytes.as_ref())),
            &journal.checkpoint_members,
        )
        .map_err(owned_failure)?;
        let [canonical, transitions] = snapshot.bounds();
        Ok((
            crate::durability::medium::RelationalDurableReadBounds {
                canonical: PatchStreamPosition(canonical),
                transitions: crate::branch::RelationalBranchTransitionSequence(transitions),
            },
            snapshot,
        ))
    }
    fn read_commit_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        position: crate::publication::patch::data::PatchStreamPosition,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        let snapshot = original
            .downcast_ref::<worth_testkit_relational::durable_medium::OwnedNativeReadSnapshot>()
            .ok_or(
                crate::durability::medium::RelationalDurableOwnedReadRefusal::ForeignProtection,
            )?;
        snapshot
            .canonical(request, position.0, limit)
            .map_err(owned_failure)
    }
    fn read_transition_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        sequence: crate::branch::RelationalBranchTransitionSequence,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        let snapshot = original
            .downcast_ref::<worth_testkit_relational::durable_medium::OwnedNativeReadSnapshot>()
            .ok_or(
                crate::durability::medium::RelationalDurableOwnedReadRefusal::ForeignProtection,
            )?;
        snapshot
            .transition(request, sequence.0, limit)
            .map_err(owned_failure)
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: crate::durability::medium::RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        let snapshot = original
            .downcast_ref::<worth_testkit_relational::durable_medium::OwnedNativeReadSnapshot>()
            .ok_or(
                crate::durability::medium::RelationalDurableOwnedReadRefusal::ForeignProtection,
            )?;
        snapshot
            .member(request, locator.sha256, locator.length, limit)
            .map_err(owned_failure)
    }
    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<crate::durability::medium::RelationalDurableMemberCustody, Cause> {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let journal = self.journal();
        let bytes = journal
            .member_bytes(locator)
            .ok_or(Cause::SourceUnavailable)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.to_vec())
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        let journal = self.journal();
        let bytes = journal.bytes_at(position).ok_or(Cause::SourceUnavailable)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.to_vec())
    }
}

fn owned_failure(
    cause: worth_testkit_relational::durable_medium::OwnedSnapshotDenial,
) -> crate::durability::medium::RelationalDurableOwnedReadRefusal {
    use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
    use worth_testkit_relational::durable_medium::OwnedSnapshotDenial as Failure;
    match cause {
        Failure::Request(cause) => Refusal::Request(cause.into()),
        Failure::Buffer(cause) => Refusal::Buffer(cause),
        Failure::Missing => Refusal::Storage(Cause::MissingMember),
        Failure::Integrity => Refusal::Storage(Cause::IntegrityMismatch),
        Failure::Limit { length, limit } => {
            Refusal::Storage(Cause::LimitExceeded { length, limit })
        }
        Failure::Capacity => Refusal::Storage(Cause::MemoryExhausted),
    }
}
