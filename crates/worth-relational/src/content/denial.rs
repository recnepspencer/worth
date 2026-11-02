use crate::execution::RelationalExecutionDenialCause;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalContentDenial {
    MissingContent,
    FormatBindingMissing,
    DurabilityNotReady,
    SchemaTransitionRequiresReconstruction,
    FormatMismatch,
    UnsupportedVersion,
    InvalidFormat,
    NonCanonicalContent,
    IntegrityMismatch,
    ContentIdentityConflict,
    EncodingInconsistency,
    InputTypeMismatch,
    DuplicateContentFormat,
    SizeLimit { observed: usize, maximum: usize },
    ArithmeticOverflow,
    Execution(RelationalExecutionDenialCause),
}

impl From<RelationalExecutionDenialCause> for RelationalContentDenial {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        Self::Execution(cause)
    }
}

impl std::fmt::Display for RelationalContentDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Native content admission refused: {self:?}")
    }
}
impl std::error::Error for RelationalContentDenial {}
