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
    fn observe_canonical_member_read(&self, _: RelationalDurableMemberLocator) {
        self.canonical_member_reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    fn read_member_prefix_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        length: usize,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        let snapshot = original.downcast_ref::<super::OwnedNativeReadSnapshot>()
            .ok_or(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        snapshot
            .member_prefix(request, member.sha256, member.length, length)
            .map_err(owned_failure)
    }
    fn read_initial_inputs_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        limit: u64,
    ) -> Result<
        Option<worth_execution::ExecutionImmutableBytes>,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        let snapshot = original.downcast_ref::<super::OwnedNativeReadSnapshot>().ok_or(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        snapshot
            .initial_inputs(request, limit)
            .map_err(owned_failure)
    }
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
            contents
                .initial_inputs
                .as_ref()
                .map(|artifact| artifact.as_bytes()),
        )
        .map_err(owned_failure)?;
        let [canonical, transitions] = snapshot.bounds();
        Ok((
            worth_relational::facade::durability::RelationalDurableReadBounds {
                selected_checkpoint: contents.checkpoint.is_some(),
                canonical: PatchStreamPosition(canonical),
                transitions: worth_relational::facade::branch::RelationalBranchTransitionSequence(
                    transitions,
                ),
            },
            snapshot,
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_relational::facade::durability::RelationalDurableReadPayload,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        let contents = self.lock();
        super::OwnedNativeReadSnapshot::capture(
            request,
            std::iter::empty(),
            std::iter::empty(),
            &contents.members,
            contents
                .initial_inputs
                .as_ref()
                .map(|artifact| artifact.as_bytes()),
        )
        .map(|original| {
            original as worth_relational::facade::durability::RelationalDurableReadPayload
        })
        .map_err(owned_failure)
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
        self.canonical_journal_reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
        if let Some(cancellation) = self.member_cancellation.lock().unwrap().take() {
            cancellation.cancel();
        }
        let snapshot=original.downcast_ref::<super::OwnedNativeReadSnapshot>().ok_or(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        self.read_original_member(request, snapshot, locator, limit)
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), worth_relational::facade::durability::RelationalDurableMemberRetirementRefusal>
    {
        // These actual immutable payloads own bytes/metadata only, with no physical pin or registration.
        if original.is::<()>() || original.is::<super::OwnedNativeReadSnapshot>() {
            Ok(())
        } else {
            Err(worth_relational::facade::durability::RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> worth_relational::facade::durability::RelationalDurableMemberRetirementObservation {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => worth_relational::facade::durability::RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => worth_relational::facade::durability::RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<
        worth_relational::facade::durability::RelationalDurableMemberCustody,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        worth_relational::facade::durability::RelationalDurableMemberCustody::try_new_from_source_for_request(issuer,request,||Ok(()))
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
        if let Some(cancellation) = self.member_cancellation.lock().unwrap().take() {
            cancellation.cancel();
        }
        bounded(
            self.lock().members.get(locator.sha256, locator.length),
            limit,
        )
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.canonical_journal_reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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

pub(super) fn owned_failure(
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
