impl super::UiPortalRuntimeState {
    pub(crate) fn has_mounted_presentations(&self) -> bool {
        self.records.values().any(|record| {
            record.posture != super::super::UiPortalLifecyclePosture::Closed
                && record.placement.is_some()
        })
    }

    pub(crate) fn placement(
        &self,
        portal: super::super::UiPortalIdentity,
    ) -> Option<super::super::UiCommittedPortalPlacement> {
        self.records
            .get(&portal)
            .and_then(|record| record.placement)
    }

    pub(crate) fn topmost_presentation(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        self.records
            .values()
            .filter(|record| record.posture != super::super::UiPortalLifecyclePosture::Closed)
            .filter(|record| record.placement.is_some())
            .max_by_key(|record| record.stack_ordinal)
            .and_then(|record| record.placement)
            .map(|placement| placement.prepared().presentation())
    }

    pub(crate) fn exit_retention_presentation(
        &self,
        retention: super::super::UiPortalExitRetentionReceipt,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        self.records
            .get(&retention.portal())
            .filter(|record| {
                record.posture == super::super::UiPortalLifecyclePosture::Closing
                    && record.exit_retention == Some(retention)
            })
            .and_then(|record| record.placement)
            .map(|placement| placement.prepared().presentation())
    }

    pub(crate) fn mounted_projection_inputs(
        &self,
        transition: &super::super::UiPreparedPortalServiceTransition,
        retain_exit: bool,
    ) -> Vec<crate::mounting::UiMountedPortalOverlayProjectionInput> {
        let target = transition.portal();
        let mut inputs = self
            .records
            .iter()
            .filter_map(|(portal, record)| {
                ((!transition.closes(*portal) || retain_exit)
                    && record.posture != super::super::UiPortalLifecyclePosture::Closed)
                    .then_some(record.placement?)
                    .map(|placement| {
                        crate::mounting::UiMountedPortalOverlayProjectionInput::new(
                            portal.diagnostic_value(),
                            record.stack_ordinal,
                            portal.owner().mounted_instance_identity(),
                            record.semantic_surface,
                            placement.prepared(),
                            if transition.closes(*portal) {
                                super::super::UiPortalLifecyclePosture::Closing
                            } else {
                                record.posture
                            },
                        )
                    })
            })
            .collect::<Vec<_>>();
        if transition.opens_portal() {
            if let Some(placement) = transition.placement() {
                inputs.push(crate::mounting::UiMountedPortalOverlayProjectionInput::new(
                    target.diagnostic_value(),
                    transition
                        .stack_ordinal()
                        .expect("opening Portal carries issued stack order"),
                    target.owner().mounted_instance_identity(),
                    transition.request().semantic_surface(),
                    placement,
                    super::super::UiPortalLifecyclePosture::Visible,
                ));
            }
        }
        inputs
    }

    pub(crate) fn current_mounted_projection_inputs(
        &self,
    ) -> Vec<crate::mounting::UiMountedPortalOverlayProjectionInput> {
        self.records
            .iter()
            .filter_map(|(portal, record)| {
                (record.posture != super::super::UiPortalLifecyclePosture::Closed)
                    .then_some(record.placement?)
                    .map(|placement| {
                        crate::mounting::UiMountedPortalOverlayProjectionInput::new(
                            portal.diagnostic_value(),
                            record.stack_ordinal,
                            portal.owner().mounted_instance_identity(),
                            record.semantic_surface,
                            placement.prepared(),
                            record.posture,
                        )
                    })
            })
            .collect()
    }
}
