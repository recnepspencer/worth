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
#[derive(Clone, Debug)]
pub struct WorthQueryApplicationOneShotDenial {
    kind: WorthQueryApplicationOneShotDenialKind,
    payload: std::sync::Arc<OneShotDenialPayload>,
}

#[derive(Debug)]
struct OneShotDenialPayload {
    authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
    query: String,
    subject: String,
    custody: std::sync::OnceLock<worth_execution::ExecutionMemoryReservation>,
}

impl PartialEq for WorthQueryApplicationOneShotDenial {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.payload.authorization_denial == other.payload.authorization_denial
            && self.payload.query == other.payload.query
            && self.payload.subject == other.payload.subject
    }
}
impl Eq for WorthQueryApplicationOneShotDenial {}

pub(super) fn denial(
    kind: WorthQueryApplicationOneShotDenialKind,
    query: impl Into<String>,
    subject: impl Into<String>,
) -> WorthQueryApplicationOneShotDenial {
    WorthQueryApplicationOneShotDenial {
        kind,
        payload: std::sync::Arc::new(OneShotDenialPayload {
            authorization_denial: None,
            query: query.into(),
            subject: subject.into(),
            custody: std::sync::OnceLock::new(),
        }),
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
        payload: std::sync::Arc::new(OneShotDenialPayload {
            query: query.to_owned(),
            subject: denial.subject().to_string(),
            authorization_denial: Some(Box::new(denial)),
            custody: std::sync::OnceLock::new(),
        }),
    }
}

impl WorthQueryApplicationOneShotDenial {
    /// The shared payload and its one reservation live until the last denial clone drops.
    fn retain_custody(&self, hold: worth_execution::ExecutionMemoryReservation) {
        // A concurrent clone may have retained custody first. Its hold remains;
        // the rejected second hold refunds itself when this result drops.
        let _ = self.payload.custody.set(hold);
    }

    pub const fn kind(&self) -> WorthQueryApplicationOneShotDenialKind {
        self.kind
    }

    pub fn query(&self) -> &str {
        &self.payload.query
    }

    pub fn subject(&self) -> &str {
        &self.payload.subject
    }

    pub fn authorization_denial(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.payload.authorization_denial.as_deref()
    }
}

/// Owner stages and the complete reconstruction return through this boundary.
/// A worker's map allowance ends before its returned payload acquires custody.
pub(in crate::domain_computation::primary_graph) fn retain_reconstruction_result<T>(
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    result: Result<T, super::super::WorthQueryManagedDerivedViewDenial>,
    on_admission_denial: impl FnOnce(
        worth_execution::LeaseDenial,
    ) -> super::super::WorthQueryManagedDerivedViewDenial,
) -> Result<T, super::super::WorthQueryManagedDerivedViewDenial> {
    use super::super::WorthQueryManagedDerivedViewDenial as Denial;
    use worth_execution::ChargedBytes;
    if let Err(Denial::ReadDenied { denial, .. }) = &result {
        if denial.payload.custody.get().is_none() {
            let hold = worth_execution::ExecutionMemoryReservation::reserve_in_scope(
                lease,
                denial.additional_charged_bytes(),
            )
            .map_err(on_admission_denial)?;
            denial.retain_custody(hold);
        }
    }
    result
}

impl std::fmt::Display for WorthQueryApplicationOneShotDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application-query one-shot denied: {:?} for {} ({})",
            self.kind, self.payload.query, self.payload.subject
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

impl worth_execution::ChargedBytes for WorthQueryApplicationOneShotDenial {
    fn additional_charged_bytes(&self) -> u64 {
        let OneShotDenialPayload {
            authorization_denial,
            query,
            subject,
            custody: _,
        } = &*self.payload;
        // The Arc allocation contains the payload and its two reference counters.
        (std::mem::size_of::<OneShotDenialPayload>() as u64)
            .saturating_add((2 * std::mem::size_of::<usize>()) as u64)
            .saturating_add(query.capacity() as u64)
            .saturating_add(subject.capacity() as u64)
            .saturating_add(authorization_denial.as_ref().map_or(0, |denial| {
                // Box<T>'s implementation counts its allocation and T's payload.
                denial.additional_charged_bytes()
            }))
    }
}

#[cfg(test)]
mod capacity_tests;
