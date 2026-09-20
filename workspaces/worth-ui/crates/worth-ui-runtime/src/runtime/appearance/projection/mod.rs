mod appearance_projection;
mod attempt;
mod backdrop;
mod change_receipt;
mod resolved_aspect;
mod resolver;
mod scroll_chrome_projection;

#[cfg(test)]
#[path = "backdrop_digest_support.rs"]
mod backdrop_digest_support;
#[cfg(test)]
#[path = "backdrop_digest_tests.rs"]
mod backdrop_digest_tests;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::inputs as projection_test_inputs;
#[cfg(test)]
pub(crate) use tests::inputs_from_session as projection_test_inputs_from_session;

pub(crate) use crate::runtime::overlay_composition::UiOverlayStackSnapshot;
pub(crate) use appearance_projection::UiAppearanceProjection;
pub(crate) use attempt::{UiAppearanceAttemptContext, UiAppearanceProjectionAttempt};
pub(crate) use backdrop::UiBackdropAppearanceProjection;
pub(crate) use change_receipt::{UiAppearanceChangeReceipt, UiAppearanceMountAffinity};
pub(crate) use resolved_aspect::{
    UiAppearanceProvenance, UiAppearanceSupportPosture, UiResolvedAppearanceAspect,
};
pub(crate) use resolver::{
    UiAppearanceResolutionDenial, UiAppearanceResolutionFailure, UiAppearanceResolver,
};
pub(crate) use scroll_chrome_projection::UiScrollChromeAppearanceProjection;
