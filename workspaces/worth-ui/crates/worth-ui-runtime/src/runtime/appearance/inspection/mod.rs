mod denial;
mod producer;
mod projection_record;

#[cfg(test)]
#[path = "producer_tests.rs"]
mod producer_tests;

pub(crate) use producer::{
    UiAppearanceInspectionAttemptBatch, UiAppearanceInspectionDenial,
    UiAppearanceInspectionProducer, UiAppearanceInspectionRecord,
};
