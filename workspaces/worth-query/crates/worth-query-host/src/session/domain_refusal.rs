//! The application business rule behind a refused session run.
use worth_query_execution::facade::domain_refusal::{
    WorthQueryDomainRefusal, WorthQueryDomainRefusalCarrier,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted as Uncommitted;

use super::{WorthQueryOwnedSessionDenial, WorthQuerySessionRunDenial};
use crate::background::pending::WorthQueryBackgroundDenial;

impl<DomainDenial: WorthQueryDomainRefusalCarrier> WorthQuerySessionRunDenial<DomainDenial> {
    /// The code and message of the application rule that refused: a handler's
    /// `HandlerResult::domain_refusal` (or a domain denial carrying one) or an
    /// installed invariant declaring `WorthQueryDomainRefusalBinding`. Nothing
    /// was committed.
    ///
    /// `None` for every other refusal. Authority and session checks
    /// (`PermissionDenied`, `SessionExpired`) refuse before any handler or
    /// invariant runs, so they never carry one.
    pub fn domain_refusal(&self) -> Option<WorthQueryDomainRefusal> {
        match self {
            Self::DomainDenied(denial) => denial.domain_refusal().cloned(),
            Self::Uncommitted(Uncommitted::Denied(denial)) => denial.domain_refusal(),
            Self::Request(_)
            | Self::AlreadyCommitted(_)
            | Self::RecoveredCommitted(_)
            | Self::IdempotencyIntentDrift
            | Self::AutoIdempotencyKeysExhausted
            | Self::Cancelled
            | Self::DeadlineExceeded
            | Self::Uncommitted(_) => None,
        }
    }
}

impl<DomainDenial: WorthQueryDomainRefusalCarrier>
    WorthQueryOwnedSessionDenial<WorthQuerySessionRunDenial<DomainDenial>>
{
    /// [`WorthQuerySessionRunDenial::domain_refusal`] of an owned-session run.
    pub fn domain_refusal(&self) -> Option<WorthQueryDomainRefusal> {
        match self {
            Self::Denied(denial) => denial.domain_refusal(),
            Self::HomeClosed => None,
        }
    }
}

impl<DomainDenial: WorthQueryDomainRefusalCarrier>
    WorthQueryBackgroundDenial<WorthQuerySessionRunDenial<DomainDenial>>
{
    /// [`WorthQuerySessionRunDenial::domain_refusal`] of a `spawn_run` call.
    pub fn domain_refusal(&self) -> Option<WorthQueryDomainRefusal> {
        match self {
            Self::Denied(denial) => denial.domain_refusal(),
            Self::HomeClosed | Self::Busy | Self::Panicked | Self::Stopped | Self::AlreadyTaken => {
                None
            }
        }
    }
}
