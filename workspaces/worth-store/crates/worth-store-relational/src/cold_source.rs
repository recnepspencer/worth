//! Read-only placement adapter. Relational verifies its own member and artifact
//! frames after Store has verified physical integrity and current membership.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock, Weak,
};

use worth_relational::facade::durability::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_store::physical_runtime::{
    OrderedJournalReadFailure, RecordReadDenial, RecordReadError, ServingPhysicalRuntime,
};

mod frame_prefix;
mod initial_inputs;
mod initial_selection;
mod member_custody;
mod member_read;
mod owned_read;
mod read_protection;
mod selected_read;
pub(crate) mod selected_replacement;
mod selection;
pub use selection::RelationalStoreMemberSelectionDenial;
#[cfg(test)]
mod member_custody_tests;
#[cfg(test)]
pub(crate) mod test_store;
#[cfg(test)]
mod tests;
pub(crate) use member_custody::RetainedMemberPublication;

/// Canonical transport/demand and member reads, including typed refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalStoreColdReadCounters {
    pub member_reads: u64,
    pub commit_reads: u64,
}

pub(crate) struct StoreColdSource {
    weak_self: Weak<Self>,
    members: RwLock<selection::SelectedMembers>,
    custody: member_custody::MemberCustodyRegistry,
    member_reads: AtomicU64,
    commit_reads: AtomicU64,
    commit_read_observer:
        Option<Box<dyn Fn(PatchStreamPosition, RelationalStoreColdReadCounters) + Send + Sync>>,
    // Local selected readers/pins retire before a possible final pool-owner Drop.
    serving: Arc<ServingPhysicalRuntime>,
    _allocation_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl std::fmt::Debug for StoreColdSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreColdSource").finish_non_exhaustive()
    }
}

impl StoreColdSource {
    pub(crate) fn observe_original_commit_transport(&self, position: PatchStreamPosition) {
        // The closed Home transport delivers the body read by its original
        // protected reader. Observe that SAME ingress; never read it again.
        self.commit_reads.fetch_add(1, Ordering::Relaxed);
        if let Some(observer) = &self.commit_read_observer {
            observer(position, self.counters());
        }
    }
    pub(crate) fn new(serving: Arc<ServingPhysicalRuntime>) -> Arc<Self> {
        Self::new_admitted(serving, None, None)
    }
    pub(crate) fn new_for_request(
        serving: Arc<ServingPhysicalRuntime>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, worth_execution::WorkCeilingDenial> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed Source allocation layout fits")
            .0
            .pad_to_align();
        let memory = request.reserve_retained_memory(layout.size() as u64)?;
        Ok(Self::new_admitted(serving, memory, None))
    }
    fn new_admitted(
        serving: Arc<ServingPhysicalRuntime>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        observer: Option<
            Box<dyn Fn(PatchStreamPosition, RelationalStoreColdReadCounters) + Send + Sync>,
        >,
    ) -> Arc<Self> {
        Arc::new_cyclic(|weak_self| Self {
            weak_self: weak_self.clone(),
            members: RwLock::new(Default::default()),
            custody: Default::default(),
            member_reads: AtomicU64::new(0),
            commit_reads: AtomicU64::new(0),
            commit_read_observer: observer,
            serving,
            _allocation_memory: memory,
        })
    }
    pub(crate) fn counters(&self) -> RelationalStoreColdReadCounters {
        RelationalStoreColdReadCounters {
            member_reads: self.member_reads.load(Ordering::Relaxed),
            commit_reads: self.commit_reads.load(Ordering::Relaxed),
        }
    }
    #[cfg(feature = "certification-test-authority")]
    pub(crate) fn new_with_commit_read_observer(
        serving: Arc<ServingPhysicalRuntime>,
        observer: impl Fn(PatchStreamPosition, RelationalStoreColdReadCounters) + Send + Sync + 'static,
    ) -> Arc<Self> {
        Self::new_admitted(serving, None, Some(Box::new(observer)))
    }
    #[cfg(feature = "certification-test-authority")]
    pub(crate) fn new_with_owned_observer_for_request(
        serving: Arc<ServingPhysicalRuntime>,
        observer: Box<dyn Fn(PatchStreamPosition, RelationalStoreColdReadCounters) + Send + Sync>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, worth_execution::WorkCeilingDenial> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed Source allocation layout fits")
            .0
            .pad_to_align();
        let memory = request.reserve_retained_memory(layout.size() as u64)?;
        Ok(Self::new_admitted(serving, memory, Some(observer)))
    }
    pub(crate) fn protect_member_selection_for_request(
        &self,
        next: impl IntoIterator<Item = RelationalDurableMemberLocator>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        RetainedMemberPublication,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        self.custody.protect_selection_for_request(next, request)
    }
    pub(crate) fn member_locators(&self) -> Vec<RelationalDurableMemberLocator> {
        self.members
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .keys()
            .copied()
            .collect()
    }

    fn retain_custody_owner(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<member_custody::MemberCustodyOwner, Cause> {
        self.custody
            .retain(members, self.serving.clone(), |selected| {
                let current = self
                    .members
                    .read()
                    .unwrap_or_else(|error| error.into_inner());
                if selected
                    .iter()
                    .all(|member| current.entries.contains_key(member))
                {
                    Ok(())
                } else {
                    Err(Cause::MissingMember)
                }
            })
    }
}

impl RelationalDurableColdSource for StoreColdSource {
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
        self.owned_member_prefix(request, original, member, length)
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
        self.owned_initial_inputs(request, original, limit)
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_relational::facade::durability::RelationalDurableReadPayload,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        self.capture_selected_read(request)
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
        self.capture_native_read(request)
    }
    fn read_commit_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        self.owned_commit(request, original, position, limit)
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
        self.owned_transition(request, original, sequence, limit)
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        self.owned_member(request, original, locator, limit)
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), worth_relational::facade::durability::RelationalDurableMemberRetirementRefusal>
    {
        self.validate_custody_owner(original)
    }

    fn advance_member_custody_retirement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> worth_relational::facade::durability::RelationalDurableMemberRetirementObservation {
        self.advance_custody_owner(execution, original)
    }

    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
    ) -> Result<
        RelationalDurableMemberCustody,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        use worth_relational::facade::durability::RelationalDurableOwnedReadRefusal as Refusal;
        if issuer.as_ptr().cast::<()>() != self.weak_self.as_ptr().cast::<()>() {
            return Err(Refusal::ForeignSource);
        }
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            self.custody
                .retain_for_request(members, self.serving.clone(), request, |selected| {
                    let current = match self.members.try_read() {
                        Ok(current) => current,
                        Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
                        Err(std::sync::TryLockError::WouldBlock) => return Err(Refusal::Busy),
                    };
                    if selected
                        .iter()
                        .all(|member| current.entries.contains_key(member))
                    {
                        Ok(())
                    } else {
                        Err(Refusal::Storage(Cause::MissingMember))
                    }
                })
        })
    }

    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new_from_source(
            self.retain_custody_owner(members)?,
            self.weak_self.clone(),
        ))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.read_selected_member(locator, limit)
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.commit_reads.fetch_add(1, Ordering::Relaxed);
        let bytes = self
            .serving
            .canonical_commits()
            .map_err(|_| Cause::SourceUnavailable)?
            .read(position.0, limit)
            .map(|record| record.into_bytes())
            .map_err(|failure| match failure {
                OrderedJournalReadFailure::NotFound { .. } => Cause::MissingMember,
                OrderedJournalReadFailure::Open(error) => record_failure(error),
                OrderedJournalReadFailure::Damaged => Cause::Damaged,
                OrderedJournalReadFailure::LimitExceeded { payload_bytes } => {
                    Cause::LimitExceeded {
                        length: payload_bytes,
                        limit,
                    }
                }
                _ => Cause::SourceUnavailable,
            })?;
        #[cfg(feature = "certification-test-authority")]
        if let Some(observer) = &self.commit_read_observer {
            observer(position, self.counters());
        }
        Ok(bytes)
    }
}

fn record_failure(error: RecordReadError) -> Cause {
    match error.denial() {
        RecordReadDenial::RecordNotFound => Cause::MissingMember,
        RecordReadDenial::ArtifactDamaged | RecordReadDenial::FormatMismatch => Cause::Damaged,
        _ => Cause::SourceUnavailable,
    }
}
