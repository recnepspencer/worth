//! Actual provider-issued, request-funded selected-member placement rosters.
use super::*;
use std::any::Any;
use std::sync::atomic::AtomicUsize;
use worth_execution::{ExecutionArray, ExecutionRequestArrayBuilder};

#[derive(Debug, Default)]
pub(in super::super) struct CustodyControl {
    pub(in super::super) live: AtomicUsize,
    pub(in super::super) captures: AtomicUsize,
    pub(in super::super) members: AtomicUsize,
    exhaust: Mutex<Option<Exhaustion>>,
}
#[derive(Debug)]
struct Exhaustion {
    budget: worth_execution::SerialMemoryBudget,
    held: Arc<Mutex<Option<ExecutionMemoryReservation>>>,
}

struct PinnedMembers {
    members: ExecutionArray<RelationalDurableMemberLocator>,
    selected: RelationalDurableReadPayload,
    control: Arc<CustodyControl>,
    released: bool,
}
impl PinnedMembers {
    fn release(&mut self) {
        if !self.released {
            self.control.live.fetch_sub(1, Ordering::SeqCst);
            self.released = true;
        }
    }
}
impl Drop for PinnedMembers {
    fn drop(&mut self) {
        self.release();
    }
}
impl CustodyControl {
    pub(in super::super) fn exhaust_once(
        &self,
        budget: worth_execution::SerialMemoryBudget,
        held: Arc<Mutex<Option<ExecutionMemoryReservation>>>,
    ) {
        assert!(self
            .exhaust
            .lock()
            .unwrap()
            .replace(Exhaustion { budget, held })
            .is_none());
    }
    pub(super) fn retain(
        self: &Arc<Self>,
        medium: &Arc<TestDurableMedium>,
        request: &ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        if let Some(pressure) = self.exhaust.lock().unwrap().take() {
            super::reserve_budget_room(&pressure.budget, &pressure.held, 0);
        }
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            let mut roster = ExecutionRequestArrayBuilder::allocate(*request, members.len())
                .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
            for member in members {
                roster
                    .push(*member)
                    .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
            }
            let members = roster
                .seal()
                .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
            let selected = medium.checkpoint_selected_read_protection(request)?;
            self.members.store(members.len(), Ordering::SeqCst);
            self.captures.fetch_add(1, Ordering::SeqCst);
            self.live.fetch_add(1, Ordering::SeqCst);
            Ok(PinnedMembers {
                members,
                selected,
                control: self.clone(),
                released: false,
            })
        })
    }
}

pub(super) fn read_payload(
    original: &(dyn Any + Send + Sync),
    member: RelationalDurableMemberLocator,
) -> Result<&(dyn Any + Send + Sync), RelationalDurableOwnedReadRefusal> {
    let Some(pin) = original.downcast_ref::<PinnedMembers>() else {
        return Ok(original);
    };
    if pin.released || !pin.members.contains(&member) {
        return Err(RelationalDurableOwnedReadRefusal::ForeignProtection);
    }
    Ok(pin.selected.as_ref())
}
pub(super) fn validate(
    original: &(dyn Any + Send + Sync),
) -> Option<Result<(), RelationalDurableMemberRetirementRefusal>> {
    original.downcast_ref::<PinnedMembers>().map(|_| Ok(()))
}
pub(super) fn retire(
    original: &mut (dyn Any + Send + Sync),
) -> Option<RelationalDurableMemberRetirementObservation> {
    original.downcast_mut::<PinnedMembers>().map(|pin| {
        pin.release();
        RelationalDurableMemberRetirementObservation::Complete
    })
}
