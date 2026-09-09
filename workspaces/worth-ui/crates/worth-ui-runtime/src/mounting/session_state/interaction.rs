use super::WorthUiMountedSessionState;

impl WorthUiMountedSessionState {
    pub(crate) fn current_presented_hit_row(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Result<crate::mounting::UiPresentedHitTestRow, crate::mounting::UiPresentedFrameBasisDenial>
    {
        self.current_semantic_surface_for_presentation(presentation)?;
        self.retention
            .current_presented_hit_row(presentation, instance, work)
    }

    pub(crate) fn current_presentation_for_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        let presentation = self.retention.current_presentation_for_surface(surface)?;
        (self
            .current_semantic_surface_for_presentation(presentation)
            .ok()
            == Some(surface))
        .then_some(presentation)
    }

    /// Retention admits the physical epoch; mounted identity admits the current
    /// frame and binding. Historical publication epochs cannot authorize Motion.
    pub(crate) fn current_semantic_surface_for_presentation(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        use crate::mounting::UiPresentedFrameBasisDenial as Denial;
        self.validate_current_frame(presentation.frame())
            .map_err(|_| Denial::Expired)?;
        self.validate_binding(presentation.binding())
            .map_err(|_| Denial::BindingNotPresented)?;
        if self
            .presentation
            .binding_requires_reconstruction(presentation.binding())
        {
            return Err(Denial::PresentationTruthUnavailable);
        }
        self.retention
            .current_semantic_surface_for_presentation(presentation)
    }

    pub(crate) fn classify_interaction_presentation(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        crate::mounting::UiPresentedFrameBasisRelation,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        if self
            .presentation
            .binding_requires_reconstruction(presentation.binding())
        {
            return Err(crate::mounting::UiPresentedFrameBasisDenial::PresentationTruthUnavailable);
        }
        self.retention.classify(presentation, None, None)
    }

    pub(crate) fn interaction_hit_test_candidates(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        point: [f64; 2],
    ) -> Result<
        crate::mounting::UiPresentedHitTestBasis,
        crate::mounting::UiPresentedPointLookupDenial,
    > {
        if self
            .presentation
            .binding_requires_reconstruction(presentation.binding())
        {
            return Err(crate::mounting::UiPresentedPointLookupDenial::Presentation(
                crate::mounting::UiPresentedFrameBasisDenial::PresentationTruthUnavailable,
            ));
        }
        let (relation, query) = self.retention.presented_hit_candidates(
            presentation,
            point,
            crate::mounting::spatial_index::UiMountedSpatialBudget {
                node_visits: 1024,
                candidates: 256,
            },
        )?;
        Ok(crate::mounting::UiPresentedHitTestBasis::from_candidates(
            presentation,
            relation,
            query,
        ))
    }
    pub(crate) fn current_presented_incarnation_receipt(
        &self,
        input: crate::mounting::UiMountedIncarnationAffinityInput,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        crate::mounting::UiCurrentHitTargetAffinityDenial,
    > {
        if self
            .current_semantic_surface_for_presentation(presentation)
            .ok()
            != Some(input.surface)
            || presentation.binding() != input.binding
        {
            return Err(crate::mounting::UiCurrentHitTargetAffinityDenial::PresentationNotCurrent);
        }
        self.identity
            .current_incarnation_receipt(input, presentation.frame())
    }

    pub(crate) fn interaction_hit_test_basis(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        crate::mounting::UiPresentedHitTestBasis,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        if self
            .presentation
            .binding_requires_reconstruction(presentation.binding())
        {
            return Err(crate::mounting::UiPresentedFrameBasisDenial::PresentationTruthUnavailable);
        }
        let mut basis = self.retention.interaction_hit_test_basis(presentation)?;
        basis.apply_motion_samples(&self.motion_sampling);
        Ok(basis)
    }

    pub(crate) fn semantic_focus_placement_basis(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        crate::mounting::UiPresentedHitTestBasis,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        self.retention.interaction_hit_test_basis(presentation)
    }

    pub(crate) fn admit_current_hit_target(
        &self,
        row: worth_ui_host_contract::UiMountedHitTestMechanic,
    ) -> Result<
        crate::mounting::UiCurrentHitTarget,
        crate::mounting::UiCurrentHitTargetAffinityDenial,
    > {
        self.identity.admit_current_hit_target(row)
    }

    pub(crate) fn admit_current_interaction_affinity(
        &self,
        input: crate::mounting::UiMountedInteractionAffinityInput,
    ) -> Result<
        crate::mounting::UiCurrentInteractionAffinity,
        crate::mounting::UiCurrentHitTargetAffinityDenial,
    > {
        self.identity.admit_current_interaction_affinity(input)
    }

    pub(crate) fn admit_current_mounted_incarnation_affinity(
        &self,
        input: crate::mounting::UiMountedIncarnationAffinityInput,
    ) -> Result<
        crate::mounting::UiCurrentInteractionAffinity,
        crate::mounting::UiCurrentHitTargetAffinityDenial,
    > {
        self.identity
            .admit_current_mounted_incarnation_affinity(input)
    }

    pub(crate) fn input_text_profile(
        &self,
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    ) -> Option<worth_ui_host_contract::UiTextProfileGeneration> {
        crate::runtime::interaction::targeting::require_current_target(self, target).ok()?;
        Some(self.identity.current_projection()?.input_text_profile())
    }
}
