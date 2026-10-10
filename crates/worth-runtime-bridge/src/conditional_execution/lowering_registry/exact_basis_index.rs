//! Exact-basis point lookup and cleanup by the installed declaration and alias position.

use std::collections::{BTreeMap, HashMap, TryReserveError};
use std::sync::Arc;

use super::{BridgeConditionalLoweringKey, BridgeExactConditionalBasisKey};
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
    ) -> Result<(), TryReserveError> {
        self.exact.try_reserve(additional)?;
        let aliases = self.aliases.entry(owner.clone()).or_default();
        if let Err(error) = aliases.keys.try_reserve(additional) {
            self.prune_empty_owner(owner);
            return Err(error);
        }
        // Each owner count is bounded by the registry's checked total of claims.
        aliases.reservations += 1;
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
            aliases.reservations -= 1;
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
