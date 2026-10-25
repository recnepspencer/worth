//! Owned application-attempt refusals and their original causes.
mod kind;
pub use kind::WorthQueryApplicationAttemptDenialKind;

/// A typed refusal of an application attempt before commit, with the subject it
/// names (usually the operation, entity, field, or relation involved).
///
/// Nothing was committed. Match on [`kind`](Self::kind); the subject is for
/// diagnostics.
#[derive(Debug)]
pub struct WorthQueryApplicationAttemptDenial {
    subject: String,
    meaning: AttemptDenialMeaning,
}

/// The native storage case owns its category; authored refusals retain theirs.
/// Keep storage failures inline without inflating every authored cause.
#[derive(Debug)]
enum AttemptDenialMeaning {
    Authored {
        kind: WorthQueryApplicationAttemptDenialKind,
        cause: AttemptDenialCause,
    },
    NativeStorage(worth_relational::facade::storage::RelationalStorageUnavailable),
}

/// Rich causes own their original box; native memory refusals stay inline.
/// Inspection preserves the original owner cause.
#[derive(Debug)]
enum AttemptDenialCause {
    IndexedLookup(Box<worth_relational::facade::indexes::BoundedEntityFieldLookupDenial>),
    OutputObservation(Box<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop>),
    None,
    RelationalExecution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    NativeQueryRead(Box<worth_relational::facade::runtime::QueryReadExecutionStop>),
    RetainedSource(Box<super::retained_decision_facts::StoreDenial>),
    Expression(Box<worth_foundational::expression_api::ExpressionDenial>),
    Allocation(Box<worth_execution::ExecutionAllocationDenial>),
    RequestAuthority(
        Box<crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>,
    ),
}

impl WorthQueryApplicationAttemptDenial {
    fn from_cause(
        kind: WorthQueryApplicationAttemptDenialKind,
        subject: String,
        cause: AttemptDenialCause,
    ) -> Self {
        Self {
            subject,
            meaning: AttemptDenialMeaning::Authored { kind, cause },
        }
    }

    const fn cause(&self) -> Option<&AttemptDenialCause> {
        match &self.meaning {
            AttemptDenialMeaning::Authored { cause, .. } => Some(cause),
            AttemptDenialMeaning::NativeStorage(_) => None,
        }
    }
    pub(in crate::domain_computation) fn relational_execution_cause(
        &self,
    ) -> Option<worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        match self.cause() {
            Some(AttemptDenialCause::RelationalExecution(cause)) => Some(*cause),
            _ => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryApplicationAttemptDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self::from_cause(kind, subject.into(), AttemptDenialCause::None)
    }

    /// The attempt's request lost its authority: it was cancelled, passed
    /// its deadline, or its authentication expired.
    pub(in crate::domain_computation::primary_graph) fn request_authority_lost(
        denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self::from_cause(
            WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
            denial.subject().to_owned(),
            AttemptDenialCause::RequestAuthority(Box::new(denial)),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn authorization_denied(
        kind: WorthQueryApplicationAttemptDenialKind,
        denial: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self::from_cause(
            kind,
            denial.subject().to_owned(),
            AttemptDenialCause::RequestAuthority(Box::new(denial)),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn allocation_denied(
        subject: impl Into<String>,
        denial: worth_execution::ExecutionAllocationDenial,
    ) -> Self {
        Self::from_cause(
            WorthQueryApplicationAttemptDenialKind::AllocationDenied,
            subject.into(),
            AttemptDenialCause::Allocation(Box::new(denial)),
        )
    }

    /// Exact physical owner refusal behind AllocationDenied; not graph authority.
    pub(in crate::domain_computation::primary_graph) fn retained_source_denied(
        kind: WorthQueryApplicationAttemptDenialKind,
        subject: &str,
        denial: super::retained_decision_facts::StoreDenial,
    ) -> Self {
        Self::from_cause(
            kind,
            subject.to_owned(),
            AttemptDenialCause::RetainedSource(Box::new(denial)),
        )
    }
    pub fn source_retention_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match self.cause() {
            Some(AttemptDenialCause::RetainedSource(denial)) => match denial.as_ref() {
                super::retained_decision_facts::StoreDenial::RequestInterruption(interruption) => {
                    Some(*interruption)
                }
                _ => None,
            },
            _ => None,
        }
    }
    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        match self.cause() {
            Some(AttemptDenialCause::Allocation(denial)) => Some(denial.as_ref()),
            Some(AttemptDenialCause::RelationalExecution(_)) => None,
            Some(AttemptDenialCause::RetainedSource(denial)) => match denial.as_ref() {
                super::retained_decision_facts::StoreDenial::Allocation(denial) => Some(denial),
                _ => None,
            },
            _ => None,
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationAttemptDenialKind {
        match &self.meaning {
            AttemptDenialMeaning::Authored { kind, .. } => *kind,
            AttemptDenialMeaning::NativeStorage(_) => {
                WorthQueryApplicationAttemptDenialKind::StorageUnavailable
            }
        }
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The original Relational read refusal, including non-request causes.
    pub fn native_query_read_stop(
        &self,
    ) -> Option<&worth_relational::facade::runtime::QueryReadExecutionStop> {
        match self.cause() {
            Some(AttemptDenialCause::NativeQueryRead(stop)) => Some(stop),
            _ => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn into_native_query_read_stop(
        self,
    ) -> Option<Box<worth_relational::facade::runtime::QueryReadExecutionStop>> {
        match self.meaning {
            AttemptDenialMeaning::Authored {
                cause: AttemptDenialCause::NativeQueryRead(stop),
                ..
            } => Some(stop),
            _ => None,
        }
    }

    /// The language denial behind a workflow condition or body requirement refusal.
    pub const fn expression(
        &self,
    ) -> Option<&worth_foundational::expression_api::ExpressionDenial> {
        match self.cause() {
            Some(AttemptDenialCause::Expression(denial)) => Some(denial),
            _ => None,
        }
    }

    /// The original authorization denial behind request-authority loss or
    /// approval readmission, including any retained native actor-source cause.
    pub const fn request_authority(
        &self,
    ) -> Option<&crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        match self.cause() {
            Some(AttemptDenialCause::RequestAuthority(denial)) => Some(denial),
            _ => None,
        }
    }
}

impl std::fmt::Display for WorthQueryApplicationAttemptDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application attempt {:?}: {}",
            self.kind(),
            self.subject
        )
    }
}

mod output_observation;

impl std::error::Error for WorthQueryApplicationAttemptDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.cause() {
            Some(AttemptDenialCause::IndexedLookup(denial)) => Some(denial.as_ref()),
            Some(AttemptDenialCause::OutputObservation(stop)) => Some(stop.as_ref()),
            Some(AttemptDenialCause::NativeQueryRead(stop)) => Some(stop.as_ref()),
            Some(AttemptDenialCause::RetainedSource(denial)) => Some(denial.as_ref()),
            Some(AttemptDenialCause::Allocation(denial)) => Some(denial.as_ref()),
            Some(AttemptDenialCause::RequestAuthority(denial)) => Some(denial.as_ref()),
            Some(AttemptDenialCause::RelationalExecution(_)) => None,
            _ => None,
        }
    }
}

#[cfg(test)]
mod allocation_error_source_tests;

#[cfg(test)]
mod native_query_cause;

mod expression;
mod request_read;

mod capacity;
mod indexed_lookup;

#[cfg(test)]
mod native_storage_tests;
