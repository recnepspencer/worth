#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalPublicationReadDependencyDenial {
    EmptySourceUnsupported,
    ForeignOwner,
    AlreadyRequired,
    CandidateUnavailable,
    ConflictingSource,
    NotCurrent,
}
