//! Moved whole from `domain_computation::primary_graph::application_query::observed_source::denial`: the items below and every impl they own.

/// Why an observed query source could not serve as a mutation's source
/// expectation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySourceExpectationDenialKind {
    /// The exact admission authority refusal before any source work.
    AdmissionAuthorization,
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
    /// Managed expansion requires the genuine current Execution request.
    /// Legacy conversion without that request preserves evidence and refuses.
    ManagedSourceExpansionUnsupported,
}
