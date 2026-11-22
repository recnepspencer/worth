//! Pre-image retention denial and relation pre-image failure vocabulary.

use worth_relational::facade::transactions::RecordRef;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreImageRetentionDenial {
    Content(worth_relational::facade::content::RelationalContentDenial),
    MissingDemandedField,
    ExceedsByteBound,
    EmptyDemand,
    /// Demanded loci have no unique common mutated record, or a demanded locus
    /// has several admitted observations on that selected record (Q8.26-C7).
    AmbiguousDemandedField,
    /// The operation mutates nothing, so there is no prior truth to invert.
    NoMutatedRecord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRelationPreImageDenial {
    MissingObservation,
    MissingMutation,
    ExceedsByteBound,
    UnsupportedVisibility,
    IncompleteInverse,
    ForeignEndpoint,
    ExtraInverse,
    RepeatedInverse,
    RestoredFieldMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRelationPreImageFailure {
    kind: WorthQueryRelationPreImageDenial,
    affected_records: Vec<RecordRef>,
}

impl WorthQueryRelationPreImageFailure {
    pub fn kind(&self) -> WorthQueryRelationPreImageDenial {
        self.kind
    }

    pub fn affected_records(&self) -> &[RecordRef] {
        &self.affected_records
    }

    pub fn new(kind: WorthQueryRelationPreImageDenial, affected_records: Vec<RecordRef>) -> Self {
        Self {
            kind,
            affected_records,
        }
    }
}
