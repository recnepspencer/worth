use super::UiMountedProjectionFrameOwner;

impl UiMountedProjectionFrameOwner {
    #[cfg(test)]
    pub(crate) fn lower_appearance(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.lower_appearance_with_motion(
            presentation,
            bindings,
            profile,
            crate::mounting::presentation::UiAcceptedAppearanceMotion::default(),
        )
    }

    #[cfg(test)]
    pub(crate) fn lower_appearance_with_motion(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.lower_appearance_with_motion_and_overlays(
            presentation,
            bindings,
            profile,
            motion,
            &[],
            &[],
        )
    }

    pub(crate) fn lower_appearance_with_motion_and_overlays(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
        overlays: &[crate::mounting::UiMountedAppearanceSurfaceOverlayInput],
        scroll_chrome: &[crate::mounting::UiMountedAppearanceScrollChromeInput],
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        let mut candidate = self.appearance.clone();
        if let Err(_denial) =
            candidate.stage_portal_ownership_changes(&self.projection, bindings, overlays)
        {
            self.unpublished_appearance =
                Err(super::UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
            return self.appearance.reject_unpublished_output(Vec::new());
        }
        if let Err(_denial) =
            candidate.stage_scroll_chrome_changes(&self.projection, bindings, scroll_chrome)
        {
            self.unpublished_appearance =
                Err(super::UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
            return self.appearance.reject_unpublished_output(Vec::new());
        }
        if overlays.is_empty()
            && self.projection.portal_overlay_inputs().is_empty()
            && !candidate.has_pending_lowering()
        {
            let records = Vec::new();
            match super::appearance_output::assemble(
                &self.projection,
                presentation,
                bindings,
                Vec::new(),
                Vec::new(),
                &self.pointer,
            ) {
                Ok((projection, pointers)) => {
                    self.appearance = candidate;
                    self.pointer = pointers;
                    self.unpublished_appearance = Ok(projection.map(std::rc::Rc::new));
                    return records;
                }
                Err(denial) => {
                    self.unpublished_appearance = Err(denial);
                    return self.appearance.reject_unpublished_output(records);
                }
            }
        }
        let geometry =
            crate::mounting::projection::appearance::UiMountedAppearanceGeometryScope::with_motion(
                bindings, profile, motion,
            )
            .with_scroll_chrome(scroll_chrome);
        let records = match candidate.lower(presentation, &geometry) {
            Ok(records) => records,
            Err(denial) => {
                self.unpublished_appearance = Err(denial);
                return self.appearance.reject_unpublished_output(Vec::new());
            }
        };
        if records.iter().any(|record| {
            matches!(
                record,
                crate::runtime::appearance::UiAppearanceInspectionRecord::Denial { denial, .. }
                    if denial.blocks_mounted_output()
            )
        }) {
            self.unpublished_appearance = Err(super::UiMountedAppearanceOutputDenial::NodeLowering);
            return self.appearance.reject_unpublished_output(records);
        }
        if let Err(denial) = candidate.lower_scroll_chrome_owners(
            self.projection.frame_identity(),
            presentation,
            &geometry,
        ) {
            self.unpublished_appearance = Err(denial);
            return self.appearance.reject_unpublished_output(records);
        }
        if let Err(_denial) =
            candidate.lower_retirements(self.projection.frame_identity(), presentation)
        {
            self.unpublished_appearance = Err(super::UiMountedAppearanceOutputDenial::NodeLowering);
            return self.appearance.reject_unpublished_output(records);
        }
        if let Err(denial) = candidate.admit_order(&self.projection) {
            self.unpublished_appearance =
                Err(super::UiMountedAppearanceOutputDenial::Order(denial));
            return self.appearance.reject_unpublished_output(records);
        }
        if let Err(_denial) =
            candidate.lower_overlays(&self.projection, presentation, &geometry, overlays)
        {
            self.unpublished_appearance = Err(super::UiMountedAppearanceOutputDenial::NodeLowering);
            return self.appearance.reject_unpublished_output(records);
        }
        match super::appearance_output::assemble(
            &self.projection,
            presentation,
            bindings,
            candidate.take_node_work(),
            candidate.take_overlay_work(),
            &self.pointer,
        ) {
            Ok((projection, pointers)) => {
                self.appearance = candidate;
                self.pointer = pointers;
                self.unpublished_appearance = Ok(projection.map(std::rc::Rc::new));
                records
            }
            Err(denial) => {
                self.unpublished_appearance = Err(denial);
                self.appearance.reject_unpublished_output(records)
            }
        }
    }

    pub(crate) fn deny_appearance_output(
        &mut self,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.unpublished_appearance = Err(super::UiMountedAppearanceOutputDenial::NodeLowering);
        self.appearance.reject_unpublished_output(Vec::new())
    }
}
