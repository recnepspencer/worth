#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantAccessDenialKind {
    ForeignBinding,
    ForeignView,
    OutsideDeclaredAccess,
    OutsidePreparedScope,
    WrongEntityKind,
    EntityUnavailable,
    MissingRequiredField,
    InvalidValue,
    RelationNotInstalled,
    WorkBudgetExceeded,
    Interrupted(worth_relational::facade::runtime::CustomInvariantInterruption),
    MemoryBudgetExceeded,
    MemoryAdmission(worth_execution::MemoryLimitDenial),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantAccessDenial {
    kind: WorthQueryInvariantAccessDenialKind,
    subject: String,
}

impl WorthQueryInvariantAccessDenial {
    pub(super) fn from_native_read(
        cause: worth_relational::facade::runtime::StructuralWalkDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        use worth_relational::facade::runtime::{StructuralReadError, StructuralWalkDenialKind};
        let kind = match cause {
            StructuralWalkDenialKind::Read(StructuralReadError::WorkBudgetExceeded) => {
                WorthQueryInvariantAccessDenialKind::WorkBudgetExceeded
            }
            StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess) => {
                WorthQueryInvariantAccessDenialKind::OutsideDeclaredAccess
            }
            StructuralWalkDenialKind::Read(StructuralReadError::RecordUnavailable) => {
                WorthQueryInvariantAccessDenialKind::EntityUnavailable
            }
            StructuralWalkDenialKind::Interrupted(stop) => {
                WorthQueryInvariantAccessDenialKind::Interrupted(stop)
            }
            StructuralWalkDenialKind::MemoryBudgetExceeded => {
                WorthQueryInvariantAccessDenialKind::MemoryBudgetExceeded
            }
            StructuralWalkDenialKind::MemoryAdmission(cause) => {
                WorthQueryInvariantAccessDenialKind::MemoryAdmission(cause)
            }
        };
        Self::new(kind, subject)
    }

    pub(super) fn new(
        kind: WorthQueryInvariantAccessDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryInvariantAccessDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryInvariantAccessDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application invariant access denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryInvariantAccessDenial {}
