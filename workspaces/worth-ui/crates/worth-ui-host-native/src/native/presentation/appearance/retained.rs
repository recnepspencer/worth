#[path = "retained/damage.rs"]
mod damage;
use damage::{map_index_denial, record_damage, record_overlay_order_damage};
#[path = "retained/capacity.rs"]
mod capacity;
use capacity::{family_capacity, family_slot};
#[path = "retained/order_query.rs"]
mod order_query;
#[path = "retained/replay.rs"]
mod replay;
#[path = "retained/text_index.rs"]
mod text_index;
#[path = "retained/text_paint.rs"]
mod text_paint;
pub(super) use text_index::text_paint_commands;
#[path = "retained/text_transaction.rs"]
mod text_transaction;
pub(crate) use text_transaction::{UiNativeAppearanceCommandUndo, UiNativeTextCoverageUndo};

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::super::damage_index::UiNativeDamageIndex;
use super::super::retained_order::{UiNativeRetainedOrder, UiNativeRetainedOrderDenial};
use super::command::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandFamily, UiNativeAppearanceCommandIdentity,
    UiNativeAppearanceCommandKey,
};
use super::damage::{UiNativeAppearanceDamage, UiNativeAppearanceDamageRect};
use super::damage_candidate::candidate_bounds;
use super::geometry::{UiNativeAppearanceScale, UiNativeGeometryDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeAppearanceRetentionCounters {
    pub(crate) inserted_commands: usize,
    pub(crate) replaced_commands: usize,
    pub(crate) removed_commands: usize,
    pub(crate) order_edits: usize,
    pub(crate) damage_queries: usize,
    pub(crate) damage_branch_probes: usize,
    pub(crate) damage_leaf_probes: usize,
    pub(crate) replayed_commands: usize,
    pub(crate) full_scan_commands: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeAppearanceRetainedDenial {
    DuplicateIdentity,
    MissingIdentity,
    CapacityExceeded,
    FamilyCapacityExceeded,
    IdentityFamilyMismatch,
    EmptyDamage,
    StaleTextImages,
    Geometry(UiNativeGeometryDenial),
    DamageCapacityExceeded,
    DamageIndexCapacityExceeded,
    Order(UiNativeRetainedOrderDenial),
    OverlayOrderMismatch,
}

pub(crate) struct UiNativeAppearanceRetained {
    scale: UiNativeAppearanceScale,
    commands: BTreeMap<UiNativeAppearanceCommandKey, UiNativeAppearanceCommand>,
    identities: BTreeMap<UiNativeAppearanceCommandIdentity, UiNativeAppearanceCommandKey>,
    text_keys_by_paint_command: HashMap<
        worth_ui_host_contract::UiMountedPaintCommandIdentity,
        BTreeSet<UiNativeAppearanceCommandKey>,
    >,
    overlay_order: Option<UiNativeAppearanceCommandKey>,
    order: UiNativeRetainedOrder<UiNativeAppearanceCommandKey>,
    damage_index: UiNativeDamageIndex<UiNativeAppearanceCommandKey>,
    damage_bounds: BTreeMap<UiNativeAppearanceCommandKey, UiNativeAppearanceDamageRect>,
    pending_damage: UiNativeAppearanceDamage,
    family_counts: [u16; 7],
    next_key: u32,
    counters: UiNativeAppearanceRetentionCounters,
}

impl UiNativeAppearanceRetained {
    pub(crate) fn new(scale: UiNativeAppearanceScale) -> Self {
        Self {
            scale,
            commands: BTreeMap::new(),
            identities: BTreeMap::new(),
            text_keys_by_paint_command: HashMap::new(),
            overlay_order: None,
            order: UiNativeRetainedOrder::initial([])
                .expect("the empty staged retained order is always admissible"),
            damage_index: UiNativeDamageIndex::new(),
            damage_bounds: BTreeMap::new(),
            pending_damage: UiNativeAppearanceDamage::new(usize::from(
                crate::native_profile::APPEARANCE_PROFILE.damage_regions,
            )),
            family_counts: [0; 7],
            next_key: 1,
            counters: UiNativeAppearanceRetentionCounters {
                inserted_commands: 0,
                replaced_commands: 0,
                removed_commands: 0,
                order_edits: 0,
                damage_queries: 0,
                damage_branch_probes: 0,
                damage_leaf_probes: 0,
                replayed_commands: 0,
                full_scan_commands: 0,
            },
        }
    }

    pub(crate) const fn scale(&self) -> UiNativeAppearanceScale {
        self.scale
    }

    pub(crate) fn insert(
        &mut self,
        command: UiNativeAppearanceCommand,
        predecessor: Option<UiNativeAppearanceCommandKey>,
    ) -> Result<UiNativeAppearanceCommandKey, UiNativeAppearanceRetainedDenial> {
        let identity = command.identity();
        let family = command.family();
        if self.identities.contains_key(&identity) {
            return Err(UiNativeAppearanceRetainedDenial::DuplicateIdentity);
        }
        self.check_capacity(command.family())?;
        let next_key = self
            .next_key
            .checked_add(1)
            .ok_or(UiNativeAppearanceRetainedDenial::CapacityExceeded)?;
        let key = UiNativeAppearanceCommandKey::new(self.next_key);
        let damage = command
            .damage_rect(self.scale)
            .map_err(UiNativeAppearanceRetainedDenial::Geometry)?;
        let mut pending_damage = self.pending_damage.clone();
        record_damage(&command, damage, &mut pending_damage)?;
        let candidate = damage
            .map(|rect| candidate_bounds(rect).map_err(UiNativeAppearanceRetainedDenial::Geometry))
            .transpose()?;
        self.order
            .place_after(key, predecessor)
            .map_err(UiNativeAppearanceRetainedDenial::Order)?;
        if let Some(candidate) = candidate {
            if let Err(error) = self.damage_index.insert(key, candidate) {
                let _ = self.order.remove(key);
                return Err(map_index_denial(error));
            }
        }
        self.next_key = next_key;
        let text_commands = text_paint_commands(&command);
        self.identities.insert(identity, key);
        self.commands.insert(key, command);
        if family == UiNativeAppearanceCommandFamily::OverlayOrder {
            self.overlay_order = Some(key);
        }
        for identity in text_commands {
            self.text_keys_by_paint_command
                .entry(identity)
                .or_default()
                .insert(key);
        }
        if let Some(rect) = damage {
            self.damage_bounds.insert(key, rect);
        }
        self.pending_damage = pending_damage;
        self.family_counts[family_slot(family)] += 1;
        self.counters.inserted_commands += 1;
        Ok(key)
    }

    pub(crate) fn replace(
        &mut self,
        command: UiNativeAppearanceCommand,
    ) -> Result<UiNativeAppearanceCommandKey, UiNativeAppearanceRetainedDenial> {
        let identity = command.identity();
        let key = *self
            .identities
            .get(&identity)
            .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?;
        self.replace_key(key, command)
    }

    pub(crate) fn replace_key(
        &mut self,
        key: UiNativeAppearanceCommandKey,
        command: UiNativeAppearanceCommand,
    ) -> Result<UiNativeAppearanceCommandKey, UiNativeAppearanceRetainedDenial> {
        let identity = command.identity();
        if self
            .identities
            .get(&identity)
            .is_some_and(|existing| *existing != key)
        {
            return Err(UiNativeAppearanceRetainedDenial::DuplicateIdentity);
        }
        let old_family = self
            .commands
            .get(&key)
            .map(UiNativeAppearanceCommand::family)
            .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?;
        if old_family != command.family() {
            return Err(UiNativeAppearanceRetainedDenial::IdentityFamilyMismatch);
        }
        let old_damage = self.damage_bounds.get(&key).copied();
        let new_damage = command
            .damage_rect(self.scale)
            .map_err(UiNativeAppearanceRetainedDenial::Geometry)?;
        let mut pending_damage = self.pending_damage.clone();
        let old_identity = self
            .commands
            .get(&key)
            .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?
            .identity();
        record_damage(
            self.commands
                .get(&key)
                .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?,
            old_damage,
            &mut pending_damage,
        )?;
        record_damage(&command, new_damage, &mut pending_damage)?;
        if let (
            Some(UiNativeAppearanceCommand::OverlayOrder(previous)),
            UiNativeAppearanceCommand::OverlayOrder(successor),
        ) = (self.commands.get(&key), &command)
        {
            record_overlay_order_damage(
                previous,
                successor,
                &self.identities,
                &self.damage_bounds,
                &mut pending_damage,
            )?;
        }
        self.replace_damage_index(key, old_damage, new_damage)?;
        let old_text_commands = self
            .commands
            .get(&key)
            .map(text_paint_commands)
            .unwrap_or_default();
        let new_text_commands = text_paint_commands(&command);
        self.commands.insert(key, command);
        self.remove_text_key(key, &old_text_commands);
        for identity in new_text_commands {
            self.text_keys_by_paint_command
                .entry(identity)
                .or_default()
                .insert(key);
        }
        if old_identity != identity {
            self.identities.remove(&old_identity);
            self.identities.insert(identity, key);
        }
        match new_damage {
            Some(rect) => {
                self.damage_bounds.insert(key, rect);
            }
            None => {
                self.damage_bounds.remove(&key);
            }
        }
        self.pending_damage = pending_damage;
        self.counters.replaced_commands += 1;
        Ok(key)
    }

    fn replace_damage_index(
        &mut self,
        key: UiNativeAppearanceCommandKey,
        old_damage: Option<UiNativeAppearanceDamageRect>,
        new_damage: Option<UiNativeAppearanceDamageRect>,
    ) -> Result<(), UiNativeAppearanceRetainedDenial> {
        match (old_damage, new_damage) {
            (Some(_), Some(new)) => self
                .damage_index
                .replace(
                    key,
                    candidate_bounds(new).map_err(UiNativeAppearanceRetainedDenial::Geometry)?,
                )
                .map_err(map_index_denial),
            (Some(_), None) => self.damage_index.remove(key).map_err(map_index_denial),
            (None, Some(new)) => self
                .damage_index
                .insert(
                    key,
                    candidate_bounds(new).map_err(UiNativeAppearanceRetainedDenial::Geometry)?,
                )
                .map_err(map_index_denial),
            (None, None) => Ok(()),
        }
    }

    pub(crate) fn remove(
        &mut self,
        key: UiNativeAppearanceCommandKey,
    ) -> Result<(), UiNativeAppearanceRetainedDenial> {
        let command = self
            .commands
            .get(&key)
            .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?;
        let damage = self.damage_bounds.get(&key).copied();
        let mut pending_damage = self.pending_damage.clone();
        record_damage(command, damage, &mut pending_damage)?;
        if damage.is_some() {
            self.damage_index.remove(key).map_err(map_index_denial)?;
        }
        self.order
            .remove(key)
            .map_err(UiNativeAppearanceRetainedDenial::Order)?;
        let identity = command.identity();
        let family = command.family();
        let text_commands = text_paint_commands(command);
        self.commands.remove(&key);
        if family == UiNativeAppearanceCommandFamily::OverlayOrder {
            self.overlay_order = None;
        }
        self.remove_text_key(key, &text_commands);
        self.identities.remove(&identity);
        self.damage_bounds.remove(&key);
        self.pending_damage = pending_damage;
        self.family_counts[family_slot(family)] -= 1;
        self.counters.removed_commands += 1;
        Ok(())
    }

    pub(crate) fn place_after(
        &mut self,
        key: UiNativeAppearanceCommandKey,
        predecessor: Option<UiNativeAppearanceCommandKey>,
    ) -> Result<(), UiNativeAppearanceRetainedDenial> {
        if !self.commands.contains_key(&key) {
            return Err(UiNativeAppearanceRetainedDenial::MissingIdentity);
        }
        self.order
            .place_after(key, predecessor)
            .map_err(UiNativeAppearanceRetainedDenial::Order)?;
        self.counters.order_edits += 1;
        Ok(())
    }

    pub(crate) fn take_damage(&mut self) -> Box<[UiNativeAppearanceDamageRect]> {
        self.pending_damage.take()
    }
    pub(crate) fn command(
        &self,
        key: UiNativeAppearanceCommandKey,
    ) -> Option<&UiNativeAppearanceCommand> {
        self.commands.get(&key)
    }

    pub(crate) fn key_for_identity(
        &self,
        identity: &UiNativeAppearanceCommandIdentity,
    ) -> Option<UiNativeAppearanceCommandKey> {
        self.identities.get(identity).copied()
    }

    pub(crate) fn counters(&self) -> UiNativeAppearanceRetentionCounters {
        self.counters
    }

    fn check_capacity(
        &self,
        family: UiNativeAppearanceCommandFamily,
    ) -> Result<(), UiNativeAppearanceRetainedDenial> {
        let profile = crate::native_profile::APPEARANCE_PROFILE;
        if self.commands.len()
            >= usize::from(crate::native_profile::APPEARANCE_PROFILE.retained_commands)
        {
            return Err(UiNativeAppearanceRetainedDenial::CapacityExceeded);
        }
        if usize::from(self.family_counts[family_slot(family)]) >= family_capacity(family, profile)
        {
            return Err(UiNativeAppearanceRetainedDenial::FamilyCapacityExceeded);
        }
        Ok(())
    }
}
