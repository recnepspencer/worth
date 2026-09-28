/// Why an observed query source could not serve as a mutation's source
/// expectation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySourceExpectationDenialKind {
    /// The operation declares a source expectation and none was bound.
    MissingExpectation,
    /// The source was observed by a different application runtime.
    ForeignApplication,
    /// The source was observed under a different installation.
    ForeignInstallation,
    /// The source was observed under a different schema or package.
    ForeignSchema,
    /// The source was observed under a different model root.
    ForeignModel,
    /// The source was not observed on the admitted product branch.
    ForeignBranch,
    /// The observed source has been retired.
    SourceRetired,
    /// The same source was observed with conflicting revisions.
    SourceChanged,
    /// The observation did not record a complete read footprint.
    IncompleteFootprint,
    /// The source does not come from the query or contract the operation expects.
    SourceContractMismatch,
    /// The source query's parameters do not match the operation input.
    SourceParametersMismatch,
    /// Comparing the source exceeded its canonical work budget.
    WorkBudgetExceeded,
}

/// Refusal to bind an observed query source as an admitted mutation's source
/// expectation.
///
/// Nothing was bound and the mutation did not run. [`Self::kind`] says why and
/// [`Self::subject`] names what was compared.
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
