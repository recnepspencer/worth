//! Real scope and before/proposed rows recorded by the source rule evaluators.
#[derive(Debug)]
pub(crate) struct SourceObservation<Row> {
    pub scope: (u32, u64, u32),
    pub before: Option<Vec<Row>>,
    pub after: Vec<Row>,
}
