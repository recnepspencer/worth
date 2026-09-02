use super::super::{
    WorthUiActiveApplicationSession, WorthUiApplicationCutoverReceipt,
    WorthUiApplicationSemanticNoOpReceipt, WorthUiPreparedApplicationActivation,
};

pub struct WorthUiPreparedMountedApplicationReplacement<'session> {
    pub(super) session: &'session mut WorthUiActiveApplicationSession,
    pub(super) application: Box<WorthUiPreparedApplicationActivation>,
    pub(super) mounted_successor: crate::mounting::UiMountedGraphReplacementSuccessor,
    pub(super) frame: crate::mounting::UiPreparedMountedFrame,
    pub(super) lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
}

pub struct WorthUiMountedApplicationReplacementInFlight<'session> {
    pub(super) session: &'session mut WorthUiActiveApplicationSession,
    pub(super) application: Box<WorthUiPreparedApplicationActivation>,
    pub(super) mounted: crate::mounting::UiMountedGraphReplacementInFlight,
    pub(super) lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
}

pub struct WorthUiMountedApplicationReplacementIndeterminate<'session> {
    pub(super) session: &'session mut WorthUiActiveApplicationSession,
    pub(super) application: Box<WorthUiPreparedApplicationActivation>,
    pub(super) frame: crate::mounting::UiMountedIndeterminateFrame,
    pub(super) lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
}

pub struct WorthUiMountedReplacementAdmissionDenial<'session> {
    pub(super) denial: crate::mounting::UiMountedPresentationAdmissionDenial,
    pub(super) replacement: Box<WorthUiPreparedMountedApplicationReplacement<'session>>,
}

pub struct WorthUiMountedReplacementRetentionDenial<'session> {
    pub(super) denial: crate::mounting::UiMountedFrameRetentionDenial,
    pub(super) replacement: Box<WorthUiPreparedMountedApplicationReplacement<'session>>,
}

pub struct WorthUiMountedReplacementCompletionDenial<'session> {
    pub(super) denial: crate::mounting::UiMountedPresentationCompletionDenial,
    pub(super) in_flight: WorthUiMountedApplicationReplacementInFlight<'session>,
}

pub(crate) struct WorthUiDetachedPreparedMountedApplicationReplacement {
    pub(super) session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(super) application: Box<WorthUiPreparedApplicationActivation>,
    pub(super) mounted_successor: crate::mounting::UiMountedGraphReplacementSuccessor,
    pub(super) frame: crate::mounting::UiPreparedMountedFrame,
    pub(super) lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
}

pub(crate) struct WorthUiDetachedMountedApplicationReplacementInFlight {
    pub(super) session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(super) application: Box<WorthUiPreparedApplicationActivation>,
    pub(super) mounted: crate::mounting::UiMountedGraphReplacementInFlight,
    pub(super) lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
}

pub struct WorthUiMountedReplacementHostRejection<'session> {
    pub(super) rejections: Box<[crate::mounting::UiMountedSurfacePresentationRejection]>,
    pub(super) replacement: Box<WorthUiPreparedMountedApplicationReplacement<'session>>,
}

pub enum WorthUiMountedReplacementPreparationOutcome<'session> {
    SemanticNoOp(Box<WorthUiApplicationSemanticNoOpReceipt>),
    Prepared(Box<WorthUiPreparedMountedApplicationReplacement<'session>>),
}

pub enum WorthUiMountedApplicationReplacementOutcome<'session> {
    Published {
        application: WorthUiApplicationCutoverReceipt,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
    },
    RejectedBeforeEffects(WorthUiMountedReplacementHostRejection<'session>),
    InFlight(Box<WorthUiMountedApplicationReplacementInFlight<'session>>),
    PresentationIndeterminate(Box<WorthUiMountedApplicationReplacementIndeterminate<'session>>),
    RetentionDenied(WorthUiMountedReplacementRetentionDenial<'session>),
    AdmissionDenied(WorthUiMountedReplacementAdmissionDenial<'session>),
    CompletionDenied(Box<WorthUiMountedReplacementCompletionDenial<'session>>),
}

impl<'session> WorthUiMountedApplicationReplacementIndeterminate<'session> {
    pub fn frame(&self) -> &crate::mounting::UiMountedIndeterminateFrame {
        &self.frame
    }

    pub(crate) fn into_session_for_shutdown(
        self: Box<Self>,
    ) -> &'session mut WorthUiActiveApplicationSession {
        let Self {
            session,
            application,
            frame,
            lifecycle,
        } = *self;
        drop((application, frame));
        drop(lifecycle);
        session
    }
}

impl<'session> WorthUiMountedReplacementAdmissionDenial<'session> {
    pub fn denial(&self) -> crate::mounting::UiMountedPresentationAdmissionDenial {
        self.denial
    }

    pub fn into_replacement(self) -> Box<WorthUiPreparedMountedApplicationReplacement<'session>> {
        self.replacement
    }
}

impl<'session> WorthUiMountedReplacementRetentionDenial<'session> {
    pub fn denial(&self) -> crate::mounting::UiMountedFrameRetentionDenial {
        self.denial
    }

    pub fn into_replacement(self) -> Box<WorthUiPreparedMountedApplicationReplacement<'session>> {
        self.replacement
    }
}

impl<'session> WorthUiMountedReplacementCompletionDenial<'session> {
    pub fn denial(&self) -> crate::mounting::UiMountedPresentationCompletionDenial {
        self.denial
    }

    pub fn into_in_flight(
        self: Box<Self>,
    ) -> WorthUiMountedApplicationReplacementInFlight<'session> {
        self.in_flight
    }
}

impl<'session> WorthUiMountedReplacementHostRejection<'session> {
    pub fn rejections(&self) -> &[crate::mounting::UiMountedSurfacePresentationRejection] {
        &self.rejections
    }

    pub fn into_parts(
        self,
    ) -> (
        Box<[crate::mounting::UiMountedSurfacePresentationRejection]>,
        Box<WorthUiPreparedMountedApplicationReplacement<'session>>,
    ) {
        (self.rejections, self.replacement)
    }

    pub fn into_replacement(self) -> Box<WorthUiPreparedMountedApplicationReplacement<'session>> {
        self.replacement
    }
}
