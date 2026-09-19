#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeComponentPresenceCertificationDenial {
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeMotionTickCertificationDenial {
    Rejected,
}

impl super::WorthUiNativeApplicationShell {
    #[doc(hidden)]
    pub fn apply_component_presence_for_certification(
        &mut self,
        changes: &[crate::facade::entry::UiNativeComponentPresenceChange],
    ) -> Result<bool, UiNativeComponentPresenceCertificationDenial> {
        self.apply_component_presence(changes)
            .map(|progress| {
                progress == super::component_presence::UiNativeComponentPresenceProgress::AwaitingPortalDismissal
            })
            .map_err(|_| UiNativeComponentPresenceCertificationDenial::Rejected)
    }

    #[doc(hidden)]
    pub fn resume_component_presence_for_certification(
        &mut self,
        now_tick: u64,
    ) -> Result<bool, UiNativeComponentPresenceCertificationDenial> {
        self.resume_pending_component_presence(now_tick)
            .map(|progress| {
                progress == super::component_presence::UiNativeComponentPresenceProgress::AwaitingPortalDismissal
            })
            .map_err(|_| UiNativeComponentPresenceCertificationDenial::Rejected)
    }

    #[doc(hidden)]
    pub fn component_is_present_for_certification(
        &self,
        authored_semantic_identity: &str,
    ) -> Option<bool> {
        self.mounted_row_indices
            .get(authored_semantic_identity)
            .map(|index| {
                self.current_native_mounted_instance(&self.mounted_rows[*index])
                    .is_some()
            })
    }

    pub fn inspect_motion_presentation_for_certification(
        &self,
    ) -> crate::certification_support::UiMotionPresentationCertificationSnapshot {
        self.session.inspect_motion_presentation_for_certification()
    }

    pub fn admit_reduced_motion_tick_for_certification(
        &mut self,
        tick: u64,
    ) -> Result<bool, UiNativeMotionTickCertificationDenial> {
        self.admit_native_motion_tick(
            tick,
            worth_ui_host_native::UiNativeReducedMotionPosture::Reduce,
        )
        .map(|disposition| {
            matches!(
                disposition,
                super::motion_sampling::UiNativeMotionTickDisposition::Inactive
            )
        })
        .map_err(|_| UiNativeMotionTickCertificationDenial::Rejected)
    }

    pub fn inspect_portal_runtime_for_certification(
        &self,
    ) -> crate::certification_support::UiPortalRuntimeCertificationSnapshot {
        self.session.inspect_portal_runtime_for_certification()
    }

    pub fn inspect_focus_runtime_for_certification(
        &self,
    ) -> crate::certification_support::UiFocusRuntimeCertificationSnapshot {
        self.session.inspect_focus_runtime_for_certification()
    }

    pub fn inspect_service_proposals_for_certification(
        &self,
    ) -> crate::certification_support::UiServiceProposalCertificationSnapshot {
        self.session.inspect_service_proposals_for_certification()
    }
}
