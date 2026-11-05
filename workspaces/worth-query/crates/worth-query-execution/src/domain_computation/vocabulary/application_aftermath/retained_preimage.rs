//! Moved whole from `domain_computation::application_aftermath::retained_preimage`: the items below and every impl they own.

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
