//! Join appearance paint to the ordinary retained semantic command order.
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::appearance::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandIdentity, UiNativeAppearanceCommandKey,
};
use std::collections::{BTreeSet, HashSet};
use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPaintCommand, UiMountedPaintCommandIdentity,
    UiOverlayParticipantIdentity,
};

#[derive(Clone, Copy)]
pub(super) enum UiNativeRetainedRenderItem {
    Appearance(UiNativeAppearanceCommandKey),
    Paint(UiMountedPaintCommandIdentity),
}

impl UiNativeRetainedDrawList {
    pub(super) fn ordered_render_items(
        &self,
        paint: impl IntoIterator<Item = UiMountedPaintCommandIdentity>,
        appearance_keys: impl IntoIterator<Item = UiNativeAppearanceCommandKey>,
    ) -> Result<Box<[UiNativeRetainedRenderItem]>, Denial> {
        let (_, appearance) = self
            .staged_appearance
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let paint = paint.into_iter().collect::<HashSet<_>>();
        let mut appearance_keys = appearance_keys.into_iter().collect::<BTreeSet<_>>();
        for identity in &paint {
            match self.command(*identity).ok_or(Denial::CommandMismatch)? {
                UiMountedPaintCommand::FilledRect { mechanic, .. } => {
                    extend_if_present(
                        appearance,
                        &mut appearance_keys,
                        UiNativeAppearanceCommandIdentity::Surface(mechanic.mounted_instance()),
                    );
                    extend_if_present(
                        appearance,
                        &mut appearance_keys,
                        UiNativeAppearanceCommandIdentity::Outline(mechanic.mounted_instance()),
                    );
                }
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => extend_if_present(
                    appearance,
                    &mut appearance_keys,
                    UiNativeAppearanceCommandIdentity::PortalSurface(mechanic.owner()),
                ),
                UiMountedPaintCommand::SemanticText { .. } => {}
            }
        }

        let needs_overlay_order = paint.iter().any(|identity| {
            matches!(
                self.command(*identity),
                Some(UiMountedPaintCommand::PortalOverlay { .. })
            )
        }) || appearance_keys.iter().any(|key| {
            matches!(
                appearance.command(*key),
                Some(
                    UiNativeAppearanceCommand::PortalSurface(_)
                        | UiNativeAppearanceCommand::Backdrop(_)
                )
            )
        });
        let overlay = needs_overlay_order
            .then(|| appearance.overlay_order())
            .transpose()
            .map_err(|_| Denial::CommandMismatch)?;
        let mut entries = Vec::with_capacity(paint.len() + appearance_keys.len());
        for identity in paint {
            let command = self.command(identity).ok_or(Denial::CommandMismatch)?;
            let replaced = match command {
                UiMountedPaintCommand::FilledRect { mechanic, .. } => appearance
                    .key_for_identity(&UiNativeAppearanceCommandIdentity::Surface(
                        mechanic.mounted_instance(),
                    ))
                    .is_some(),
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => appearance
                    .key_for_identity(&UiNativeAppearanceCommandIdentity::PortalSurface(
                        mechanic.owner(),
                    ))
                    .is_some(),
                UiMountedPaintCommand::SemanticText { .. } => false,
            };
            if !replaced {
                entries.push((
                    (paint_rank(self, identity)?, 2_u8, 0_usize, 0_u32),
                    UiNativeRetainedRenderItem::Paint(identity),
                ));
            }
        }

        for key in appearance_keys {
            let command = appearance.command(key).ok_or(Denial::CommandMismatch)?;
            let order = match command {
                UiNativeAppearanceCommand::Surface(mechanic) => (
                    semantic_rank(self, mechanic.surface_paint_order())?,
                    0,
                    0,
                    key.value(),
                ),
                UiNativeAppearanceCommand::Outline(mechanic) => (
                    semantic_rank(self, mechanic.surface_paint_order())?,
                    1,
                    0,
                    key.value(),
                ),
                UiNativeAppearanceCommand::PortalSurface(mechanic) => (
                    portal_anchor_rank(self, mechanic.portal_instance())?,
                    1,
                    0,
                    key.value(),
                ),
                UiNativeAppearanceCommand::Backdrop(mechanic) => backdrop_order(
                    self,
                    overlay.ok_or(Denial::CommandMismatch)?,
                    mechanic.identity(),
                    key,
                )?,
                UiNativeAppearanceCommand::TextForeground(_)
                | UiNativeAppearanceCommand::OverlayOrder(_)
                | UiNativeAppearanceCommand::PointerAffordance(_) => continue,
            };
            entries.push((order, UiNativeRetainedRenderItem::Appearance(key)));
        }
        if let Some(overlay) = overlay {
            validate_portal_anchors(self, overlay)?;
        }
        entries.sort_by_key(|entry| entry.0);
        Ok(entries.into_iter().map(|(_, item)| item).collect())
    }

    pub(super) fn record_render_order_cost(
        &self,
        counters: &mut super::UiNativeRetainedMutationCounters,
    ) -> Result<(), Denial> {
        let order = self.order.take_cost();
        counters.order_index_lookups = counters
            .order_index_lookups
            .checked_add(order.identity_lookups())
            .ok_or(Denial::CounterOverflow)?;
        counters.order_index_node_touches = counters
            .order_index_node_touches
            .checked_add(order.node_touches())
            .ok_or(Denial::CounterOverflow)?;
        counters.order_index_rotations = counters
            .order_index_rotations
            .checked_add(order.rotations())
            .ok_or(Denial::CounterOverflow)?;
        counters.order_index_high_water = counters
            .order_index_high_water
            .max(order.high_water_entries());
        Ok(())
    }
}

fn extend_if_present(
    appearance: &crate::native::presentation::appearance::UiNativeAppearanceRetained,
    keys: &mut BTreeSet<UiNativeAppearanceCommandKey>,
    identity: UiNativeAppearanceCommandIdentity,
) {
    if let Some(key) = appearance.key_for_identity(&identity) {
        keys.insert(key);
    }
}

fn paint_rank(
    retained: &UiNativeRetainedDrawList,
    identity: UiMountedPaintCommandIdentity,
) -> Result<usize, Denial> {
    retained
        .order
        .rank(worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(identity))
        .ok_or(Denial::OrderMismatch)
}

fn semantic_rank(
    retained: &UiNativeRetainedDrawList,
    semantic_order: u32,
) -> Result<usize, Denial> {
    retained
        .order
        .first_with_weight_at_least(semantic_order)
        .map_or(Ok(retained.order.len()), |identity| {
            paint_rank(retained, identity.command())
        })
}

fn portal_anchor_rank(
    retained: &UiNativeRetainedDrawList,
    instance: UiMountedInstanceIdentity,
) -> Result<usize, Denial> {
    let mut matches = retained
        .commands
        .identities_for_instance(instance)
        .filter(|identity| {
            retained.command(*identity).is_some_and(|command| {
                matches!(command, UiMountedPaintCommand::PortalOverlay { .. })
            })
        });
    let identity = matches.next().ok_or(Denial::CommandMismatch)?;
    if matches.next().is_some() {
        return Err(Denial::CommandMismatch);
    }
    paint_rank(retained, identity)
}

fn backdrop_order(
    retained: &UiNativeRetainedDrawList,
    overlay: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
    identity: &worth_ui_host_contract::UiMountedBackdropIdentity,
    key: UiNativeAppearanceCommandKey,
) -> Result<(usize, u8, usize, u32), Denial> {
    let participant = UiOverlayParticipantIdentity::Backdrop(identity.clone());
    let position = overlay
        .bottom_to_top()
        .iter()
        .position(|candidate| candidate == &participant)
        .ok_or(Denial::CommandMismatch)?;
    let next_portal = overlay.bottom_to_top()[position + 1..]
        .iter()
        .find_map(|participant| match participant {
            UiOverlayParticipantIdentity::Portal(instance) => Some(*instance),
            UiOverlayParticipantIdentity::Backdrop(_) => None,
        });
    let rank = next_portal.map_or(Ok(retained.order.len()), |instance| {
        portal_anchor_rank(retained, instance)
    })?;
    Ok((rank, 0, position, key.value()))
}

fn validate_portal_anchors(
    retained: &UiNativeRetainedDrawList,
    overlay: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
) -> Result<(), Denial> {
    for participant in overlay.bottom_to_top() {
        if let UiOverlayParticipantIdentity::Portal(instance) = participant {
            portal_anchor_rank(retained, *instance)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "render_order_tests.rs"]
mod tests;
