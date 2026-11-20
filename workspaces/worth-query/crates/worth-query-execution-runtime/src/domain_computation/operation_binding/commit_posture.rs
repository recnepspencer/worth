/// Installed effect-finalization posture admitted for one bound operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExecutionCommitPosture {
    ReadOnly,
    Atomic,
    Compensated,
}

impl WorthQueryExecutionCommitPosture {
    pub fn requires_atomic_commit(self) -> bool {
        self == Self::Atomic
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::Atomic => "atomic",
            Self::Compensated => "compensated",
        }
    }
}
