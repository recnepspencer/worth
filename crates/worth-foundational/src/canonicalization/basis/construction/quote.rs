//! Fixed context from the actual refused canonical preparation callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalBasisPreparationPhase {
    SequenceVisit,
    EntryMeasurement,
    EntryFieldMeasurement,
    StableSort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalBasisPreparationQuote {
    pub phase: CanonicalBasisPreparationPhase,
    pub work: usize,
    pub bytes: usize,
    pub entry_count: usize,
    pub comparison_width: Option<usize>,
}
