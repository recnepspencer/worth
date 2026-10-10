use std::collections::BTreeMap;
use std::sync::{Arc, PoisonError, RwLock};

use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
};

mod exact_basis;
mod exact_basis_index;
mod key;
pub(in crate::conditional_execution) use exact_basis::BridgeExactConditionalBasisKey;
use exact_basis_index::BridgeExactConditionalBasisIndex;
pub(in crate::conditional_execution) use key::BridgeConditionalLoweringKey;

pub(in crate::conditional_execution) enum BridgeConditionalLoweringSlot {
    Claimed,
    Installed(Arc<BridgeInstalledConditionalLowering>),
}

#[derive(Default)]
pub(super) struct BridgeConditionalLoweringRegistry {
    slots: BTreeMap<BridgeConditionalLoweringKey, BridgeConditionalLoweringSlot>,
    exact_basis: BridgeExactConditionalBasisIndex,
    reserved_exact_basis_slots: usize,
}

pub(super) struct BridgeConditionalLoweringSlotClaim {
    registry: Arc<RwLock<BridgeConditionalLoweringRegistry>>,
    key: BridgeConditionalLoweringKey,
    active: bool,
}

pub(super) enum BridgeExactConditionalBasisLookup {
    Indexed(Arc<BridgeInstalledConditionalLowering>),
    Unindexed(BridgePreparedExactConditionalBasisResolution),
}

pub(super) struct BridgePreparedExactConditionalBasisResolution {
    candidates: Vec<Arc<BridgeInstalledConditionalLowering>>,
    claim: BridgeExactConditionalBasisClaim,
}

struct BridgeExactConditionalBasisClaim {
    registry: Arc<RwLock<BridgeConditionalLoweringRegistry>>,
    key: BridgeExactConditionalBasisKey,
    alias_owners: Vec<BridgeConditionalLoweringKey>,
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
                self.exact_basis.remove_owner(key);
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
    ) -> BTreeMap<BridgeConditionalLoweringKey, Arc<BridgeInstalledConditionalLowering>> {
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
        lowerings: BTreeMap<BridgeConditionalLoweringKey, Arc<BridgeInstalledConditionalLowering>>,
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
        installed.reserve_exact_basis_slot(&key)?;
        installed
            .slots
            .insert(key.clone(), BridgeConditionalLoweringSlot::Claimed);
        Ok(BridgeConditionalLoweringSlotClaim {
            registry: Arc::clone(registry),
            key,
            active: true,
        })
    }

    pub(super) fn prepare_exact_basis_resolution(
        registry: &Arc<RwLock<Self>>,
        anchor: &Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeExactConditionalBasisLookup, BridgeConditionalDenial> {
        let key = BridgeExactConditionalBasisKey::new(
            basis.admission_identity().clone(),
            anchor.signal_node(),
        );
        let mut installed = registry.write().unwrap_or_else(PoisonError::into_inner);
        if let Some(lowering) = installed.exact_basis.get(&key) {
            return Ok(BridgeExactConditionalBasisLookup::Indexed(Arc::clone(
                lowering,
            )));
        }

        let mut candidates = Vec::new();
        candidates
            .try_reserve(installed.slots.len())
            .map_err(|_| exact_basis_capacity_denial())?;
        candidates.extend(
            installed
                .values()
                .filter(|candidate| {
                    candidate.signal_node() == anchor.signal_node()
                        && candidate.contract().identity() == anchor.contract().identity()
                        && candidate.location() == anchor.location()
                })
                .cloned(),
        );
        let mut alias_owners = Vec::new();
        alias_owners
            .try_reserve_exact(candidates.len() + 1)
            .map_err(|_| exact_basis_capacity_denial())?;
        let anchor_owner = super::contract::lowering_key(anchor);
        alias_owners.push(anchor_owner.clone());
        // Installed slots already have unique declaration keys; only the anchor
        // can occur both here and in that inventory. No quadratic dedup scan.
        for candidate in &candidates {
            let owner = super::contract::lowering_key(candidate);
            if owner != anchor_owner {
                alias_owners.push(owner.clone());
            }
        }
        let total = installed
            .reserved_exact_basis_slots
            .checked_add(1)
            .ok_or_else(exact_basis_capacity_denial)?;
        for (reserved, owner) in alias_owners.iter().enumerate() {
            if installed.exact_basis.reserve(owner, total).is_err() {
                for prior in &alias_owners[..reserved] {
                    installed.exact_basis.release_reservation(prior);
                }
                return Err(exact_basis_capacity_denial());
            }
        }
        installed.reserved_exact_basis_slots = total;
        drop(installed);

        Ok(BridgeExactConditionalBasisLookup::Unindexed(
            BridgePreparedExactConditionalBasisResolution {
                candidates,
                claim: BridgeExactConditionalBasisClaim {
                    registry: Arc::clone(registry),
                    key,
                    alias_owners,
                    active: true,
                },
            },
        ))
    }

    fn reserve_exact_basis_slot(
        &mut self,
        owner: &BridgeConditionalLoweringKey,
    ) -> Result<(), BridgeConditionalDenial> {
        let exact_reservations = self
            .reserved_exact_basis_slots
            .checked_add(1)
            .ok_or_else(exact_basis_capacity_denial)?;
        self.exact_basis
            .reserve(owner, exact_reservations)
            .map_err(|_| exact_basis_capacity_denial())?;
        self.reserved_exact_basis_slots = exact_reservations;
        Ok(())
    }
}

impl BridgePreparedExactConditionalBasisResolution {
    pub(super) fn candidates(&self) -> &[Arc<BridgeInstalledConditionalLowering>] {
        &self.candidates
    }

    pub(super) fn publish(mut self, lowering: &Arc<BridgeInstalledConditionalLowering>) {
        self.claim.publish(lowering);
    }
}

impl BridgeExactConditionalBasisClaim {
    fn publish(&mut self, lowering: &Arc<BridgeInstalledConditionalLowering>) {
        let mut registry = self
            .registry
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        if registry.exact_basis.get(&self.key).is_none() {
            registry
                .exact_basis
                .insert(self.key.clone(), Arc::clone(lowering));
        }
        for owner in &self.alias_owners {
            registry.exact_basis.release_reservation(owner);
        }
        registry.reserved_exact_basis_slots = registry
            .reserved_exact_basis_slots
            .checked_sub(1)
            .expect("published exact-basis claim owns one reservation");
        self.active = false;
    }
}

impl Drop for BridgeExactConditionalBasisClaim {
    fn drop(&mut self) {
        if self.active {
            let mut registry = self
                .registry
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            for owner in &self.alias_owners {
                registry.exact_basis.release_reservation(owner);
            }
            registry.reserved_exact_basis_slots = registry
                .reserved_exact_basis_slots
                .checked_sub(1)
                .expect("active exact-basis claim owns one reservation");
        }
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
        registry.exact_basis.release_reservation(&self.key);
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
            registry.exact_basis.release_reservation(&self.key);
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
mod tests;
