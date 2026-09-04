mod denial;
mod producer;
mod projection_record;

#[cfg(test)]
#[path = "producer_multisurface_tests.rs"]
mod producer_multisurface_tests;
#[cfg(test)]
#[path = "producer_tests.rs"]
mod producer_tests;

pub use producer::UiAppearanceInspectionGenerationSuccessionDenial;
pub(crate) use producer::{
    UiAppearanceInspectionAttemptBatch, UiAppearanceInspectionDenial,
    UiAppearanceInspectionProducer, UiAppearanceInspectionRecord,
    UiPreparedAppearanceInspectionGenerationSuccession,
};
