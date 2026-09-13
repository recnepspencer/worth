//! Component attribution follows the same combined order as native rasterization.
use super::render_order::UiNativeRetainedRenderItem;
use super::{UiNativeRetainedDrawList, UiNativeRetainedPresentationAttribution as Attribution};
use crate::native::presentation::appearance::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandIdentity,
};
use worth_ui_host_contract::{UiMountedPaintCommand, UiMountedSurfacePaint};

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn top_paint_attribution(
        &self,
    ) -> Option<(usize, Attribution)> {
        let top = self.current_render_top();
        // A backdrop has no component owner. Only an empty render list may retain
        // the attribution of removed paint for the terminal-removal observation.
        let attributed = match top {
            Some((ordinal, item)) => self
                .attribution_for_item(item)
                .map(|value| (ordinal, value)),
            None => self.last_paint_attribution,
        };
        attributed.map(|(ordinal, mut value)| {
            value.node_receipt = self.regions.current_receipt(value.node_receipt);
            (ordinal, value)
        })
    }

    fn current_render_top(&self) -> Option<(usize, UiNativeRetainedRenderItem)> {
        if self.staged_appearance.is_some() {
            self.top_render_item()
                .expect("retained paint has validated combined render order")
        } else {
            self.order
                .ordered()
                .enumerate()
                .last()
                .map(|(ordinal, identity)| {
                    (
                        ordinal,
                        UiNativeRetainedRenderItem::Paint(identity.command()),
                    )
                })
        }
    }

    fn attribution_for_item(&self, item: UiNativeRetainedRenderItem) -> Option<Attribution> {
        match item {
            UiNativeRetainedRenderItem::Appearance(key) => {
                match self.staged_appearance.as_ref()?.1.command(key)? {
                    UiNativeAppearanceCommand::Surface(surface) => surface_attribution(surface),
                    UiNativeAppearanceCommand::PortalSurface(portal) => {
                        surface_attribution(portal.surface())
                    }
                    UiNativeAppearanceCommand::Outline(outline) => Some(Attribution {
                        color: rgba(outline.color()),
                        bounds: super::appearance_regions::canonical_visual(
                            outline.visual_bounds(),
                        )?,
                        mounted_instance: outline.node_receipt().mounted_instance(),
                        node_receipt: outline.node_receipt(),
                    }),
                    UiNativeAppearanceCommand::Backdrop(_)
                    | UiNativeAppearanceCommand::TextForeground(_)
                    | UiNativeAppearanceCommand::OverlayOrder(_)
                    | UiNativeAppearanceCommand::PointerAffordance(_) => None,
                }
            }
            UiNativeRetainedRenderItem::Paint(identity) => match self.command(identity)? {
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => Some(Attribution {
                    color: mechanic.color(),
                    bounds: mechanic.bounds(),
                    mounted_instance: mechanic.owner(),
                    node_receipt: mechanic.owner_receipt(),
                }),
                UiMountedPaintCommand::SemanticText { mechanic, .. } => {
                    let span = mechanic.foregrounds().first()?;
                    let adopted = self.staged_appearance.as_ref().and_then(|(_, appearance)| {
                        let key = appearance.key_for_identity(
                            &UiNativeAppearanceCommandIdentity::TextForeground {
                                target: mechanic.mounted_instance(),
                                command: identity.semantic_text_identity_parts()?,
                                span_digest: span.identity().digest(),
                            },
                        )?;
                        match appearance.command(key)? {
                            UiNativeAppearanceCommand::TextForeground(value) => {
                                Some(value.mechanic())
                            }
                            _ => None,
                        }
                    });
                    Some(Attribution {
                        color: adopted.map_or(span.color(), |value| rgba(value.foreground())),
                        bounds: mechanic.bounds(),
                        mounted_instance: mechanic.mounted_instance(),
                        node_receipt: adopted
                            .map_or(mechanic.node_receipt(), |value| value.node_receipt()),
                    })
                }
            },
        }
    }

    pub(super) fn retain_current_paint_attribution(&mut self) {
        if let Some((ordinal, item)) = self.current_render_top() {
            if let Some(value) = self.attribution_for_item(item) {
                self.last_paint_attribution = Some((ordinal, value));
            }
        }
    }
}

fn surface_attribution(
    surface: &worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic,
) -> Option<Attribution> {
    let color = match surface.paint() {
        UiMountedSurfacePaint::Fill(fill) | UiMountedSurfacePaint::FillAndBorder { fill, .. } => {
            *fill
        }
        UiMountedSurfacePaint::Border { color, .. } => *color,
    };
    Some(Attribution {
        color: rgba(color),
        bounds: super::appearance_regions::canonical_visual(surface.visual_bounds())?,
        mounted_instance: surface.node_receipt().mounted_instance(),
        node_receipt: surface.node_receipt(),
    })
}

fn rgba(
    color: worth_ui_host_contract::UiMountedAppearanceColor,
) -> worth_ui_host_contract::UiMountedRgba8 {
    let [r, g, b, a] = color.straight_srgba();
    worth_ui_host_contract::UiMountedRgba8::new(r, g, b, a)
}
