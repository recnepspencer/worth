use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

use super::super::WorthQueryApplicationProjectionDenialKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationOneShotDenialKind {
    ForeignPlan,
    StaleInstalledQuery,
    StalePrincipal,
    StaleScope,
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    Cancelled,
    DeadlineExceeded,
    BasisUnavailable,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    ExpiredBasis,
    BasisReleaseFailed,
    PredicateIndexUnavailable,
    PredicateLookupOverflow,
    ResultLimitExceeded,
    CardinalityMismatch,
    TraversalUnavailable,
    ProjectionUnavailable,
    Projection(WorthQueryApplicationProjectionDenialKind),
    ResultBufferLimitExceeded,
    WorkLimitExceeded,
    SourceIdentityExhausted,
}

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
