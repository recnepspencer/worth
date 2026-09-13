//! Contract-only headless mechanics and record-only presentation evidence.

mod headless_baseline_unavailable_host;
mod headless_capability_profile_host;
mod headless_host;
mod headless_measurement;
mod headless_portal_anchor_host;
mod headless_recorder;
mod headless_transcript;
mod headless_translation;
#[cfg(test)]
mod headless_translation_effect_tests;
mod solicited_effect;

pub use headless_baseline_unavailable_host::WorthUiHeadlessBaselineUnavailableHost;
pub use headless_capability_profile_host::WorthUiHeadlessCapabilityProfileHost;
pub use headless_host::WorthUiHeadlessHost;
pub use headless_portal_anchor_host::WorthUiHeadlessPortalAnchorHost;
pub use headless_recorder::{UiHeadlessPresentationSampleObservation, WorthUiHeadlessRecorder};
pub use headless_transcript::appearance::{
    UiHeadlessAppearanceFragmentTranscript, UiHeadlessAppearanceFrameTranscript,
    UiHeadlessAppearanceMechanic, UiHeadlessAppearanceMechanicChange,
    UiHeadlessAppearancePresentationTranscript, UiHeadlessAppearanceProjectionTranscript,
    UiHeadlessAppearanceWorkTranscript,
};
pub use headless_transcript::{
    UiHeadlessClipMechanic, UiHeadlessLayerMechanic, UiHeadlessMountedFrameTranscript,
    UiHeadlessNodeMechanic, UiHeadlessNodePaintMechanic, UiHeadlessPaintBatchMechanic,
    UiHeadlessRecorderCapacity, UiHeadlessResolvedClip, UiHeadlessResourceContact,
    UiHeadlessSemanticTextMechanic, UiHeadlessTextAccessibilityGeometry, UiHeadlessTextMeasurement,
    UiHeadlessUnperformedEffect,
};
pub use headless_translation::appearance::UiHeadlessAppearanceTranslationDenial;
#[cfg(feature = "certification-support")]
pub use headless_translation::translate_appearance_projection_for_certification;
