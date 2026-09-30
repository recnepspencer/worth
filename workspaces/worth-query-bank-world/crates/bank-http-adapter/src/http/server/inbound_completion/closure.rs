//! Orderly HTTP close retains the installed route for surviving owner custody.

use std::fmt;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::route::BankRailCompletionRoute;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankRailCloseAssessment {
    /// Counts overlap: pending recovery, dispatched provenance, and accepted
    /// replay custody are separate owner views, not additive obligations.
    /// The known counts are complete only when `unavailable_routes` is zero.
    Assessed {
        /// Accepted signed or transport work needing owner maintenance.
        known_remaining: usize,
        /// Co-committed dispatch provenance not yet released by the owner.
        outstanding_dispatches: u64,
        /// Live accepted custody slots; compact terminal replay is signaled
        /// separately by `next_expiry_unix_seconds`.
        retained_accepted_occurrences: u64,
        blocked: usize,
        unavailable_routes: u8,
        /// Earliest terminal replay cleanup, including compact replay entries.
        next_expiry_unix_seconds: Option<u64>,
    },
    /// The installed route could not complete its bounded assessment.
    Unavailable,
}

pub struct BankRailCompletionContinuation {
    route: Arc<dyn BankRailCompletionRoute>,
    maximum_deadline: Duration,
}

pub struct BankRailCompletionClose {
    assessment: BankRailCloseAssessment,
    continuation: BankRailCompletionContinuation,
}

pub struct BankRailCompletionCloseFailure {
    error: io::Error,
    close: BankRailCompletionClose,
}

impl BankRailCompletionContinuation {
    pub(in crate::http::server) fn new(
        route: Arc<dyn BankRailCompletionRoute>,
        maximum_deadline: Duration,
    ) -> Self {
        Self {
            route,
            maximum_deadline,
        }
    }

    /// One explicit bounded owner cue after the HTTP listener has closed.
    /// The retained route preserves the original verifier installation.
    pub async fn continue_once(&self) -> BankRailCloseAssessment {
        let Some(deadline) = Instant::now().checked_add(self.maximum_deadline) else {
            return BankRailCloseAssessment::Unavailable;
        };
        let route = Arc::clone(&self.route);
        let batch = tokio::task::spawn_blocking(move || {
            let cancellation = WorthQueryCancellationSource::new();
            let request = WorthQueryRequestScope::new(deadline, cancellation.token());
            route.maintain_custody(&request)
        })
        .await;
        match batch {
            Ok(Ok(report)) => BankRailCloseAssessment::Assessed {
                known_remaining: report.remaining,
                outstanding_dispatches: report.outstanding_dispatches,
                retained_accepted_occurrences: report.retained_accepted_occurrences,
                blocked: report.blocked,
                unavailable_routes: report.unavailable_routes,
                next_expiry_unix_seconds: report.next_expiry_unix_seconds,
            },
            Ok(Err(_)) | Err(_) => BankRailCloseAssessment::Unavailable,
        }
    }

    pub(in crate::http::server) async fn close(self) -> BankRailCompletionClose {
        let assessment = self.continue_once().await;
        BankRailCompletionClose {
            assessment,
            continuation: self,
        }
    }
}

impl BankRailCompletionClose {
    pub const fn assessment(&self) -> BankRailCloseAssessment {
        self.assessment
    }

    pub fn into_continuation(self) -> BankRailCompletionContinuation {
        self.continuation
    }
}

impl BankRailCompletionCloseFailure {
    pub(super) fn new(error: io::Error, close: BankRailCompletionClose) -> Self {
        Self { error, close }
    }

    pub fn error(&self) -> &io::Error {
        &self.error
    }

    pub fn into_close(self) -> BankRailCompletionClose {
        self.close
    }
}

impl fmt::Debug for BankRailCompletionCloseFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BankRailCompletionCloseFailure")
            .field("error", &self.error)
            .finish()
    }
}

impl fmt::Display for BankRailCompletionCloseFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Bank rail callback shutdown: {}", self.error)
    }
}

impl std::error::Error for BankRailCompletionCloseFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

#[cfg(test)]
mod tests;
