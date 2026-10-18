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
    /// A prepared source was consumed by a different admitted operation or product selection.
    ForeignOperation,
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
    /// Constructing the source facts exceeded this request's preparation memory.
    PreparationMemoryExceeded,
    /// Exact backing or request stop while retaining a source payload.
    RetainedSourceStorageDenied,
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
    retention_denial: Option<crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial>,
}

impl WorthQuerySourceExpectationDenial {
    pub const fn kind(&self) -> WorthQuerySourceExpectationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub(in crate::domain_computation::primary_graph) fn source_retention_denied(
        subject: &str,
        denial: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
    ) -> Self {
        Self {
            kind: WorthQuerySourceExpectationDenialKind::RetainedSourceStorageDenied,
            subject: subject.to_owned(),
            retention_denial: Some(denial),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn retention_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match &self.retention_denial {
            Some(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::RequestInterruption(stop)) => Some(*stop),
            _ => None,
        }
    }

    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        match &self.retention_denial {
            Some(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Allocation(denial)) => Some(denial), _ => None,
        }
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
            retention_denial: None,
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

impl worth_execution::ChargedBytes for WorthQuerySourceExpectationDenial {
    fn additional_charged_bytes(&self) -> u64 {
        // A native retained-source storage refusal has no resident-error quote.
        // It may be returned, but may not be retained as an underfunded computation error.
        if self.retention_denial.is_some() {
            u64::MAX
        } else {
            self.subject.capacity() as u64
        }
    }
}
