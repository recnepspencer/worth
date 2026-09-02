impl super::UiPortalRuntimeState {
    /// Removes Portal scopes whose exact mounted owner disappeared during a
    /// published graph replacement. A removed parent also removes every
    /// descendant, even when a descendant's mounted owner independently
    /// survived, because ancestry is Portal-owned truth.
    pub(crate) fn remove_rebound_portals(
        &mut self,
        successor: &crate::mounting::UiMountedIdentityView,
    ) -> Box<[super::UiPortalIdentity]> {
        let removed_roots = self
            .records
            .keys()
            .copied()
            .filter(|portal| {
                let owner = portal.owner();
                !successor.mounted_instances().iter().any(|instance| {
                    instance.identity() == owner.mounted_instance_identity()
                        && instance.graph_node_identity() == owner.graph_node()
                })
            })
            .collect::<Vec<_>>();
        let removed = self
            .records
            .keys()
            .copied()
            .filter(|portal| {
                removed_roots
                    .iter()
                    .any(|root| *portal == *root || self.portal_descends_from(*portal, *root))
            })
            .collect::<Vec<_>>();
        for portal in &removed {
            if let Some(record) = self.records.remove(portal) {
                self.stack_order.remove(record.stack_ordinal, *portal);
            }
        }
        if !removed.is_empty() {
            self.revision = self
                .revision
                .checked_add(1)
                .expect("Portal rebind revision remains representable");
        }
        removed.into_boxed_slice()
    }

    pub(crate) fn rebind_published_presentations(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        surfaces: &[crate::mounting::UiMountedSurfacePresentationReceipt],
    ) {
        for record in self.records.values_mut().filter(|record| {
            record.posture != super::UiPortalLifecyclePosture::Closed && record.placement.is_some()
        }) {
            let placement = record.placement.expect("filtered portal retains placement");
            let predecessor = placement.prepared().presentation();
            let surface = surfaces
                .iter()
                .find(|surface| surface.host_surface() == predecessor.host_surface())
                .expect("published portal overlay retains its exact host surface");
            let presentation = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
                surface.host_surface(),
                frame,
                surface.binding(),
                surface.epoch(),
            );
            record.placement = Some(super::UiCommittedPortalPlacement::from_prepared(
                placement.prepared().with_presentation(presentation),
            ));
        }
    }
}
