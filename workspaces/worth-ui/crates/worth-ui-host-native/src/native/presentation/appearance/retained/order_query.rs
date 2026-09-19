use super::*;
use worth_ui_host_contract::UiOverlayParticipantIdentity;

impl UiNativeAppearanceRetained {
    pub(crate) fn overlay_order(
        &self,
    ) -> Result<
        &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
        UiNativeAppearanceRetainedDenial,
    > {
        self.overlay_order
            .and_then(|key| self.commands.get(&key))
            .and_then(|command| match command {
                UiNativeAppearanceCommand::OverlayOrder(order) => Some(order),
                _ => None,
            })
            .ok_or(UiNativeAppearanceRetainedDenial::OverlayOrderMismatch)
    }

    pub(crate) fn ordered_keys(&self) -> Box<[UiNativeAppearanceCommandKey]> {
        self.order.ordered().collect::<Vec<_>>().into_boxed_slice()
    }

    pub(crate) fn ordered_render_keys(
        &self,
    ) -> Result<Box<[UiNativeAppearanceCommandKey]>, UiNativeAppearanceRetainedDenial> {
        self.ordered_render_subset_keys(self.ordered_keys())
    }

    pub(crate) fn predecessor(
        &self,
        key: UiNativeAppearanceCommandKey,
    ) -> Result<Option<UiNativeAppearanceCommandKey>, UiNativeAppearanceRetainedDenial> {
        self.order
            .neighbors(key)
            .map(|(predecessor, _)| predecessor)
            .map_err(UiNativeAppearanceRetainedDenial::Order)
    }

    pub(crate) fn ordered_motion_keys(
        &self,
        direct: impl IntoIterator<Item = UiNativeAppearanceCommandIdentity>,
        text: impl IntoIterator<Item = worth_ui_host_contract::UiMountedPaintCommandIdentity>,
    ) -> Result<Box<[UiNativeAppearanceCommandKey]>, UiNativeAppearanceRetainedDenial> {
        let direct = direct
            .into_iter()
            .filter_map(|identity| self.key_for_identity(&identity));
        let text = text.into_iter().flat_map(|identity| {
            self.text_keys_by_paint_command
                .get(&identity)
                .into_iter()
                .flat_map(|keys| keys.iter().copied())
        });
        self.ordered_render_subset_keys(direct.chain(text))
    }

    pub(crate) fn ordered_subset_keys(
        &self,
        keys: impl IntoIterator<Item = UiNativeAppearanceCommandKey>,
    ) -> Result<Box<[UiNativeAppearanceCommandKey]>, UiNativeAppearanceRetainedDenial> {
        self.ordered_render_subset_keys(keys)
    }

    fn ordered_render_subset_keys(
        &self,
        keys: impl IntoIterator<Item = UiNativeAppearanceCommandKey>,
    ) -> Result<Box<[UiNativeAppearanceCommandKey]>, UiNativeAppearanceRetainedDenial> {
        let overlay_rank = self.overlay_order.and_then(|key| {
            self.commands.get(&key).and_then(|command| match command {
                UiNativeAppearanceCommand::OverlayOrder(order) => Some(
                    order
                        .bottom_to_top()
                        .iter()
                        .cloned()
                        .enumerate()
                        .map(|(rank, participant)| (participant, rank as u64))
                        .collect::<std::collections::BTreeMap<_, _>>(),
                ),
                _ => None,
            })
        });
        let mut result = Vec::new();
        for key in keys.into_iter().collect::<std::collections::BTreeSet<_>>() {
            let command = self
                .commands
                .get(&key)
                .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?;
            let order = match command {
                UiNativeAppearanceCommand::Surface(surface) => Some((
                    0_u8,
                    u64::from(surface.surface_paint_order()),
                    0_u8,
                    key.value(),
                )),
                UiNativeAppearanceCommand::Outline(outline) => {
                    Some((0, u64::from(outline.surface_paint_order()), 1, key.value()))
                }
                UiNativeAppearanceCommand::TextForeground(_) => Some((0, 0, 2, key.value())),
                UiNativeAppearanceCommand::PortalSurface(portal) => Some((
                    1,
                    *overlay_rank
                        .as_ref()
                        .ok_or(UiNativeAppearanceRetainedDenial::OverlayOrderMismatch)?
                        .get(&UiOverlayParticipantIdentity::Portal(
                            portal.portal_instance(),
                        ))
                        .ok_or(UiNativeAppearanceRetainedDenial::OverlayOrderMismatch)?,
                    0,
                    key.value(),
                )),
                UiNativeAppearanceCommand::Backdrop(backdrop) => Some((
                    1,
                    *overlay_rank
                        .as_ref()
                        .ok_or(UiNativeAppearanceRetainedDenial::OverlayOrderMismatch)?
                        .get(&UiOverlayParticipantIdentity::Backdrop(
                            backdrop.identity().clone(),
                        ))
                        .ok_or(UiNativeAppearanceRetainedDenial::OverlayOrderMismatch)?,
                    0,
                    key.value(),
                )),
                UiNativeAppearanceCommand::OverlayOrder(_)
                | UiNativeAppearanceCommand::PointerAffordance(_) => None,
            };
            if let Some(order) = order {
                result.push((order, key));
            }
        }
        result.sort_by_key(|entry| entry.0);
        Ok(result.into_iter().map(|(_, key)| key).collect())
    }
}
