mod curve;
mod damage;
mod interruption;
mod opacity;
mod receipt;
mod sampled_geometry;
mod sampling;
#[cfg(test)]
mod tests;
mod track_sampling;
mod velocity;
#[cfg(test)]
mod velocity_continuity_tests;

pub(crate) use damage::{UiPresentationMotionDamage, UiPresentationSampledClipGeometry};
pub use receipt::UiPresentationMotionSamplingCost;
pub(crate) use receipt::{
    UiPresentationMotionInstallationReceipt, UiPresentationMotionPresentedSurface,
    UiPresentationMotionSamplePosture, UiPresentationMotionSampleReceipt,
    UiPresentationMotionSamplingReceipt, UiPresentationMotionTerminalRequest,
    UiPresentationReducedMotionPosture,
};
pub(crate) use sampled_geometry::{
    UiPresentationGeometrySamplingDenial, UiPresentationSampledGeometry,
};
pub(crate) use sampling::{
    UiMountedMotionSampler, UiPreparedMotionSampling, UiPreparedMotionWork,
    UiPresentationMotionSamplingDenial,
};
