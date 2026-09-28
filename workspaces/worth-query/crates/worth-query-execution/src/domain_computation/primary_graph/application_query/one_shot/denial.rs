use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

use super::super::WorthQueryApplicationProjectionDenialKind;

/// The specific reason a one-shot query read was refused.
///
/// No rows were returned. Stale and foreign causes mean the plan can no longer
/// be used: admit the query again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationOneShotDenialKind {
    /// The plan belongs to a different runtime or is not a one-shot plan.
    ForeignPlan,
    /// The installed query changed since the plan was admitted.
    StaleInstalledQuery,
    /// The principal no longer resolves as it did.
    StalePrincipal,
    /// The request scope no longer resolves as it did.
    StaleScope,
    /// Authorization refused the read; the full denial is attached.
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    /// The read was cancelled.
    Cancelled,
    /// The read reached its deadline.
    DeadlineExceeded,
    /// The basis selected for the read is not available.
    BasisUnavailable,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// No capacity remains to retain the basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The retained basis has expired.
    ExpiredBasis,
    /// The basis used for the read could not be released cleanly.
    BasisReleaseFailed,
    /// The predicate index the query needs is unavailable.
    PredicateIndexUnavailable,
    /// A predicate lookup exceeded its bound.
    PredicateLookupOverflow,
    /// The read would exceed the result limit.
    ResultLimitExceeded,
    /// The read produced a result count the query shape does not allow.
    CardinalityMismatch,
    /// A graph traversal the query needs is unavailable.
    TraversalUnavailable,
    /// The result projection is unavailable.
    ProjectionUnavailable,
    /// Projecting a result row failed.
    Projection(WorthQueryApplicationProjectionDenialKind),
    /// The read would exceed the result buffer limit.
    ResultBufferLimitExceeded,
    /// The read would exceed the work limit.
    WorkLimitExceeded,
    /// The runtime ran out of observed-source identities.
    SourceIdentityExhausted,
}

/// Refusal to execute an admitted one-shot query read.
///
/// No rows were returned. [`Self::kind`] says why, [`Self::query`] names the
/// query, and [`Self::subject`] names the part of it that was refused; for
/// authorization causes, [`Self::authorization_denial`] carries the full denial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationOneShotDenial {
    kind: WorthQueryApplicationOneShotDenialKind,
    authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
    query: String,
    subject: String,
}

pub(super) fn denial(
    kind: WorthQueryApplicationOneShotDenialKind,
    query: impl Into<String>,
    subject: impl Into<String>,
) -> WorthQueryApplicationOneShotDenial {
    WorthQueryApplicationOneShotDenial {
        kind,
        authorization_denial: None,
        query: query.into(),
        subject: subject.into(),
    }
}

pub(super) fn authorization_denial(
    denial: WorthQueryOperationAuthorizationDenial,
    query: &str,
) -> WorthQueryApplicationOneShotDenial {
    let kind = match denial.kind() {
        WorthQueryOperationAuthorizationDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryApplicationOneShotDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        WorthQueryOperationAuthorizationDenialKind::RetentionCapacityExhausted => {
            WorthQueryApplicationOneShotDenialKind::RetentionCapacityExhausted
        }
        WorthQueryOperationAuthorizationDenialKind::RetentionIdentityExhausted => {
            WorthQueryApplicationOneShotDenialKind::RetentionIdentityExhausted
        }
        WorthQueryOperationAuthorizationDenialKind::SnapshotIdentityExhausted => {
            WorthQueryApplicationOneShotDenialKind::SnapshotIdentityExhausted
        }
        kind => WorthQueryApplicationOneShotDenialKind::Authorization(kind),
    };
    WorthQueryApplicationOneShotDenial {
        kind,
        query: query.to_owned(),
        subject: denial.subject().to_string(),
        authorization_denial: Some(Box::new(denial)),
    }
}

impl WorthQueryApplicationOneShotDenial {
    pub const fn kind(&self) -> WorthQueryApplicationOneShotDenialKind {
        self.kind
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn authorization_denial(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.authorization_denial.as_deref()
    }
}

impl std::fmt::Display for WorthQueryApplicationOneShotDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application-query one-shot denied: {:?} for {} ({})",
            self.kind, self.query, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationOneShotDenial {}

#[cfg(test)]
mod tests {
    use super::{denial, WorthQueryApplicationOneShotDenialKind};

    #[test]
    fn execution_denial_preserves_query_and_internal_subject_separately() {
        let denial = denial(
            WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded,
            "FrameMeasurementCoverageQuery",
            "root/relation[0]/field[0]",
        );

        assert_eq!(denial.query(), "FrameMeasurementCoverageQuery");
        assert_eq!(denial.subject(), "root/relation[0]/field[0]");
    }
}
