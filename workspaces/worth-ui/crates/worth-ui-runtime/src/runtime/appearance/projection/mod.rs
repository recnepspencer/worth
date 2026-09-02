mod backdrop;
mod change_receipt;
mod projection;
mod resolved_aspect;
mod resolver;

#[cfg(test)]
#[path = "backdrop_digest_tests.rs"]
mod backdrop_digest_tests;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub(crate) use crate::runtime::overlay_composition::{
    UiBackdropInstanceIdentity, UiOverlayStackSnapshot,
};
pub(crate) use backdrop::UiBackdropAppearanceProjection;
pub(crate) use change_receipt::{UiAppearanceChangeOutcome, UiAppearanceChangeReceipt};
pub(crate) use projection::UiAppearanceProjection;
pub(crate) use resolved_aspect::{
    UiAppearanceProvenance, UiAppearanceSupportPosture, UiResolvedAppearanceAspect,
};
pub(crate) use resolver::{UiAppearanceResolutionDenial, UiAppearanceResolver};
