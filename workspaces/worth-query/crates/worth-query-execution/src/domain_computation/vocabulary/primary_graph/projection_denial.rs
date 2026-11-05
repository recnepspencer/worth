//! Moved whole from `domain_computation::primary_graph::vocabulary::application_query::projection_denial`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::value_decode::WorthQueryValueDecodeDenial;

/// Why a query result row could not be projected into an application value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationProjectionDenialKind {
    /// Native admitted value decoding refused the original execution request.
    ValueDecode(WorthQueryValueDecodeDenial),
    /// The selected field is not part of this row's projection.
    FieldNotProjected,
    /// The field selector does not match the installed field contract or disclosure.
    FieldContractMismatch,
    /// The field's value could not be decoded as the selector's value type.
    FieldTypeMismatch,
    /// A required field was withheld by governed disclosure.
    FieldOmitted,
    /// The selected relation is not part of this row's projection.
    RelationNotProjected,
    /// The relation selector does not match the installed relation contract or
    /// disclosure.
    RelationContractMismatch,
    /// The relation's result count does not match the selector's cardinality.
    RelationCardinalityMismatch,
    /// A required relation was withheld by governed disclosure.
    RelationOmitted,
    /// Application projection code rejected the row with `reject`.
    DomainProjectionRejected,
}

/// Refusal to project a query result row into an application value.
///
/// Returned by row reads and by application code through `reject`.
/// [`Self::kind`] says why and the subject names the refused result path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationProjectionDenial {
    kind: WorthQueryApplicationProjectionDenialKind,
    subject: String,
}

impl WorthQueryApplicationProjectionDenial {
    pub fn reject(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryApplicationProjectionDenialKind::DomainProjectionRejected,
            subject,
        )
    }

    pub const fn kind(&self) -> WorthQueryApplicationProjectionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub(in crate::domain_computation) fn new(
        kind: WorthQueryApplicationProjectionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQueryApplicationProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application-query projection denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationProjectionDenial {}
