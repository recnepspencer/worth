mod producer;

#[cfg(test)]
#[path = "producer_tests.rs"]
mod producer_tests;

pub(crate) use producer::{UiAppearanceInspectionDenial, UiAppearanceInspectionProducer};
