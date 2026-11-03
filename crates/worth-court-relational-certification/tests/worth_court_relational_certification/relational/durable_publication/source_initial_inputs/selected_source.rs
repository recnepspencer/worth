//! Observe and interrupt the real public selected-read port, retaining its issuer.
use super::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, Weak,
};
use worth_execution::{CancellationSource, ExecutionMemoryReservation};
use worth_relational::facade::{
    durability::*, publication::PatchStreamPosition, storage::RelationalStorageUnavailableCause,
};

#[derive(Debug)]
pub(super) enum MemberStop {
    Cancel(CancellationSource),
    Exhaust {
        held: Arc<Mutex<Option<ExecutionMemoryReservation>>>,
    },
}
#[derive(Debug)]
pub(super) struct SelectedSource {
    pub(super) medium: Arc<TestDurableMedium>,
    pub(super) canonical_reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(super) fired: AtomicBool,
    pub(super) journal_reads: Mutex<Vec<PatchStreamPosition>>,
    stop: Mutex<Option<(RelationalDurableMemberLocator, MemberStop)>>,
}
impl SelectedSource {
    pub(super) fn new(medium: Arc<TestDurableMedium>) -> Arc<Self> {
        Arc::new(Self {
            medium,
            canonical_reads: Mutex::new(Vec::new()),
            fired: AtomicBool::new(false),
            journal_reads: Mutex::new(Vec::new()),
            stop: Mutex::new(None),
        })
    }
    pub(super) fn arm(&self, member: RelationalDurableMemberLocator, stop: MemberStop) {
        assert!(self.stop.lock().unwrap().replace((member, stop)).is_none());
        self.fired.store(false, Ordering::Relaxed);
    }
    fn after_member(
        &self,
        request: &ExecutionRequest<'_, '_>,
        member: RelationalDurableMemberLocator,
    ) {
        let mut armed = self.stop.lock().unwrap();
        if armed
            .as_ref()
            .is_none_or(|(selected, _)| *selected != member)
        {
            return;
        }
        let (_, stop) = armed.take().unwrap();
        self.fired.store(true, Ordering::Relaxed);
        match stop {
            MemberStop::Cancel(cancellation) => cancellation.cancel(),
            MemberStop::Exhaust { held } => {
                let (reservation, _) = request
                    .run_native_with_checkpoint(|_| {
                        let remaining = ExecutionMemoryReservation::available_in_scope(None)
                            .expect("the invoking serial scope has its original ceiling");
                        ExecutionMemoryReservation::reserve_in_scope(None, remaining).unwrap()
                    })
                    .unwrap();
                assert!(held.lock().unwrap().replace(reservation).is_none());
            }
        }
    }
}
impl RelationalDurableColdSource for SelectedSource {
    fn observe_canonical_member_read(&self, member: RelationalDurableMemberLocator) {
        self.canonical_reads.lock().unwrap().push(member);
        self.medium.observe_canonical_member_read(member);
    }
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, RelationalStorageUnavailableCause> {
        self.medium.retain_members(members)
    }
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        self.medium
            .retain_members_for_request(request, members, issuer)
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        self.medium.validate_member_custody_retirement(original)
    }
    fn advance_member_custody_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        self.medium
            .advance_member_custody_retirement(request, original)
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        self.medium.checkpoint_selected_read_protection(request)
    }
    fn checkpoint_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (RelationalDurableReadBounds, RelationalDurableReadPayload),
        RelationalDurableOwnedReadRefusal,
    > {
        self.medium.checkpoint_read_protection(request)
    }
    fn read_initial_inputs_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        limit: u64,
    ) -> Result<Option<ExecutionImmutableBytes>, RelationalDurableOwnedReadRefusal> {
        self.medium
            .read_initial_inputs_owned(request, original, limit)
    }
    fn read_member_prefix_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        length: usize,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        self.medium
            .read_member_prefix_owned(request, original, member, length)
    }
    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        let bytes = self
            .medium
            .read_member_owned(request, original, member, limit)?;
        self.after_member(request, member);
        Ok(bytes)
    }
    fn read_commit_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        self.journal_reads.lock().unwrap().push(position);
        self.medium
            .read_commit_owned(request, original, position, limit)
    }
    fn read_transition_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        self.medium
            .read_transition_owned(request, original, sequence, limit)
    }
    fn read_member(
        &self,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        self.medium.read_member(member, limit)
    }
    fn read_commit(
        &self,
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        self.journal_reads.lock().unwrap().push(position);
        self.medium.read_commit(position, limit)
    }
}
