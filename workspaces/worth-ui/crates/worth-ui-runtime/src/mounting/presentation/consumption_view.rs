use worth_ui_host_contract::{
    UiHostProtocolAgreement, UiMountedFrameConsumptionInput, UiMountedFrameConsumptionView,
    UiMountedPresentationAttemptIdentity, UiMountedSurfaceBindingRequirement,
    UiPresentationDeadline, WorthUiHostCapabilityReport,
};

use super::{UiMountedPresentationLease, UiMountedPresentationWork};

#[derive(Clone, Copy)]
pub(crate) struct UiMountedHostPresentationAuthority<'authority> {
    host_session_identity: u64,
    protocol: UiHostProtocolAgreement,
    capability_report: &'authority WorthUiHostCapabilityReport,
    presentation: &'authority UiMountedPresentationLease,
}

pub(super) struct UiRuntimeMountedFrameConsumptionInput<'frame> {
    pub attempt: UiMountedPresentationAttemptIdentity,
    pub deadline: UiPresentationDeadline,
    pub requirement: UiMountedSurfaceBindingRequirement,
    pub presentation_work: &'frame UiMountedPresentationWork,
    pub text_raster_work: Option<&'frame worth_ui_host_contract::UiMountedTextRasterWork<'frame>>,
}

impl<'authority> UiMountedHostPresentationAuthority<'authority> {
    pub(crate) fn new(
        host_session_identity: u64,
        protocol: UiHostProtocolAgreement,
        capability_report: &'authority WorthUiHostCapabilityReport,
        presentation: &'authority UiMountedPresentationLease,
    ) -> Self {
        Self {
            host_session_identity,
            protocol,
            capability_report,
            presentation,
        }
    }

    pub(super) fn protocol(self) -> UiHostProtocolAgreement {
        self.protocol
    }

    pub(super) fn capability_report(self) -> &'authority WorthUiHostCapabilityReport {
        self.capability_report
    }

    pub(super) fn presentation(self) -> &'authority UiMountedPresentationLease {
        self.presentation
    }

    pub(super) fn bind<'frame>(
        self,
        input: UiRuntimeMountedFrameConsumptionInput<'frame>,
    ) -> UiMountedFrameConsumptionView<'frame> {
        assert!(
            self.presentation.admits_work(input.presentation_work),
            "mounted presentation work must be issued by the opening runtime lease"
        );
        UiMountedFrameConsumptionView::from_inert_mechanics(UiMountedFrameConsumptionInput {
            authority: self.presentation.mechanics_authority(),
            host_session_identity: self.host_session_identity,
            protocol: self.protocol,
            capability_generation: self.capability_report.observation_generation(),
            capability_profile_digest: self.capability_report.profile_identity_digest(),
            attempt: input.attempt,
            deadline: input.deadline,
            requirement: input.requirement,
            presentation_work: input.presentation_work.view(),
            appearance_work: input.presentation_work.appearance(),
            qualified_text: input.presentation_work,
            text_raster_work: input.text_raster_work,
        })
    }
}
