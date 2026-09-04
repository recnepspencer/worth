//! Contract-only headless mechanics and record-only presentation evidence.
#![cfg_attr(
    not(feature = "certification-support"),
    doc = r#"
The unpublished appearance certification facade is intentionally absent from
the default headless surface.

```compile_fail
use worth_ui_host_headless::translate_unpublished_appearance_for_certification;
```
"#
)]

mod headless_baseline_unavailable_host;
mod headless_capability_profile_host;
mod headless_host;
mod headless_measurement;
mod headless_portal_anchor_host;
mod headless_recorder;
#[cfg(test)]
mod headless_static_paint_tests;
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
#[cfg(feature = "certification-support")]
pub use headless_transcript::appearance::{
    UiHeadlessAppearanceFrameTranscript, UiHeadlessAppearanceMechanic,
    UiHeadlessAppearanceMechanicChange, UiHeadlessAppearanceWorkTranscript,
    UiHeadlessUnpublishedAppearanceFragmentTranscript,
    UiHeadlessUnpublishedAppearanceFrameTranscript,
};
pub use headless_transcript::{
    UiHeadlessClipMechanic, UiHeadlessFilledRectMechanic, UiHeadlessLayerMechanic,
    UiHeadlessMountedFrameTranscript, UiHeadlessNodeMechanic, UiHeadlessNodePaintMechanic,
    UiHeadlessPaintBatchMechanic, UiHeadlessRecorderCapacity, UiHeadlessResolvedClip,
    UiHeadlessResourceContact, UiHeadlessSemanticTextMechanic, UiHeadlessTextAccessibilityGeometry,
    UiHeadlessTextMeasurement, UiHeadlessUnperformedEffect,
};
#[cfg(feature = "certification-support")]
pub use headless_translation::appearance::UiHeadlessAppearanceTranslationDenial;
#[cfg(feature = "certification-support")]
pub use headless_translation::translate_unpublished_appearance_for_certification;
