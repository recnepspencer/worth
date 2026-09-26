#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySourceExpectationDenialKind {
    MissingExpectation,
    ForeignApplication,
    ForeignInstallation,
    ForeignSchema,
    ForeignModel,
    ForeignBranch,
    SourceRetired,
    SourceChanged,
    IncompleteFootprint,
    SourceContractMismatch,
    SourceParametersMismatch,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySourceExpectationDenial {
    kind: WorthQuerySourceExpectationDenialKind,
    subject: String,
}

impl WorthQuerySourceExpectationDenial {
    pub const fn kind(&self) -> WorthQuerySourceExpectationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    #[doc(hidden)]
    pub fn new_missing(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQuerySourceExpectationDenialKind::MissingExpectation,
            subject,
        )
    }

    pub(in crate::domain_computation) fn new(
        kind: WorthQuerySourceExpectationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQuerySourceExpectationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "source expectation denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQuerySourceExpectationDenial {}
