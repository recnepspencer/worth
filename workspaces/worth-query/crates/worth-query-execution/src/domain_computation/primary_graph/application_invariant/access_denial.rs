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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantAccessDenial {
    kind: WorthQueryInvariantAccessDenialKind,
    subject: String,
}

impl WorthQueryInvariantAccessDenial {
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
