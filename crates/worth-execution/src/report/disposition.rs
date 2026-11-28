/// Platform-wide semantic classification of a boundary denial.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DenialClass {
    InvalidInput,
    NotFound,
    Conflict,
    Authorization,
    Invariant,
    RelationLaw,
    DeadlineElapsed,
    Cancelled,
    Exhausted,
    Unavailable,
    ExternalEffect,
    Indeterminate,
    Defect,
}

/// Condition under which the owner advises retrying.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RetryHint {
    Immediate,
    AfterBasisAdvance,
    AfterBackoff,
    WithFreshAuthority,
}

/// The boundary owner's recommended response to a denial.
///
/// This guidance grants no execution, recovery, or retry authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Disposition {
    Retry(RetryHint),
    CorrectInput,
    Reauthorize,
    Escalate,
    Final,
}
