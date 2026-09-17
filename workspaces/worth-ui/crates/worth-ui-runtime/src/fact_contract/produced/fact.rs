use super::{
    UiAuthoredChangedFact, UiCommittedPortalAnchorChangedFact, UiCommittedScrollExtentChangedFact,
    UiHostDeviceScaleChangedFact, UiHostViewportChangedFact, UiIntentPostureChangedFact,
    UiMeasurementChangedFact, UiPointerPresenceTargetChangedFact, UiProducedFactFamily,
    UiQueryChangedFact,
};

pub enum UiProducedFact {
    AuthoredSource(UiAuthoredChangedFact),
    HostViewport(UiHostViewportChangedFact),
    HostDeviceScale(UiHostDeviceScaleChangedFact),
    PointerPresenceTarget(UiPointerPresenceTargetChangedFact),
    Measurement(UiMeasurementChangedFact),
    Query(UiQueryChangedFact),
    IntentPosture(UiIntentPostureChangedFact),
    CommittedScrollExtent(UiCommittedScrollExtentChangedFact),
    CommittedPortalAnchor(UiCommittedPortalAnchorChangedFact),
}

impl UiProducedFact {
    pub const fn family(&self) -> UiProducedFactFamily {
        match self {
            Self::AuthoredSource(_) => UiProducedFactFamily::AuthoredSource,
            Self::HostViewport(_) => UiProducedFactFamily::HostViewport,
            Self::HostDeviceScale(_) => UiProducedFactFamily::HostDeviceScale,
            Self::PointerPresenceTarget(_) => UiProducedFactFamily::PointerPresenceTarget,
            Self::Measurement(_) => UiProducedFactFamily::Measurement,
            Self::Query(_) => UiProducedFactFamily::Query,
            Self::IntentPosture(_) => UiProducedFactFamily::IntentPosture,
            Self::CommittedScrollExtent(_) => UiProducedFactFamily::CommittedScrollExtent,
            Self::CommittedPortalAnchor(_) => UiProducedFactFamily::CommittedPortalAnchor,
        }
    }

    pub fn authored_source(&self) -> Option<&UiAuthoredChangedFact> {
        match self {
            Self::AuthoredSource(fact) => Some(fact),
            _ => None,
        }
    }

    pub fn query(&self) -> Option<&UiQueryChangedFact> {
        match self {
            Self::Query(fact) => Some(fact),
            _ => None,
        }
    }

    pub fn intent_posture(&self) -> Option<&UiIntentPostureChangedFact> {
        match self {
            Self::IntentPosture(fact) => Some(fact),
            _ => None,
        }
    }

    pub(crate) fn into_scalar_projection(
        self,
    ) -> Result<worth_ui_query_binding::UiScalarProjectionFactReceipt, Box<Self>> {
        match self {
            Self::Query(fact) => fact
                .into_scalar_projection()
                .map_err(|fact| Box::new(Self::Query(*fact))),
            other => Err(Box::new(other)),
        }
    }

    pub(crate) fn into_application_scalar_projection(
        self,
    ) -> Result<worth_ui_query_binding::UiApplicationScalarProjectionFactReceipt, Box<Self>> {
        match self {
            Self::Query(fact) => fact
                .into_application_scalar_projection()
                .map_err(|fact| Box::new(Self::Query(*fact))),
            other => Err(Box::new(other)),
        }
    }

    pub(crate) fn into_query_owner_consequence(
        self,
    ) -> Result<worth_ui_query_binding::WorthUiCollectionChangeConsequence, Box<Self>> {
        match self {
            Self::Query(fact) => fact
                .into_owner_consequence()
                .map_err(|fact| Box::new(Self::Query(*fact))),
            other => Err(Box::new(other)),
        }
    }

    pub(crate) fn into_query_projection_observation(
        self: Box<Self>,
    ) -> Result<worth_ui_query_binding::UiProjectionObservation, Box<Self>> {
        if !matches!(self.as_ref(), Self::Query(_)) {
            return Err(self);
        }

        match *self {
            Self::Query(query) => query
                .into_projection_observation()
                .map_err(|query| Box::new(Self::Query(*query))),
            _ => unreachable!("the boxed fact was checked as query-owned"),
        }
    }
}
