use super::super::visual_contract_manifest::PlatformPulseVisualContractFailure;

#[derive(Debug)]
pub(crate) enum PlatformPulsePortalPixelFailure {
    Manifest(PlatformPulseVisualContractFailure),
    CaptureMismatch,
    OverlayMissing {
        changed: usize,
        matching: usize,
        sampled: usize,
    },
    AuthoredSurfaceMissing {
        identity: &'static str,
        matching: usize,
        sampled: usize,
    },
    SemanticInkMissing {
        identity: &'static str,
        matching: usize,
    },
    RestorationMissing {
        differing: usize,
        sampled: usize,
    },
    PreferredFocusParticipantRetained {
        changed: usize,
        background_matching: usize,
        sampled: usize,
    },
    FallbackActionChanged {
        differing: usize,
        sampled: usize,
    },
    SurvivingPortalChanged {
        differing: usize,
        sampled: usize,
    },
}

impl std::fmt::Display for PlatformPulsePortalPixelFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest(failure) => write!(formatter, "visual manifest: {failure:?}"),
            Self::CaptureMismatch => formatter.write_str("capture mismatch"),
            Self::OverlayMissing {
                changed,
                matching,
                sampled,
            } => write!(
                formatter,
                "portal overlay missing: {changed} changed, {matching} matching, {sampled} sampled",
            ),
            Self::AuthoredSurfaceMissing {
                identity,
                matching,
                sampled,
            } => write!(
                formatter,
                "authored surface {identity} missing: {matching} matching of {sampled} sampled",
            ),
            Self::SemanticInkMissing { identity, matching } => {
                write!(formatter, "semantic ink {identity} missing: {matching} matching")
            }
            Self::RestorationMissing { differing, sampled } => write!(
                formatter,
                "portal restoration missing: {differing} differing of {sampled} sampled",
            ),
            Self::PreferredFocusParticipantRetained {
                changed,
                background_matching,
                sampled,
            } => write!(
                formatter,
                "preferred focus participant retained: {changed} changed, {background_matching} background matching, {sampled} sampled",
            ),
            Self::FallbackActionChanged { differing, sampled } => write!(
                formatter,
                "fallback action changed: {differing} differing of {sampled} sampled",
            ),
            Self::SurvivingPortalChanged { differing, sampled } => write!(
                formatter,
                "surviving Portal changed: {differing} differing of {sampled} sampled",
            ),
        }
    }
}
