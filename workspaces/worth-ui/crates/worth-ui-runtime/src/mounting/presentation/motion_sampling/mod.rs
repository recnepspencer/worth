mod curve;
mod damage;
mod interruption;
mod opacity;
mod receipt;
mod sampling;
mod sampling_denial;
#[cfg(test)]
mod tests;
mod track_geometry;
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
pub(crate) use sampling::{
    UiMountedMotionSampler, UiPreparedMotionSampling, UiPreparedMotionWork,
    UiPresentationMotionSamplingDenial,
};
pub(crate) use sampling_denial::UiPresentationGeometrySamplingDenial;
