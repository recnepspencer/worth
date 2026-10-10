//! Exact-basis point lookup and cleanup by the installed declaration and alias position.

use std::collections::{BTreeMap, HashMap, TryReserveError};
use std::sync::Arc;

use super::{BridgeConditionalLoweringKey, BridgeExactConditionalBasisKey};
use crate::conditional_execution::{contract, BridgeInstalledConditionalLowering};

#[derive(Default)]
pub(super) struct BridgeExactConditionalBasisIndex {
    exact: HashMap<BridgeExactConditionalBasisKey, Arc<BridgeInstalledConditionalLowering>>,
    aliases: BTreeMap<BridgeConditionalLoweringKey, Vec<BridgeExactConditionalBasisKey>>,
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
        self.aliases
            .entry(owner.clone())
            .or_default()
            .try_reserve(additional)
    }

    pub(super) fn insert(
        &mut self,
        key: BridgeExactConditionalBasisKey,
        lowering: Arc<BridgeInstalledConditionalLowering>,
    ) {
        let owner = contract::lowering_key(&lowering).clone();
        if let Some(previous) = self.exact.insert(key.clone(), lowering) {
            let previous_owner = contract::lowering_key(&previous);
            self.aliases
                .get_mut(previous_owner)
                .expect("indexed basis has its declared owner")
                .retain(|alias| alias != &key);
        }
        // These are the existing publication positions within one declaration.
        self.aliases.entry(owner).or_default().push(key);
    }

    pub(super) fn remove_owner(&mut self, owner: &BridgeConditionalLoweringKey) {
        if let Some(aliases) = self.aliases.remove(owner) {
            for alias in aliases {
                self.exact.remove(&alias);
            }
        }
    }

    pub(super) fn clear(&mut self) {
        while let Some((_, aliases)) = self.aliases.pop_first() {
            for alias in aliases {
                self.exact.remove(&alias);
            }
        }
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
