//! Live read-side placement obligations and their checkpoint selection fence.
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use worth_relational::facade::durability::{
    RelationalDurableMemberCustody, RelationalDurableMemberLocator,
};
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_store::physical_runtime::ServingPhysicalRuntime;

type Counts = BTreeMap<RelationalDurableMemberLocator, usize>;

#[derive(Debug, Default)]
pub(super) struct MemberCustodyRegistry {
    counts: Arc<RwLock<Counts>>,
}

struct MemberCustodyOwner {
    _serving: Arc<ServingPhysicalRuntime>,
    counts: Arc<RwLock<Counts>>,
    members: Vec<RelationalDurableMemberLocator>,
}

pub(crate) struct RetainedMemberPublication<'source> {
    counts: RwLockReadGuard<'source, Counts>,
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
        let mut counts = self
            .counts
            .write()
            .unwrap_or_else(|error| error.into_inner());
        // Publication holds the reader until its new membership is selected.
        // Validation therefore cannot race a selection that removes a member.
        validate(&selected)?;
        if selected
            .iter()
            .any(|member| counts.get(member) == Some(&usize::MAX))
        {
            return Err(Cause::MemoryExhausted);
        }
        for member in &selected {
            *counts.entry(*member).or_default() += 1;
        }
        Ok(RelationalDurableMemberCustody::new(MemberCustodyOwner {
            _serving: serving,
            counts: self.counts.clone(),
            members: selected,
        }))
    }

    pub(super) fn protect_selection(&self) -> RetainedMemberPublication<'_> {
        RetainedMemberPublication {
            counts: self
                .counts
                .read()
                .unwrap_or_else(|error| error.into_inner()),
        }
    }
}

impl RetainedMemberPublication<'_> {
    pub(crate) fn member_locators(
        &self,
    ) -> impl Iterator<Item = RelationalDurableMemberLocator> + '_ {
        self.counts.keys().copied()
    }
}

impl Drop for MemberCustodyOwner {
    fn drop(&mut self) {
        let mut counts = self
            .counts
            .write()
            .unwrap_or_else(|error| error.into_inner());
        for member in &self.members {
            let count = counts
                .get_mut(member)
                .expect("owned member custody has a count");
            *count -= 1;
            if *count == 0 {
                counts.remove(member);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_fence_excludes_custody_mutation_until_membership_is_selected() {
        let registry = MemberCustodyRegistry::default();
        let fence = registry.protect_selection();
        assert!(registry.counts.try_write().is_err());
        drop(fence);
        assert!(registry.counts.try_write().is_ok());
    }
}
