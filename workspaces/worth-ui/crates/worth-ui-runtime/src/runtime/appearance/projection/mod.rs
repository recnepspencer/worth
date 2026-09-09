mod appearance_projection;
mod attempt;
mod backdrop;
mod change_receipt;
mod resolved_aspect;
mod resolver;

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

#[allow(
    unused_imports,
    reason = "Gate 1 retains sealed overlay snapshot re-exports for later appearance projection"
)]
pub(crate) use crate::runtime::overlay_composition::{
    UiBackdropInstanceIdentity, UiOverlayStackSnapshot,
};
pub(crate) use appearance_projection::UiAppearanceProjection;
pub(crate) use attempt::{UiAppearanceAttemptContext, UiAppearanceProjectionAttempt};
pub(crate) use backdrop::UiBackdropAppearanceProjection;
pub(crate) use change_receipt::{
    UiAppearanceChangeReceipt, UiAppearanceMountAffinity, UiAppearanceMountAffinityDenial,
};
pub(crate) use resolved_aspect::{
    UiAppearanceProvenance, UiAppearanceSupportPosture, UiResolvedAppearanceAspect,
};
#[allow(
    unused_imports,
    reason = "Gate 1 retains sealed appearance resolution inputs for later mounting consumers"
)]
pub(crate) use resolver::{
    UiAppearanceResolutionDenial, UiAppearanceResolutionSubject, UiAppearanceResolver,
};
