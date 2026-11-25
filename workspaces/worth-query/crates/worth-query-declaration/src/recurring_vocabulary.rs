//! Typed lifecycle and gate of a stock recurring rule row.

/// Lifecycle of a recurring rule row. `Active` rows schedule their due
/// occurrence, `Completed` rows retire after their last fire, `Cancelled`
/// rows stop without firing, and the one `Hub` row scopes every rule.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum WorthQueryRecurringLifecycle {
    Active,
    Completed,
    Cancelled,
    Hub,
}

/// Gate of a recurring rule row: the temporal node evaluates `Ready` rows.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum WorthQueryRecurringGate {
    Ready,
}

crate::__worth_query_vocabulary_binding!(
    /// Identity binding of [`WorthQueryRecurringLifecycle`].
    WorthQueryRecurringLifecycleBinding for WorthQueryRecurringLifecycle =
        "worth.recurring.lifecycle.v1" {
        Active, Completed, Cancelled, Hub,
    }
);
crate::__worth_query_vocabulary_binding!(
    /// Identity binding of [`WorthQueryRecurringGate`].
    WorthQueryRecurringGateBinding for WorthQueryRecurringGate = "worth.recurring.gate.v1" {
        Ready,
    }
);
