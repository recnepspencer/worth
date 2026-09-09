use super::{portal_child_view::UiMountedPortalChildPresentation, UiMountedProjectionFrame};

impl UiMountedProjectionFrame {
    pub(in crate::mounting) fn portal_presentation_affinity_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Option<worth_ui_host_contract::UiMountedPortalPresentationAffinity> {
        match self
            .portal_child_presentation(instance, surface, binding)
            .expect("prepared Portal children retain an unambiguous mounted owner")
        {
            UiMountedPortalChildPresentation::Presented(portal) => Some(
                worth_ui_host_contract::UiMountedPortalPresentationAffinity::from_runtime_mounting(
                    portal.owner(),
                    portal.portal_identity(),
                ),
            ),
            UiMountedPortalChildPresentation::Ordinary
            | UiMountedPortalChildPresentation::Suppressed => None,
        }
    }

    pub(in crate::mounting) fn visual_region_basis(
        &self,
    ) -> crate::mounting::UiMountedVisualRegionBasis {
        let mut portal_children = std::collections::BTreeMap::new();
        for instance in self.semantic.order.iter().copied() {
            let Some(node) = self.semantic.node(instance) else {
                continue;
            };
            if node.portal_child_owner.is_none() {
                continue;
            }
            let Some(surface) = self.semantic.surface_for(node.receipt.semantic_surface()) else {
                continue;
            };
            match self
                .portal_child_presentation(instance, surface.surface, surface.binding)
                .expect("prepared Portal children retain an unambiguous mounted owner")
            {
                UiMountedPortalChildPresentation::Ordinary => {}
                UiMountedPortalChildPresentation::Suppressed => {
                    portal_children.insert(instance, None);
                }
                UiMountedPortalChildPresentation::Presented(portal) => {
                    portal_children.insert(instance, Some(portal));
                }
            }
        }
        self.mechanics
            .visual_region_basis()
            .with_portal_overlays(self.portal_overlay_visual_rows())
            .with_portal_children(portal_children)
    }

    pub(in crate::mounting) fn presentation_commands_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> std::sync::Arc<[worth_ui_host_contract::UiMountedPaintCommand]> {
        let mut commands = self
            .mechanics
            .commands_for_instance(instance, surface, binding)
            .to_vec();
        commands = match self
            .portal_child_presentation(instance, surface, binding)
            .expect("prepared Portal children retain an unambiguous mounted owner")
        {
            UiMountedPortalChildPresentation::Ordinary => commands,
            UiMountedPortalChildPresentation::Suppressed => Vec::new(),
            UiMountedPortalChildPresentation::Presented(portal) => commands
                .into_iter()
                .filter_map(|command| present_portal_child_command(command, portal))
                .collect(),
        };
        for input in self
            .portal_overlays
            .iter()
            .copied()
            .filter(|input| input.owner() == instance)
        {
            let owner = self
                .semantic
                .node(instance)
                .expect("prepared Portal overlay retains its mounted owner");
            if owner.receipt.semantic_surface() != surface {
                continue;
            }
            let receipt = self
                .receipt_basis
                .receipt_for(instance)
                .expect("prepared Portal overlay retains its mounted receipt");
            let mechanic = input
                .mechanic_for(self.frame, surface, binding, receipt)
                .expect("prepared Portal overlay retains valid geometry");
            commands.push(
                worth_ui_host_contract::UiMountedPaintCommand::PortalOverlay {
                    identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(
                        &mechanic,
                    ),
                    mechanic,
                },
            );
        }
        commands.sort_by_key(worth_ui_host_contract::UiMountedPaintCommand::layer_semantic_order);
        commands.into()
    }

    pub(in crate::mounting) fn has_precise_command_delta(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> bool {
        self.precise_command_instances.contains(&instance)
    }

    pub(in crate::mounting) fn presentation_command_changes(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedPaintCommandChange] {
        if !self.portal_overlays_changed
            && self.precise_command_instances.len() == self.changed_instances.len()
        {
            &self.presentation_command_changes
        } else {
            &[]
        }
    }

    pub(in crate::mounting) fn presentation_instance_order(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> crate::runtime::persistent_index::UiPersistentOrder<
        worth_ui_host_contract::UiMountedInstanceIdentity,
    > {
        let mut presented = crate::runtime::persistent_index::UiPersistentOrder::default();
        for instance in self.semantic.order.iter().copied() {
            let Some(node) = self.semantic.node(instance) else {
                continue;
            };
            if node.receipt.semantic_surface() != surface
                || matches!(
                    self.portal_child_presentation(instance, surface, binding)
                        .expect("prepared Portal children retain an unambiguous mounted owner"),
                    UiMountedPortalChildPresentation::Suppressed
                )
            {
                continue;
            }
            presented
                .append(instance)
                .expect("presented mounted identities remain unique and bounded");
        }
        presented
    }
}

fn present_portal_child_command(
    command: worth_ui_host_contract::UiMountedPaintCommand,
    portal: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
) -> Option<worth_ui_host_contract::UiMountedPaintCommand> {
    Some(match command {
        worth_ui_host_contract::UiMountedPaintCommand::FilledRect { mechanic, .. } => {
            let mechanic = mechanic
                .presented_within_portal(portal)
                .expect("validated Portal-relative paint remains canonical")?;
            worth_ui_host_contract::UiMountedPaintCommand::FilledRect {
                identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::filled_rect(
                    &mechanic,
                ),
                mechanic,
            }
        }
        worth_ui_host_contract::UiMountedPaintCommand::SemanticText { mechanic, .. } => {
            let mechanic = mechanic
                .presented_within_portal(portal)
                .expect("validated Portal-relative text remains canonical")?;
            worth_ui_host_contract::UiMountedPaintCommand::SemanticText {
                identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text(
                    &mechanic,
                ),
                mechanic,
            }
        }
        worth_ui_host_contract::UiMountedPaintCommand::PortalOverlay { .. } => {
            unreachable!("Portal children cannot own nested overlay commands here")
        }
    })
}
