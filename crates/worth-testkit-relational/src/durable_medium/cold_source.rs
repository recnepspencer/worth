use super::*;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
fn bounded(bytes: Option<&[u8]>, limit: u64) -> Result<Vec<u8>, Cause> {
    let bytes = bytes.ok_or(Cause::SourceUnavailable)?;
    if bytes.len() as u64 > limit {
        return Err(Cause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    Ok(bytes.to_vec())
}
impl RelationalDurableColdSource for TestDurableMedium {
    fn checkpoint_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            worth_relational::facade::durability::RelationalDurableReadBounds,
            worth_relational::facade::durability::RelationalDurableReadPayload,
        ),
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        let contents = self.lock();
        let snapshot = super::OwnedNativeReadSnapshot::capture(
            request,
            contents
                .commits
                .entries
                .iter()
                .map(|(id, bytes)| (*id, bytes.as_ref())),
            contents
                .transitions
                .entries
                .iter()
                .map(|(id, bytes)| (*id, bytes.as_ref())),
            &contents.members,
        )
        .map_err(owned_failure)?;
        let [canonical, transitions] = snapshot.bounds();
        Ok((
            worth_relational::facade::durability::RelationalDurableReadBounds {
                canonical: PatchStreamPosition(canonical),
                transitions: worth_relational::facade::branch::RelationalBranchTransitionSequence(
                    transitions,
                ),
            },
            snapshot,
        ))
    }
    fn read_commit_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        position: worth_relational::facade::publication::PatchStreamPosition,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        let snapshot=original.downcast_ref::<super::OwnedNativeReadSnapshot>().ok_or(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        snapshot
            .canonical(request, position.0, limit)
            .map_err(owned_failure)
    }
    fn read_transition_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        let snapshot=original.downcast_ref::<super::OwnedNativeReadSnapshot>().ok_or(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        snapshot
            .transition(request, sequence.0, limit)
            .map_err(owned_failure)
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: worth_relational::facade::durability::RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        let snapshot=original.downcast_ref::<super::OwnedNativeReadSnapshot>().ok_or(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        snapshot
            .member(request, locator.sha256, locator.length, limit)
            .map_err(owned_failure)
    }
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
        bounded(
            self.lock().members.get(locator.sha256, locator.length),
            limit,
        )
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        bounded(
            self.lock()
                .commits
                .entries
                .get(&position.0)
                .map(AsRef::as_ref),
            limit,
        )
    }
}

fn owned_failure(
    cause: super::OwnedSnapshotDenial,
) -> worth_relational::facade::durability::RelationalDurableOwnedReadRefusal {
    use super::OwnedSnapshotDenial as Failure;
    use worth_relational::facade::durability::RelationalDurableOwnedReadRefusal as Refusal;
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
