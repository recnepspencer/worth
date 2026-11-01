//! Live placement obligations and one genuinely frozen publication inventory.
use std::sync::{Arc, RwLock, Weak};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionOrderedMapCell, ExecutionRequest,
};
use worth_relational::facade::durability::{
    RelationalDurableMemberLocator as Member, RelationalDurableOwnedReadRefusal as Refusal,
};
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_store::physical_runtime::ServingPhysicalRuntime;

mod admission;
mod retirement;
#[cfg(test)]
mod retirement_tests;
mod selection;

type Counts = ExecutionOrderedMap<Member, usize>;
type Cell = ExecutionOrderedMapCell<Member, usize>;
#[derive(Debug, Default)]
struct CustodyState {
    // PRIVATE and NEVER CLONED. Every edit uses only the Copy-unique SDK ports.
    counts: Counts,
    selection: Weak<FrozenSelection>,
}
#[derive(Debug)]
struct FrozenSelection {
    members: ExecutionOrderedMap<Member, ()>,
    // Its preexisting fixed Arc/Weak control allocation remains explicitly open.
}
#[derive(Debug, Default)]
pub(super) struct MemberCustodyRegistry {
    state: Arc<RwLock<CustodyState>>,
}
pub(super) struct MemberCustodyOwner {
    serving: Option<Arc<ServingPhysicalRuntime>>,
    state: Arc<RwLock<CustodyState>>,
    members: Vec<Member>,
    next: usize,
    retired_cells: Vec<Cell>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone)]
pub(crate) struct RetainedMemberPublication {
    selection: Arc<FrozenSelection>,
}

impl MemberCustodyRegistry {
    pub(super) fn retain(
        &self,
        members: &[Member],
        serving: Arc<ServingPhysicalRuntime>,
        validate: impl FnOnce(&[Member]) -> Result<(), Cause>,
    ) -> Result<MemberCustodyOwner, Cause> {
        self.admit(members, serving, None, |members| {
            validate(members).map_err(Refusal::Storage)
        })
        .map_err(|cause| match cause {
            Refusal::Storage(cause) => cause,
            _ => Cause::MemoryExhausted,
        })
    }
    pub(super) fn retain_for_request(
        &self,
        members: &[Member],
        serving: Arc<ServingPhysicalRuntime>,
        request: &ExecutionRequest<'_, '_>,
        validate: impl FnOnce(&[Member]) -> Result<(), Refusal>,
    ) -> Result<MemberCustodyOwner, Refusal> {
        self.admit(members, serving, Some(request), validate)
    }
    #[cfg(test)]
    pub(super) fn select<R>(&self, install: impl FnOnce() -> R) -> R {
        let _state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        install()
    }
    pub(super) fn try_select<R>(&self, install: impl FnOnce() -> R) -> Result<R, ()> {
        let _state = match self.state.try_write() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(()),
        };
        Ok(install())
    }
}
impl RetainedMemberPublication {
    pub(crate) fn member_locators(&self) -> impl Iterator<Item = Member> + '_ {
        self.selection.members.keys().copied()
    }
}

/// Remove only an actual original count row; its exact node stays in the guard.
fn release_member(counts: &mut Counts, member: &Member, retired: &mut Vec<Cell>) -> bool {
    let Some(count) = counts.get(member).copied().filter(|count| *count > 0) else {
        return false;
    };
    if count == 1 {
        if retired.len() == retired.capacity() {
            return false;
        }
        let original=counts.remove_retaining_unique(member)
            .expect(/* Proof: CustodyState.counts is private, initialized empty and NEVER cloned. Its sole ordinary/controlled writes use these Copy-unique ports; detached cells contain no children and expose no Arc. All live nodes/entries therefore remain uniquely owned under this same state gate. */ "private count storage stays unique")
            .expect("the original positive count is present under this same exclusive gate");
        retired.push(original);
    } else {
        counts.set_existing_unique(member,count-1)
            .expect(/* Proof: The same private never-cloned unique counts invariant holds for Copy updates; the borrowed original count remains in this exclusive state. */ "private count storage stays unique")
            .expect("original positive count remains present");
    }
    true
}
impl Drop for MemberCustodyOwner {
    fn drop(&mut self) {
        if self.next == self.members.len() {
            return;
        }
        let mut state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        while self.next < self.members.len() {
            assert!(
                release_member(
                    &mut state.counts,
                    &self.members[self.next],
                    &mut self.retired_cells
                ),
                "original custody owns a positive count and preallocated retirement slot"
            );
            self.next += 1;
        }
        // The guard releases BEFORE original detached cells/roster/grants drop.
    }
}
