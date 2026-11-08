//! Observe and interrupt the real public selected-read port, retaining its issuer.
use super::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, Weak,
};
use worth_execution::{CancellationSource, ExecutionMemoryReservation};
use worth_relational::facade::{
    publication::PatchStreamPosition, storage::RelationalStorageUnavailableCause,
};
mod custody;
pub(super) use custody::CustodyControl;

#[derive(Debug)]
pub(crate) enum MemberStop {
    Cancel(CancellationSource),
    Exhaust {
        held: Arc<Mutex<Option<ExecutionMemoryReservation>>>,
    },
    MemoryRoom {
        held: Arc<Mutex<Option<ExecutionMemoryReservation>>>,
        budget: worth_execution::SerialMemoryBudget,
        room: u64,
    },
}
#[derive(Debug)]
pub(crate) struct SelectedSource {
    pub(super) medium: Arc<TestDurableMedium>,
    pub(super) canonical_reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(crate) fired: AtomicBool,
    pub(crate) member_reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(super) journal_reads: Mutex<Vec<PatchStreamPosition>>,
    stop: Mutex<Option<(RelationalDurableMemberLocator, MemberStop)>>,
    custody: Mutex<Option<Arc<CustodyControl>>>,
}
impl SelectedSource {
    pub(crate) fn new(medium: Arc<TestDurableMedium>) -> Arc<Self> {
        Arc::new(Self {
            medium,
            canonical_reads: Mutex::new(Vec::new()),
            fired: AtomicBool::new(false),
            member_reads: Mutex::new(Vec::new()),
            journal_reads: Mutex::new(Vec::new()),
            stop: Mutex::new(None),
            custody: Mutex::new(None),
        })
    }
    pub(crate) fn arm(&self, member: RelationalDurableMemberLocator, stop: MemberStop) {
        assert!(self.stop.lock().unwrap().replace((member, stop)).is_none());
        self.fired.store(false, Ordering::Relaxed);
    }
    pub(super) fn arm_custody(&self, control: Arc<CustodyControl>) {
        assert!(self.custody.lock().unwrap().replace(control).is_none());
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
                reserve_leaving(request, &held, 0);
            }
            MemberStop::MemoryRoom { held, budget, room } => {
                reserve_budget_room(&budget, &held, room)
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
        assert!(
            self.custody.lock().unwrap().is_none(),
            "armed custody has no unfunded retention fallback"
        );
        self.medium.retain_members(members)
    }
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        if let Some(control) = self.custody.lock().unwrap().clone() {
            return control.retain(&self.medium, request, members, issuer);
        }
        self.medium
            .retain_members_for_request(request, members, issuer)
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        if let Some(result) = custody::validate(original) {
            return result;
        }
        self.medium.validate_member_custody_retirement(original)
    }
    fn advance_member_custody_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        if let Some(result) = custody::retire(original) {
            return result;
        }
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
        let bytes = self.medium.read_member_owned(
            request,
            custody::read_payload(original, member)?,
            member,
            limit,
        )?;
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
        let bytes = self.medium.read_member(member, limit)?;
        self.member_reads.lock().unwrap().push(member);
        let mut armed = self.stop.lock().unwrap();
        if armed.as_ref().is_some_and(|(target, stop)| {
            *target == member && !matches!(stop, MemberStop::Exhaust { .. })
        }) {
            let (_, stop) = armed.take().unwrap();
            self.fired.store(true, Ordering::Relaxed);
            match stop {
                MemberStop::Cancel(cancellation) => cancellation.cancel(),
                MemberStop::MemoryRoom { held, budget, room } => {
                    reserve_budget_room(&budget, &held, room);
                }
                MemberStop::Exhaust { .. } => {
                    unreachable!("an ordinary read has no execution scope to exhaust")
                }
            }
        }
        Ok(bytes)
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

fn reserve_leaving(
    request: &ExecutionRequest<'_, '_>,
    held: &Mutex<Option<ExecutionMemoryReservation>>,
    room: u64,
) {
    let (reservation, _) = request
        .run_native_with_checkpoint(|_| {
            let remaining = ExecutionMemoryReservation::available_in_scope(None)
                .expect("the invoking serial scope has its original ceiling");
            let bytes = remaining
                .checked_sub(room)
                .expect("fixture room fits original allowance");
            ExecutionMemoryReservation::reserve_in_scope(None, bytes).unwrap()
        })
        .unwrap();
    assert!(held.lock().unwrap().replace(reservation).is_none());
}

fn reserve_budget_room(
    budget: &worth_execution::SerialMemoryBudget,
    held: &Mutex<Option<ExecutionMemoryReservation>>,
    room: u64,
) {
    let available = budget
        .limit()
        .unwrap()
        .checked_sub(budget.charged_memory_bytes())
        .unwrap();
    let bytes = available
        .checked_sub(room)
        .expect("fixture room fits original budget");
    assert!(held
        .lock()
        .unwrap()
        .replace(budget.reserve(bytes).unwrap())
        .is_none());
}
