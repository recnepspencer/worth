use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, PoisonError, RwLock};

use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
};

mod exact_basis;
mod key;
pub(in crate::conditional_execution) use exact_basis::BridgeExactConditionalBasisKey;
pub(in crate::conditional_execution) use key::BridgeConditionalLoweringKey;

pub(in crate::conditional_execution) enum BridgeConditionalLoweringSlot {
    Claimed,
    Installed(Arc<BridgeInstalledConditionalLowering>),
}

#[derive(Default)]
pub(super) struct BridgeConditionalLoweringRegistry {
    slots: BTreeMap<BridgeConditionalLoweringKey, BridgeConditionalLoweringSlot>,
    exact_basis: HashMap<BridgeExactConditionalBasisKey, Arc<BridgeInstalledConditionalLowering>>,
    reserved_exact_basis_slots: usize,
}

pub(super) struct BridgeConditionalLoweringSlotClaim {
    registry: Arc<RwLock<BridgeConditionalLoweringRegistry>>,
    key: BridgeConditionalLoweringKey,
    active: bool,
}

impl BridgeConditionalLoweringRegistry {
    pub(super) fn install(
        &mut self,
        key: BridgeConditionalLoweringKey,
        lowering: Arc<BridgeInstalledConditionalLowering>,
    ) {
        self.slots
            .insert(key, BridgeConditionalLoweringSlot::Installed(lowering));
    }

    pub(super) fn index_exact_basis(&mut self, lowering: &Arc<BridgeInstalledConditionalLowering>) {
        if let Some(port) = lowering.signal_port() {
            self.exact_basis.insert(
                BridgeExactConditionalBasisKey::new(
                    port.issuance_basis().admission_identity().clone(),
                    lowering.signal_node(),
                ),
                Arc::clone(lowering),
            );
        }
    }

    pub(super) fn exact_basis(
        &self,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        node: worth_signal::facade::NodeId,
    ) -> Option<&Arc<BridgeInstalledConditionalLowering>> {
        self.exact_basis.get(&BridgeExactConditionalBasisKey::new(
            basis.admission_identity().clone(),
            node,
        ))
    }

    pub(super) fn get(
        &self,
        key: &BridgeConditionalLoweringKey,
    ) -> Option<&Arc<BridgeInstalledConditionalLowering>> {
        match self.slots.get(key) {
            Some(BridgeConditionalLoweringSlot::Installed(lowering)) => Some(lowering),
            Some(BridgeConditionalLoweringSlot::Claimed) | None => None,
        }
    }

    pub(super) fn remove(
        &mut self,
        key: &BridgeConditionalLoweringKey,
    ) -> Option<Arc<BridgeInstalledConditionalLowering>> {
        match self.slots.remove(key) {
            Some(BridgeConditionalLoweringSlot::Installed(lowering)) => {
                self.exact_basis
                    .retain(|_, retained| !Arc::ptr_eq(retained, &lowering));
                Some(lowering)
            }
            Some(BridgeConditionalLoweringSlot::Claimed) | None => None,
        }
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &Arc<BridgeInstalledConditionalLowering>> {
        self.slots.values().filter_map(|slot| match slot {
            BridgeConditionalLoweringSlot::Installed(lowering) => Some(lowering),
            BridgeConditionalLoweringSlot::Claimed => None,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.values().count()
    }

    pub(super) fn snapshot(
        &self,
    ) -> HashMap<BridgeConditionalLoweringKey, Arc<BridgeInstalledConditionalLowering>> {
        self.slots
            .iter()
            .filter_map(|(key, slot)| match slot {
                BridgeConditionalLoweringSlot::Installed(lowering) => {
                    Some((key.clone(), Arc::clone(lowering)))
                }
                BridgeConditionalLoweringSlot::Claimed => None,
            })
            .collect()
    }

    pub(super) fn replace_installed(
        &mut self,
        lowerings: HashMap<BridgeConditionalLoweringKey, Arc<BridgeInstalledConditionalLowering>>,
    ) {
        self.slots = lowerings
            .into_iter()
            .map(|(key, lowering)| (key, BridgeConditionalLoweringSlot::Installed(lowering)))
            .collect();
        self.exact_basis.clear();
        let installed = self.values().cloned().collect::<Vec<_>>();
        for lowering in installed {
            self.index_exact_basis(&lowering);
        }
        self.exact_basis.reserve(self.reserved_exact_basis_slots);
    }

    pub(super) fn claim(
        registry: &Arc<RwLock<Self>>,
        key: BridgeConditionalLoweringKey,
    ) -> Result<BridgeConditionalLoweringSlotClaim, BridgeConditionalDenial> {
        let mut installed = registry.write().unwrap_or_else(PoisonError::into_inner);
        if installed.slots.contains_key(&key) {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalNodeAlreadyBound,
                "conditional definition generation is already installed or reserved",
            ));
        }
        let exact_reservations = installed
            .reserved_exact_basis_slots
            .checked_add(1)
            .ok_or_else(exact_basis_capacity_denial)?;
        installed
            .exact_basis
            .try_reserve(exact_reservations)
            .map_err(|_| exact_basis_capacity_denial())?;
        installed.reserved_exact_basis_slots = exact_reservations;
        installed
            .slots
            .insert(key.clone(), BridgeConditionalLoweringSlot::Claimed);
        Ok(BridgeConditionalLoweringSlotClaim {
            registry: Arc::clone(registry),
            key,
            active: true,
        })
    }
}

impl BridgeConditionalLoweringSlotClaim {
    pub(super) fn publish(mut self, lowering: Arc<BridgeInstalledConditionalLowering>) {
        let mut registry = self
            .registry
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(slot) = registry.slots.get_mut(&self.key) {
            *slot = BridgeConditionalLoweringSlot::Installed(Arc::clone(&lowering));
        }
        registry.index_exact_basis(&lowering);
        registry.reserved_exact_basis_slots = registry
            .reserved_exact_basis_slots
            .checked_sub(1)
            .expect("published conditional claim owns one exact-basis reservation");
        self.active = false;
    }
}

impl Drop for BridgeConditionalLoweringSlotClaim {
    fn drop(&mut self) {
        if self.active {
            let mut registry = self
                .registry
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            if matches!(
                registry.slots.get(&self.key),
                Some(BridgeConditionalLoweringSlot::Claimed)
            ) {
                registry.slots.remove(&self.key);
            }
            registry.reserved_exact_basis_slots = registry
                .reserved_exact_basis_slots
                .checked_sub(1)
                .expect("active conditional claim owns one exact-basis reservation");
        }
    }
}

fn exact_basis_capacity_denial() -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(
        BridgeConditionalDenialKind::ConditionalRetentionCapacity,
        "conditional exact-basis index capacity could not be reserved before publication",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(generation: u64) -> BridgeConditionalLoweringKey {
        BridgeConditionalLoweringKey::successor(
            worth_signal::facade::branch::signal_branch_identity(
                "lowering-registry-test",
                1,
                "main",
            )
            .unwrap(),
            worth_signal::facade::NodeId::new(7, 1),
            generation,
        )
    }

    #[test]
    fn claims_preallocate_and_release_exact_basis_capacity() {
        let registry = Arc::new(RwLock::new(BridgeConditionalLoweringRegistry::default()));
        let first = BridgeConditionalLoweringRegistry::claim(&registry, key(1)).unwrap();
        {
            let state = registry.read().unwrap();
            assert_eq!(state.reserved_exact_basis_slots, 1);
            assert!(state.exact_basis.capacity() >= state.exact_basis.len() + 1);
        }
        let second = BridgeConditionalLoweringRegistry::claim(&registry, key(2)).unwrap();
        {
            let state = registry.read().unwrap();
            assert_eq!(state.reserved_exact_basis_slots, 2);
            assert!(state.exact_basis.capacity() >= state.exact_basis.len() + 2);
        }

        drop(first);
        assert_eq!(registry.read().unwrap().reserved_exact_basis_slots, 1);
        drop(second);
        let state = registry.read().unwrap();
        assert_eq!(state.reserved_exact_basis_slots, 0);
        assert!(state.slots.is_empty());
    }
}
