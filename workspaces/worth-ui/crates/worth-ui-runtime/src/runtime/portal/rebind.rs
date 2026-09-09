#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalRebindRemovalDenial {
    StalePlan,
}

pub(crate) struct UiPreparedPortalRebindRemoval {
    expected_revision: u64,
    removed: Box<[super::UiPortalIdentity]>,
}

impl UiPreparedPortalRebindRemoval {
    pub(crate) fn removed(&self) -> &[super::UiPortalIdentity] {
        &self.removed
    }
}

impl super::UiPortalRuntimeState {
    /// Prepares removal of Portal scopes whose exact mounted owner disappeared
    /// during a published graph replacement. A removed parent also removes
    /// every descendant, even when a descendant's mounted owner independently
    /// survived, because ancestry is Portal-owned truth.
    pub(crate) fn prepare_rebound_portal_removal(
        &self,
        successor: &crate::mounting::UiMountedIdentityView,
        remove_all: bool,
    ) -> UiPreparedPortalRebindRemoval {
        let removed = if remove_all {
            self.records.keys().copied().collect::<Vec<_>>()
        } else {
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
            self.records
                .keys()
                .copied()
                .filter(|portal| {
                    removed_roots
                        .iter()
                        .any(|root| *portal == *root || self.portal_descends_from(*portal, *root))
                })
                .collect::<Vec<_>>()
        };
        UiPreparedPortalRebindRemoval {
            expected_revision: self.revision,
            removed: removed.into_boxed_slice(),
        }
    }

    pub(crate) fn commit_rebound_portal_removal(
        &mut self,
        prepared: UiPreparedPortalRebindRemoval,
    ) -> Result<Box<[super::UiPortalIdentity]>, UiPortalRebindRemovalDenial> {
        self.validate_rebound_portal_removal(&prepared)?;
        for portal in &prepared.removed {
            if let Some(record) = self.records.remove(portal) {
                self.stack_order.remove(record.stack_ordinal, *portal);
                self.remove_surface_stack_row(record.semantic_surface, *portal);
            }
        }
        if !prepared.removed.is_empty() {
            self.revision = self
                .revision
                .checked_add(1)
                .expect("Portal rebind revision remains representable");
        }
        Ok(prepared.removed)
    }

    pub(crate) fn validate_rebound_portal_removal(
        &self,
        prepared: &UiPreparedPortalRebindRemoval,
    ) -> Result<(), UiPortalRebindRemovalDenial> {
        if prepared.expected_revision != self.revision
            || prepared
                .removed
                .iter()
                .any(|portal| !self.records.contains_key(portal))
        {
            return Err(UiPortalRebindRemovalDenial::StalePlan);
        }
        Ok(())
    }

    /// Rebinds Portal presentation bases without changing Portal membership.
    pub(crate) fn rebind_published_presentations(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        surfaces: &[crate::mounting::UiMountedSurfacePresentationReceipt],
    ) {
        for record in self.records.values_mut().filter(|record| {
            record.posture != super::UiPortalLifecyclePosture::Closed && record.placement.is_some()
        }) {
            let placement = record.placement.expect("filtered portal retains placement");
            let Some(surface) = surfaces
                .iter()
                .find(|surface| surface.semantic_surface() == record.semantic_surface)
            else {
                // Partial publications leave omitted surfaces at their accepted basis.
                continue;
            };
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
