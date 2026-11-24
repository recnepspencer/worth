//! Typed refusal shared by native schema transition and durable codec readmission.
use worth_relational_contracts::internal_port as contracts;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalSchemaTransitionAdmissionDenialKind {
    TransactionAdmission,
    SourceBasisMismatch,
    TargetBasisMismatch,
    TargetContractMismatch,
    InvalidTargetRegistry,
    RecoveredAuthorityMismatch,
    UnsupportedCodecConsequence,
    CodecClosureRejected,
    NativeExecution,
    Staging,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalSchemaTransitionAdmissionDenial {
    kind: RelationalSchemaTransitionAdmissionDenialKind,
    detail: String,
    request_cause: Option<contracts::execution::denial_cause::RelationalExecutionDenialCause>,
    staging_cause:
        Option<contracts::mvcc::transaction::staging::RelationalTransactionStagingDenial>,
}

impl RelationalSchemaTransitionAdmissionDenial {
    pub(crate) fn new(
        kind: RelationalSchemaTransitionAdmissionDenialKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            request_cause: None,
            staging_cause: None,
        }
    }

    pub const fn kind(&self) -> RelationalSchemaTransitionAdmissionDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn request_cause(
        &self,
    ) -> Option<&contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        self.request_cause.as_ref()
    }

    pub fn staging_cause(
        &self,
    ) -> Option<&contracts::mvcc::transaction::staging::RelationalTransactionStagingDenial> {
        self.staging_cause.as_ref()
    }

    pub(crate) fn staging(
        cause: contracts::mvcc::transaction::staging::RelationalTransactionStagingDenial,
    ) -> Self {
        if let Some(worth_execution::ExecutionAllocationDenialClass::Request(stop)) =
            cause.allocation_denial().map(|denial| denial.classify())
        {
            return Self::request(stop.into());
        }
        Self {
            kind: RelationalSchemaTransitionAdmissionDenialKind::Staging,
            detail: format!("{cause:?}"),
            request_cause: None,
            staging_cause: Some(cause),
        }
    }

    pub(crate) fn request(
        cause: contracts::execution::denial_cause::RelationalExecutionDenialCause,
    ) -> Self {
        Self {
            kind: RelationalSchemaTransitionAdmissionDenialKind::NativeExecution,
            detail: format!("{cause:?}"),
            request_cause: Some(cause),
            staging_cause: None,
        }
    }
}

impl std::fmt::Display for RelationalSchemaTransitionAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Relational schema transition admission denied: {:?} ({})",
            self.kind, self.detail
        )
    }
}

impl std::error::Error for RelationalSchemaTransitionAdmissionDenial {}

#[doc(hidden)]
pub trait RelationalSchemaTransitionAdmissionDenialOwnerPort: Sized {
    fn new(kind: RelationalSchemaTransitionAdmissionDenialKind, detail: impl Into<String>) -> Self;

    fn staging(
        cause: contracts::mvcc::transaction::staging::RelationalTransactionStagingDenial,
    ) -> Self;

    fn request(cause: contracts::execution::denial_cause::RelationalExecutionDenialCause) -> Self;
}

impl RelationalSchemaTransitionAdmissionDenialOwnerPort
    for RelationalSchemaTransitionAdmissionDenial
{
    fn new(kind: RelationalSchemaTransitionAdmissionDenialKind, detail: impl Into<String>) -> Self {
        Self::new(kind, detail)
    }

    fn staging(
        cause: contracts::mvcc::transaction::staging::RelationalTransactionStagingDenial,
    ) -> Self {
        Self::staging(cause)
    }

    fn request(cause: contracts::execution::denial_cause::RelationalExecutionDenialCause) -> Self {
        Self::request(cause)
    }
}
