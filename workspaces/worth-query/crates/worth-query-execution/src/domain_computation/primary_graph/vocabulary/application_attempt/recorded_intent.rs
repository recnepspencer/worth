//! Moved whole from `domain_computation::primary_graph::application_attempt::idempotency::recorded_intent`: the items below and every impl they own.

/// How a request's intent compares with the intent recorded under its key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryRecordedIntentMatch {
    /// The record commits this intent.
    Same,
    /// The record commits a different intent.
    Drift,
    /// The record's durable parts match, but it names its operation only by a
    /// seal no later runtime can confirm.
    Unverifiable,
}
