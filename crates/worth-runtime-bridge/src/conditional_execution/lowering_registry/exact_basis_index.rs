//! Exact-basis point lookup and cleanup by the installed declaration and alias position.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use super::{
    exact_basis_capacity_denial, BridgeConditionalDenial, BridgeConditionalLoweringKey,
    BridgeExactConditionalBasisKey,
};
use crate::conditional_execution::{contract, BridgeInstalledConditionalLowering};

#[derive(Default)]
pub(super) struct BridgeExactConditionalBasisIndex {
    exact: HashMap<BridgeExactConditionalBasisKey, Arc<BridgeInstalledConditionalLowering>>,
    aliases: BTreeMap<BridgeConditionalLoweringKey, AliasOwner>,
}

#[derive(Default)]
struct AliasOwner {
    keys: Vec<BridgeExactConditionalBasisKey>,
    reservations: usize,
}

impl BridgeExactConditionalBasisIndex {
    pub(super) fn get(
        &self,
        key: &BridgeExactConditionalBasisKey,
    ) -> Option<&Arc<BridgeInstalledConditionalLowering>> {
        self.exact.get(key)
    }

    pub(super) fn reserve(
        &mut self,
        owner: &BridgeConditionalLoweringKey,
        additional: usize,
    ) -> Result<(), BridgeConditionalDenial> {
        let reservations = self
            .aliases
            .get(owner)
            .map_or(Some(1), |aliases| aliases.reservations.checked_add(1))
            .ok_or_else(exact_basis_capacity_denial)?;
        self.exact
            .try_reserve(additional)
            .map_err(|_| exact_basis_capacity_denial())?;
        let aliases = self.aliases.entry(owner.clone()).or_default();
        if aliases.keys.try_reserve(additional).is_err() {
            self.prune_empty_owner(owner);
            return Err(exact_basis_capacity_denial());
        }
        aliases.reservations = reservations;
        Ok(())
    }

    pub(super) fn insert(
        &mut self,
        key: BridgeExactConditionalBasisKey,
        lowering: Arc<BridgeInstalledConditionalLowering>,
    ) {
        let owner = contract::lowering_key(&lowering).clone();
        if let Some(previous) = self.exact.insert(key.clone(), lowering) {
            let previous_owner = contract::lowering_key(&previous);
            if let Some(aliases) = self.aliases.get_mut(previous_owner) {
                aliases.keys.retain(|alias| alias != &key);
            }
            self.prune_empty_owner(previous_owner);
        }
        // These are the existing publication positions within one declaration.
        self.aliases.entry(owner).or_default().keys.push(key);
    }

    pub(super) fn release_reservation(&mut self, owner: &BridgeConditionalLoweringKey) {
        // Only a move-only, active registry claim releases its reserved owner.
        if let Some(aliases) = self.aliases.get_mut(owner) {
            aliases.reservations = aliases.reservations.saturating_sub(1);
        }
        self.prune_empty_owner(owner);
    }

    fn prune_empty_owner(&mut self, owner: &BridgeConditionalLoweringKey) {
        if self
            .aliases
            .get(owner)
            .is_some_and(|aliases| aliases.keys.is_empty() && aliases.reservations == 0)
        {
            self.aliases.remove(owner);
        }
    }

    pub(super) fn remove_owner(&mut self, owner: &BridgeConditionalLoweringKey) {
        if let Some(aliases) = self.aliases.get_mut(owner) {
            for alias in aliases.keys.drain(..) {
                self.exact.remove(&alias);
            }
        }
        self.prune_empty_owner(owner);
    }

    pub(super) fn clear(&mut self) {
        let mut reserved = BTreeMap::new();
        while let Some((owner, mut aliases)) = self.aliases.pop_first() {
            for alias in aliases.keys.drain(..) {
                self.exact.remove(&alias);
            }
            if aliases.reservations != 0 {
                reserved.insert(owner, aliases);
            }
        }
        self.aliases = reserved;
    }

    #[cfg(test)]
    pub(super) fn alias_owner_count(&self) -> usize {
        self.aliases.len()
    }

    #[cfg(test)]
    pub(super) fn capacity(&self) -> usize {
        self.exact.capacity()
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.exact.len()
    }
}

impl Drop for BridgeExactConditionalBasisIndex {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod count_tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn owner() -> BridgeConditionalLoweringKey {
        BridgeConditionalLoweringKey::successor(
            worth_signal::facade::branch::signal_branch_identity("alias-count-test", 1, "main")
                .unwrap(),
            worth_signal::facade::NodeId::new(7, 1),
            1,
        )
    }

    #[test]
    fn alias_reservation_exhaustion_refuses_without_panicking() {
        let mut index = BridgeExactConditionalBasisIndex::default();
        let owner = owner();
        index.aliases.insert(
            owner.clone(),
            AliasOwner {
                keys: Vec::new(),
                reservations: usize::MAX,
            },
        );
        let attempt = catch_unwind(AssertUnwindSafe(|| index.reserve(&owner, 1)));
        assert!(
            matches!(&attempt, Ok(Err(_))),
            "exhaustion must refuse without panic"
        );
        let denial = attempt.unwrap().unwrap_err();
        assert_eq!(
            denial.kind(),
            crate::conditional_execution::BridgeConditionalDenialKind::ConditionalRetentionCapacity
        );
        assert_eq!(index.aliases[&owner].reservations, usize::MAX);
    }

    #[test]
    fn alias_release_saturates_underflow_without_panicking() {
        let mut index = BridgeExactConditionalBasisIndex::default();
        let owner = owner();
        index.aliases.insert(owner.clone(), AliasOwner::default());
        assert!(catch_unwind(AssertUnwindSafe(|| index.release_reservation(&owner))).is_ok());
        assert_eq!(index.alias_owner_count(), 0);
    }
}
