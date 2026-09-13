use super::PlatformPulsePortalJourneyFailure;

impl std::fmt::Display for PlatformPulsePortalJourneyFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native(failure) => write!(formatter, "native platform: {failure:?}"),
            Self::ControlPoint(failure) => write!(formatter, "control point: {failure}"),
            Self::Pixels(failure) => write!(formatter, "portal pixels: {failure}"),
            Self::TextClipping(failure) => write!(formatter, "text clipping: {failure:?}"),
            Self::ModalStack(message) => write!(formatter, "modal stack: {message}"),
            Self::CaptureExport(failure) => write!(formatter, "capture export: {failure}"),
            Self::ServicePixels(failure) => write!(formatter, "service pixels: {failure:?}"),
            Self::SourceAction(failure) => write!(formatter, "source action: {failure:?}"),
            Self::SourceDefinition(failure) => write!(formatter, "source definition: {failure:?}"),
            Self::Observation(failure) => write!(formatter, "observation: {failure:?}"),
            Self::FocusEvidence(message) => write!(formatter, "focus evidence: {message}"),
            Self::UnexpectedObservation(observation) => {
                write!(formatter, "unexpected observation: {observation}")
            }
            Self::InputDelivery(message) => write!(formatter, "input delivery: {message}"),
            Self::RuntimeServiceEvidence(message) => {
                write!(formatter, "runtime service evidence: {message}")
            }
        }
    }
}
