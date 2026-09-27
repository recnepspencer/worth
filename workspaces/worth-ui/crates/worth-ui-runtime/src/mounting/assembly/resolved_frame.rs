use super::{UiAssembledMountedFrame, UiMountedFramePreparationDenial};

pub struct UiPreparedMountedFrame {
    frame: UiAssembledMountedFrame,
    motion_entrance: Option<crate::runtime::motion::UiPreparedMotionEntrance>,
    direct_scroll: Box<[crate::runtime::scroll::UiPreparedScrollDirectSuccession]>,
    scroll_hit_succession: crate::mounting::UiHitTestSpatialWork,
}

impl UiAssembledMountedFrame {
    pub(crate) fn prepared_theme_binding(
        &self,
    ) -> Option<&crate::runtime::appearance::UiActiveThemeBinding> {
        self.prepared_theme_binding.as_ref()
    }

    pub(crate) fn resolve_appearance(
        mut self,
        projection: crate::facade::entry::appearance_projection::UiAppearanceFrameProjection<'_>,
    ) -> Result<UiPreparedMountedFrame, UiMountedFramePreparationDenial> {
        let basis = crate::graph::UiGraphFactIndexBasis::from_generation(
            projection.graph.snapshot(),
            projection.capabilities,
        );
        if !self
            .candidate
            .owner
            .appearance()
            .matches_consumer_basis(basis)
        {
            return Err(UiMountedFramePreparationDenial::Projection(
                crate::mounting::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch,
            ));
        }
        if let crate::facade::entry::appearance_projection::UiAppearanceProjectionPhase::ThemeSwitch(change) = projection.phase {
            projection.presentation.appearance_theme_state()
                .ok_or(UiMountedFramePreparationDenial::AppearanceThemeSwitch(
                    crate::runtime::appearance::UiThemeSwitchDenial::MissingActiveBinding))?
                .validate_prepared_switch(change.prepared())
                .map_err(UiMountedFramePreparationDenial::AppearanceThemeSwitch)?;
            self.prepared_theme_binding = Some(change.prepared().successor().clone());
        }
        projection.finish(&mut self)?;
        Ok(UiPreparedMountedFrame {
            frame: self,
            motion_entrance: None,
            direct_scroll: Box::default(),
            scroll_hit_succession: Default::default(),
        })
    }
}

impl std::ops::Deref for UiPreparedMountedFrame {
    type Target = UiAssembledMountedFrame;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl UiPreparedMountedFrame {
    pub(in crate::mounting) fn record_scroll_hit_succession(
        &mut self,
        work: crate::mounting::UiHitTestSpatialWork,
    ) {
        self.scroll_hit_succession.merge(work);
    }

    pub fn cost_report(&self) -> crate::mounting::UiMountCostReport {
        self.frame
            .cost_report()
            .with_additional_hit_work(self.scroll_hit_succession)
    }

    pub fn receipt(&self) -> super::UiMountedFrameReceipt {
        let mut receipt = self.frame.receipt();
        receipt.cost = self.cost_report();
        receipt
    }

    /// The hit row this frame presents for `target` on `surface`: its
    /// geometry once the host accepts it, before any Motion sample moves it.
    pub(crate) fn prepared_hit_row(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::UiPresentedHitTestRow> {
        let binding = self
            .surfaces()
            .iter()
            .map(super::UiMountedSurfaceReceipt::requirement)
            .find(|requirement| requirement.semantic_surface() == surface)?
            .binding();
        self.visual_region_basis()
            .hit_test()
            .into_vec()
            .into_iter()
            .find(|row| {
                row.mechanic().binding() == binding && row.mechanic().mounted_instance() == target
            })
            .map(crate::mounting::UiPresentedHitTestRow::from_mounted)
    }

    pub(in crate::mounting) fn bind_direct_scroll(
        &mut self,
        records: Box<[crate::runtime::scroll::UiPreparedScrollDirectSuccession]>,
    ) {
        self.direct_scroll = records;
    }

    pub(in crate::mounting) fn direct_scroll(
        &self,
    ) -> &[crate::runtime::scroll::UiPreparedScrollDirectSuccession] {
        &self.direct_scroll
    }

    /// Where this frame presents each open Portal, keyed by the stack ordinal
    /// of the open it was fitted for, for the Portal to commit when the frame
    /// is accepted.
    pub(in crate::mounting) fn portal_placements(
        &self,
    ) -> Box<
        [(
            crate::runtime::portal::UiPortalStackOrdinal,
            crate::runtime::portal::UiPreparedPortalPlacement,
        )],
    > {
        self.frame
            .candidate
            .owner
            .projection()
            .portal_overlay_inputs()
            .iter()
            .map(|input| (input.input_order().1, input.placement()))
            .collect()
    }

    pub(crate) fn bind_motion_entrance(
        &mut self,
        entrance: Option<crate::runtime::motion::UiPreparedMotionEntrance>,
    ) -> Result<(), crate::runtime::rebind::UiRebindPreparationDenial> {
        if entrance.is_some_and(|entrance| entrance.frame() != self.canonical_core().frame()) {
            return Err(
                crate::runtime::rebind::UiRebindPreparationDenial::ConsequenceFrameMismatch,
            );
        }
        self.motion_entrance = entrance;
        Ok(())
    }

    pub(crate) const fn motion_entrance(
        &self,
    ) -> Option<crate::runtime::motion::UiPreparedMotionEntrance> {
        self.motion_entrance
    }

    pub(crate) fn visual_region_basis(&self) -> crate::mounting::UiMountedVisualRegionBasis {
        let mut basis = self.frame.visual_region_basis();
        basis = basis.with_direct_scroll_reservation(self.direct_scroll.len());
        if let Some(entrance) = self.motion_entrance {
            basis.presented_hits.prepare_motion_entrance(entrance);
        }
        basis
    }

    pub(in crate::mounting) fn reconcile_current(
        identity: &crate::mounting::UiMountedIdentityState,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        capability_report: &worth_ui_host_contract::WorthUiHostCapabilityReport,
    ) -> Result<Self, crate::mounting::UiMountedIdentityDenial> {
        let frame = identity.assemble_current_reconciliation_frame(
            replacements,
            protocol,
            capability_report,
        )?;
        Ok(Self {
            frame,
            motion_entrance: None,
            direct_scroll: Box::default(),
            scroll_hit_succession: Default::default(),
        })
    }

    pub(crate) fn record_accepted_motion_commands_visited(&mut self, count: usize) {
        self.frame.record_accepted_motion_commands_visited(count)
    }

    pub(crate) fn lower_appearance_with_motion_and_overlays(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
        overlays: &[crate::mounting::UiMountedAppearanceSurfaceOverlayInput],
        scroll_chrome: &[crate::mounting::UiPresented<
            crate::mounting::UiMountedAppearanceScrollChromeInput,
        >],
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        self.frame.lower_appearance_with_motion_and_overlays(
            presentation,
            profile,
            motion,
            overlays,
            scroll_chrome,
        )
    }

    pub(crate) fn deny_appearance_output(
        &mut self,
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        self.frame.deny_appearance_output()
    }

    pub(crate) fn into_publication_parts(
        self,
    ) -> (
        crate::mounting::UiProjectedMountedFrameCandidate,
        worth_ui_host_contract::UiMountedFrameManifest,
        worth_ui_host_contract::UiMountedFrameCanonicalCore,
        crate::mounting::UiMountedFrameReuseContract,
    ) {
        self.frame.into_publication_parts()
    }

    pub(in crate::mounting) fn reserve_appearance_retry_basis(&mut self) {
        self.frame.reserve_appearance_retry_basis()
    }

    pub(in crate::mounting) fn restore_rejected_appearance(&mut self) {
        self.frame.restore_rejected_appearance()
    }

    pub(crate) fn prepare_appearance_reconstruction(
        &mut self,
    ) -> Result<(), crate::mounting::projection::UiMountedProjectionDenial> {
        self.frame.prepare_appearance_reconstruction()
    }
}

#[cfg(test)]
mod test_access;
