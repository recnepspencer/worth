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

// Ordinary content precedes the issued overlay stack. Within a Portal group,
// its surface precedes children, whose component rendering meaning orders them.
#[path = "render_order/item_order.rs"]
mod item_order;

type RenderOrderKey = (u8, usize, u8, u32, u8, usize, u32);

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
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => extend_if_present(
                    appearance,
                    &mut appearance_keys,
                    UiNativeAppearanceCommandIdentity::PortalSurface(mechanic.owner()),
                ),
                UiMountedPaintCommand::SemanticText { .. } => {}
            }
        }

        let needs_overlay_order = paint.iter().any(|identity| match self.command(*identity) {
            Some(UiMountedPaintCommand::PortalOverlay { .. }) => true,
            Some(UiMountedPaintCommand::SemanticText { mechanic, .. }) => {
                mechanic.portal_group().is_some()
            }
            None => false,
        }) || appearance_keys.iter().any(|key| {
            match appearance.command(*key) {
                Some(
                    UiNativeAppearanceCommand::PortalSurface(_)
                    | UiNativeAppearanceCommand::Backdrop(_),
                ) => true,
                Some(UiNativeAppearanceCommand::Surface(mechanic)) => {
                    mechanic.portal_group().is_some()
                }
                Some(UiNativeAppearanceCommand::Outline(mechanic)) => {
                    mechanic.portal_group().is_some()
                }
                _ => false,
            }
        });
        let overlay = needs_overlay_order
            .then(|| appearance.overlay_order())
            .transpose()
            .map_err(|_| Denial::CommandMismatch)?;
        let appearance_keys = appearance
            .ordered_subset_keys(appearance_keys)
            .map_err(|_| Denial::CommandMismatch)?;
        let mut entries = Vec::with_capacity(paint.len() + appearance_keys.len());
        for identity in paint {
            if let Some(entry) = self.paint_render_entry(appearance, overlay, identity)? {
                entries.push(entry);
            }
        }
        for (rank, key) in appearance_keys.iter().copied().enumerate() {
            if let Some(entry) = self.appearance_render_entry(appearance, overlay, rank, key)? {
                entries.push(entry);
            }
        }
        if let Some(overlay) = overlay {
            validate_portal_anchors(self, overlay)?;
        }
        entries.sort_by_key(|entry| entry.0);
        Ok(entries.into_iter().map(|(_, item)| item).collect())
    }

    pub(super) fn top_render_item(
        &self,
    ) -> Result<Option<(usize, UiNativeRetainedRenderItem)>, Denial> {
        let (_, appearance) = self
            .staged_appearance
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let keys = appearance
            .ordered_render_keys()
            .map_err(|_| Denial::CommandMismatch)?;
        let overlay = appearance.overlay_order().ok();
        let paint = self
            .order
            .ordered()
            .map(|identity| self.paint_render_entry(appearance, overlay, identity.command()));
        let appearance_entries = keys
            .iter()
            .copied()
            .enumerate()
            .map(|(rank, key)| self.appearance_render_entry(appearance, overlay, rank, key));
        let mut top = None;
        let mut count = 0;
        for entry in paint.chain(appearance_entries) {
            if let Some((key, item)) = entry? {
                count += 1;
                if top.as_ref().is_none_or(|(current, _)| key > *current) {
                    top = Some((key, item));
                }
            }
        }
        if let Some(overlay) = overlay {
            validate_portal_anchors(self, overlay)?;
        }
        Ok(top.map(|(_, item)| (count - 1, item)))
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

fn appearance_surface_order(
    retained: &UiNativeRetainedDrawList,
    overlay: Option<&worth_ui_host_contract::UiMountedOverlayOrderMechanic>,
    portal_group: Option<UiMountedInstanceIdentity>,
    semantic_order: u32,
    family: u8,
    appearance_rank: usize,
    key: UiNativeAppearanceCommandKey,
) -> Result<RenderOrderKey, Denial> {
    match portal_group {
        Some(portal) => Ok((
            1,
            portal_anchor_rank(retained, overlay, portal)?,
            1,
            semantic_order,
            family,
            appearance_rank,
            key.value(),
        )),
        None => Ok((
            0,
            semantic_rank(retained, semantic_order)?,
            0,
            0,
            0,
            appearance_rank,
            key.value(),
        )),
    }
}

fn portal_anchor_rank(
    retained: &UiNativeRetainedDrawList,
    overlay: Option<&worth_ui_host_contract::UiMountedOverlayOrderMechanic>,
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
    matches.next().ok_or(Denial::CommandMismatch)?;
    if matches.next().is_some() {
        return Err(Denial::CommandMismatch);
    }
    overlay
        .ok_or(Denial::CommandMismatch)?
        .bottom_to_top()
        .iter()
        .position(|participant| *participant == UiOverlayParticipantIdentity::Portal(instance))
        .ok_or(Denial::CommandMismatch)
}

fn backdrop_order(
    overlay: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
    identity: &worth_ui_host_contract::UiMountedBackdropIdentity,
    appearance_rank: usize,
    key: UiNativeAppearanceCommandKey,
) -> Result<RenderOrderKey, Denial> {
    let participant = UiOverlayParticipantIdentity::Backdrop(identity.clone());
    let position = overlay
        .bottom_to_top()
        .iter()
        .position(|candidate| candidate == &participant)
        .ok_or(Denial::CommandMismatch)?;
    Ok((1, position, 0, 0, 0, appearance_rank, key.value()))
}

fn validate_portal_anchors(
    retained: &UiNativeRetainedDrawList,
    overlay: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
) -> Result<(), Denial> {
    for participant in overlay.bottom_to_top() {
        if let UiOverlayParticipantIdentity::Portal(instance) = participant {
            portal_anchor_rank(retained, Some(overlay), *instance)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "render_order_tests.rs"]
mod tests;
