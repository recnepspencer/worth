#[path = "retained/damage.rs"]
mod damage;
use damage::record_damage;
#[path = "retained/text_paint.rs"]
mod text_paint;
#[path = "retained/text_transaction.rs"]
mod text_transaction;
pub(crate) use text_transaction::UiNativeTextCoverageUndo;

use std::collections::BTreeMap;

use super::super::damage_index::{UiNativeDamageIndex, UiNativeDamageIndexDenial};
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
}

pub(crate) struct UiNativeAppearanceRetained {
    scale: UiNativeAppearanceScale,
    commands: BTreeMap<UiNativeAppearanceCommandKey, UiNativeAppearanceCommand>,
    identities: BTreeMap<UiNativeAppearanceCommandIdentity, UiNativeAppearanceCommandKey>,
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
            order: UiNativeRetainedOrder::initial([])
                .expect("the empty staged retained order is always admissible"),
            damage_index: UiNativeDamageIndex::new(),
            damage_bounds: BTreeMap::new(),
            pending_damage: UiNativeAppearanceDamage::new(usize::from(
                crate::native_profile::STAGED_APPEARANCE_PROFILE.damage_regions,
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

    pub(crate) fn insert(
        &mut self,
        command: UiNativeAppearanceCommand,
        predecessor: Option<UiNativeAppearanceCommandKey>,
    ) -> Result<UiNativeAppearanceCommandKey, UiNativeAppearanceRetainedDenial> {
        let identity = command.identity();
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
        self.identities.insert(identity, key);
        self.commands.insert(key, command);
        if let Some(rect) = damage {
            self.damage_bounds.insert(key, rect);
        }
        self.pending_damage = pending_damage;
        self.family_counts[family_slot(command_family(&self.commands, key))] += 1;
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
        let previous = self
            .commands
            .get(&key)
            .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?;
        record_damage(previous, old_damage, &mut pending_damage)?;
        record_damage(&command, new_damage, &mut pending_damage)?;
        self.replace_damage_index(key, old_damage, new_damage)?;
        self.commands.insert(key, command);
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
        self.commands.remove(&key);
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

    pub(crate) fn replay_for_damage(
        &mut self,
        damage: UiNativeAppearanceDamageRect,
    ) -> Result<Box<[UiNativeAppearanceCommandKey]>, UiNativeAppearanceRetainedDenial> {
        if damage.is_empty() {
            return Err(UiNativeAppearanceRetainedDenial::EmptyDamage);
        }
        let query = self
            .damage_index
            .intersecting(
                candidate_bounds(damage).map_err(UiNativeAppearanceRetainedDenial::Geometry)?,
            )
            .map_err(map_index_denial)?;
        self.counters.damage_queries += 1;
        self.counters.damage_branch_probes += query.branch_aabb_probes;
        self.counters.damage_leaf_probes += query.leaf_command_bounds_probes;
        let candidates = query.identities.into_iter().filter(|key| {
            self.commands.get(key).is_some_and(|command| {
                command.text_coverage().map_or_else(
                    || {
                        self.damage_bounds
                            .get(key)
                            .is_some_and(|bounds| bounds.intersects(damage))
                    },
                    |regions| regions.iter().any(|region| region.intersects(damage)),
                )
            })
        });
        let ordered = self
            .order
            .ordered_subset(candidates)
            .map_err(UiNativeAppearanceRetainedDenial::Order)?;
        self.counters.replayed_commands += ordered.len();
        Ok(ordered.into_boxed_slice())
    }

    pub(crate) fn take_damage(&mut self) -> Box<[UiNativeAppearanceDamageRect]> {
        self.pending_damage.take()
    }
    pub(crate) fn ordered_keys(&self) -> Box<[UiNativeAppearanceCommandKey]> {
        self.order.ordered().collect::<Vec<_>>().into_boxed_slice()
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
        let profile = crate::native_profile::STAGED_APPEARANCE_PROFILE;
        if self.commands.len()
            >= usize::from(crate::native_profile::STAGED_APPEARANCE_PROFILE.retained_commands)
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

fn family_slot(family: UiNativeAppearanceCommandFamily) -> usize {
    match family {
        UiNativeAppearanceCommandFamily::Surface => 0,
        UiNativeAppearanceCommandFamily::PortalSurface => 0,
        UiNativeAppearanceCommandFamily::Outline => 1,
        UiNativeAppearanceCommandFamily::TextForeground => 2,
        UiNativeAppearanceCommandFamily::Backdrop => 3,
        UiNativeAppearanceCommandFamily::OverlayOrder => 4,
        UiNativeAppearanceCommandFamily::PointerAffordance => 5,
    }
}

fn family_capacity(
    family: UiNativeAppearanceCommandFamily,
    profile: crate::native_profile::UiNativeStagedAppearanceProfile,
) -> usize {
    match family {
        UiNativeAppearanceCommandFamily::Surface
        | UiNativeAppearanceCommandFamily::PortalSurface => usize::from(profile.surface_commands),
        UiNativeAppearanceCommandFamily::Outline => usize::from(profile.outline_commands),
        UiNativeAppearanceCommandFamily::TextForeground => {
            usize::from(profile.text_foreground_commands)
        }
        UiNativeAppearanceCommandFamily::Backdrop => usize::from(profile.backdrop_commands),
        UiNativeAppearanceCommandFamily::OverlayOrder => {
            usize::from(profile.overlay_order_commands)
        }
        UiNativeAppearanceCommandFamily::PointerAffordance => {
            usize::from(profile.pointer_affordance_commands)
        }
    }
}

fn command_family(
    commands: &BTreeMap<UiNativeAppearanceCommandKey, UiNativeAppearanceCommand>,
    key: UiNativeAppearanceCommandKey,
) -> UiNativeAppearanceCommandFamily {
    commands
        .get(&key)
        .expect("a committed staged command has a retained family")
        .family()
}

fn map_index_denial(denial: UiNativeDamageIndexDenial) -> UiNativeAppearanceRetainedDenial {
    match denial {
        UiNativeDamageIndexDenial::CapacityExceeded => {
            UiNativeAppearanceRetainedDenial::DamageIndexCapacityExceeded
        }
        UiNativeDamageIndexDenial::DuplicateIdentity => {
            UiNativeAppearanceRetainedDenial::DuplicateIdentity
        }
        UiNativeDamageIndexDenial::MissingIdentity => {
            UiNativeAppearanceRetainedDenial::MissingIdentity
        }
    }
}
