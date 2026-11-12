//! Staging order recorded with the actual before/proposed source observation.
use super::source_observation::SourceObservation;
#[derive(Debug)]
pub(crate) struct StagedSourceObservation<Row> {
    pub source: SourceObservation<Row>,
    pub updates_before_callback: usize,
}
