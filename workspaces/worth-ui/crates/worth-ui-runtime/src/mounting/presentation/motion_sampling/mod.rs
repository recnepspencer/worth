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

pub(crate) use damage::{UiPresentationMotionDamage, UiPresentationSampledClipGeometry};
#[cfg(test)]
pub(crate) use receipt::UiPresentationMotionSamplingCost;
pub(crate) use receipt::{
    UiPresentationMotionInstallationReceipt, UiPresentationMotionSamplePosture,
    UiPresentationMotionSampleReceipt, UiPresentationMotionSamplingReceipt,
    UiPresentationMotionTerminalRequest, UiPresentationReducedMotionPosture,
};
pub(crate) use sampled_geometry::{
    UiPresentationGeometrySamplingDenial, UiPresentationSampledGeometry,
};
pub(crate) use sampling::{
    UiMountedMotionSampler, UiPreparedMotionSampling, UiPresentationMotionSamplingDenial,
};
