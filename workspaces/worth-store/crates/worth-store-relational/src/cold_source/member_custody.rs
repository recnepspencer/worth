//! Live placement obligations and an owned, frozen checkpoint selection.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock, Weak};
use worth_relational::facade::durability::{
    RelationalDurableMemberCustody, RelationalDurableMemberLocator,
};
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_store::physical_runtime::ServingPhysicalRuntime;

#[derive(Debug, Default)]
struct CustodyState {
    counts: BTreeMap<RelationalDurableMemberLocator, usize>,
    selection: Weak<FrozenSelection>,
}

#[derive(Debug)]
struct FrozenSelection {
    members: BTreeSet<RelationalDurableMemberLocator>,
}

#[derive(Debug, Default)]
pub(super) struct MemberCustodyRegistry {
    state: Arc<RwLock<CustodyState>>,
}

struct MemberCustodyOwner {
    _serving: Arc<ServingPhysicalRuntime>,
    state: Arc<RwLock<CustodyState>>,
    members: Vec<RelationalDurableMemberLocator>,
}

/// Lifetime-only protection. Clones protect the same frozen selection.
#[derive(Debug, Clone)]
pub(crate) struct RetainedMemberPublication {
    selection: Arc<FrozenSelection>,
}

impl MemberCustodyRegistry {
    pub(super) fn retain(
        &self,
        members: &[RelationalDurableMemberLocator],
        serving: Arc<ServingPhysicalRuntime>,
        validate: impl FnOnce(&[RelationalDurableMemberLocator]) -> Result<(), Cause>,
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        let mut selected = Vec::new();
        selected
            .try_reserve_exact(members.len())
            .map_err(|_| Cause::MemoryExhausted)?;
        selected.extend_from_slice(members);
        selected.sort_unstable();
        selected.dedup();
        let mut state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        if state.selection.upgrade().is_some_and(|selection| {
            selected
                .iter()
                .any(|member| !selection.members.contains(member))
        }) {
            return Err(Cause::MissingMember);
        }
        // Registry then source-map is the common order for retention and cutover.
        // New placement is not retainable until the actual selection is installed.
        validate(&selected)?;
        if selected
            .iter()
            .any(|member| state.counts.get(member) == Some(&usize::MAX))
        {
            return Err(Cause::MemoryExhausted);
        }
        for member in &selected {
            *state.counts.entry(*member).or_default() += 1;
        }
        Ok(RelationalDurableMemberCustody::new(MemberCustodyOwner {
            _serving: serving,
            state: self.state.clone(),
            members: selected,
        }))
    }

    pub(super) fn protect_selection(
        &self,
        next: impl IntoIterator<Item = RelationalDurableMemberLocator>,
    ) -> Result<RetainedMemberPublication, Cause> {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        if state.selection.upgrade().is_some() {
            return Err(Cause::SourceUnavailable);
        }
        // Acknowledged directories intrinsically own all their semantic members,
        // so their basis and every inherited read obligation are in these counts.
        let selection = Arc::new(FrozenSelection {
            members: next
                .into_iter()
                .chain(state.counts.keys().copied())
                .collect(),
        });
        state.selection = Arc::downgrade(&selection);
        Ok(RetainedMemberPublication { selection })
    }

    pub(super) fn select(&self, install: impl FnOnce()) {
        let _state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        install();
    }
}

impl RetainedMemberPublication {
    pub(crate) fn member_locators(
        &self,
    ) -> impl Iterator<Item = RelationalDurableMemberLocator> + '_ {
        self.selection.members.iter().copied()
    }
}

impl Drop for MemberCustodyOwner {
    fn drop(&mut self) {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        for member in &self.members {
            let count = state
                .counts
                .get_mut(member)
                .expect("owned member custody has a count");
            *count -= 1;
            if *count == 0 {
                state.counts.remove(member);
            }
        }
    }
}
