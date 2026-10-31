//! Preserve the pure provider or original controlled failure during consistency.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexWorkDenial as Cause;
impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn managed_index(
        cause: Cause,
        subject: impl Into<String>,
    ) -> Self {
        Self::from_cause(
            WorthQueryApplicationAttemptDenialKind::ManagedIndexSourceDenied,
            subject.into(),
            AttemptDenialCause::ManagedIndex(cause),
        )
    }
    pub fn managed_index_denial(&self) -> Option<&Cause> {
        match self.cause() {
            Some(AttemptDenialCause::ManagedIndex(cause)) => Some(cause),
            _ => None,
        }
    }
}
