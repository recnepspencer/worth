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

    /// What a host acknowledgement last proved on screen for `surface`, while
    /// that surface still owns the binding it was presented on.
    pub(crate) fn current_presentation_for_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<crate::mounting::presentation::UiDisplayedSurfaceBasis> {
        let displayed = self.retention.current_presentation_for_surface(surface)?;
        (self
            .current_semantic_surface_for_presentation(displayed.basis())
            .ok()
            == Some(surface))
        .then_some(displayed)
    }

    /// The displayed record for the surface a host report names. A report only
    /// selects the surface binding and host surface; what is on screen there
    /// is what the runtime admitted, never the reported epoch.
    pub(crate) fn current_displayed_presentation(
        &self,
        reported: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Option<crate::mounting::presentation::UiDisplayedSurfaceBasis> {
        let surface = self.current_surface_for_binding(reported.binding())?;
        self.current_presentation_for_surface(surface)
            .filter(|displayed| displayed.basis().host_surface() == reported.host_surface())
    }

    /// Retention admits the latest physical epoch of this surface; mounted
    /// identity admits its live binding. Superseded epochs cannot authorize Motion.
    pub(crate) fn current_semantic_surface_for_presentation(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        use crate::mounting::UiPresentedFrameBasisDenial as Denial;
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
            .admit_current_mounted_incarnation_affinity(input)?;
        self.retention
            .current_node_receipt(presentation, input.mounted_instance)
            .map_err(|_| {
                crate::mounting::UiCurrentHitTargetAffinityDenial::MountedInstanceNoLongerCurrent
            })
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
        let incarnation = crate::mounting::UiMountedIncarnationAffinityInput {
            surface: input.surface,
            binding: input.binding,
            mounted_instance: input.mounted_instance,
        };
        let affinity = self
            .identity
            .admit_current_mounted_incarnation_affinity(incarnation)?;
        let presentation = self
            .current_presentation_for_surface(input.surface)
            .map(|displayed| displayed.basis())
            .ok_or(crate::mounting::UiCurrentHitTargetAffinityDenial::PresentationNotCurrent)?;
        if self
            .retention
            .current_node_receipt(presentation, input.mounted_instance)
            .ok()
            != Some(input.node_receipt)
        {
            return Err(
                crate::mounting::UiCurrentHitTargetAffinityDenial::MountedInstanceNoLongerCurrent,
            );
        }
        Ok(affinity)
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
