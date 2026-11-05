//! Why an idempotency key could not be resolved.

use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial as Provider;
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency_resolution_denial::WorthQueryApplicationIdempotencyResolutionDenialKind;

/// A refusal to resolve an idempotency key. Nothing took effect.
///
/// Match on [`kind`](Self::kind); [`authorization`](Self::authorization) is set
/// only for an authorization refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationIdempotencyResolutionDenial {
    kind: WorthQueryApplicationIdempotencyResolutionDenialKind,
    // Authorization detail is present only on that refusal. Keep its owned
    // causes out of every downstream error's stack representation.
    authorization: Option<Box<WorthQueryOperationAuthorizationDenial>>,
}

impl WorthQueryApplicationIdempotencyResolutionDenial {
    /// The key's earlier commit is proven, but its live receipt is unavailable.
    /// Other denials carry no historical commit observation.
    pub const fn historical_commit(&self) -> Option<super::WorthQueryHistoricalApplicationCommit> {
        match self.kind {
            WorthQueryApplicationIdempotencyResolutionDenialKind::CommittedReceiptNotRetained {
                commit,
            } => Some(super::WorthQueryHistoricalApplicationCommit::observed(
                commit,
            )),
            WorthQueryApplicationIdempotencyResolutionDenialKind::ExecutionDenied(_)
            | WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization(_)
            | WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
            | WorthQueryApplicationIdempotencyResolutionDenialKind::ActiveSnapshotCapacityExhausted { .. }
            | WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionCapacityExhausted
            | WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionIdentityExhausted
            | WorthQueryApplicationIdempotencyResolutionDenialKind::SnapshotIdentityExhausted
            | WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable
            | WorthQueryApplicationIdempotencyResolutionDenialKind::RecordedIntentUnverifiable => None,
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationIdempotencyResolutionDenialKind {
        self.kind
    }

    pub const fn authorization(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        match &self.authorization {
            Some(denial) => Some(denial),
            None => None,
        }
    }

    pub(in crate::domain_computation) fn from_authorization(
        denial: WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization(
                denial.kind(),
            ),
            authorization: Some(Box::new(denial)),
        }
    }

    pub(super) const fn foreign_admission() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission,
            authorization: None,
        }
    }

    pub(super) const fn provider_unavailable() -> Self {
        Self {
            kind: WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable,
            authorization: None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn from_provider(denial: Provider) -> Self {
        let kind = match denial {
            Provider::ExecutionDenied(kind) => WorthQueryApplicationIdempotencyResolutionDenialKind::ExecutionDenied(kind),
            Provider::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => WorthQueryApplicationIdempotencyResolutionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Provider::RetentionCapacityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionCapacityExhausted
            }
            Provider::RetentionIdentityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::RetentionIdentityExhausted
            }
            Provider::SnapshotIdentityExhausted => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::SnapshotIdentityExhausted
            }
            Provider::Unavailable => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable
            }
            Provider::CommittedReceiptNotRetained { commit } => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::CommittedReceiptNotRetained {
                    commit,
                }
            }
            Provider::RecordedIntentUnverifiable => {
                WorthQueryApplicationIdempotencyResolutionDenialKind::RecordedIntentUnverifiable
            }
        };
        Self {
            kind,
            authorization: None,
        }
    }
}

impl std::fmt::Display for WorthQueryApplicationIdempotencyResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application idempotency resolution denied: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for WorthQueryApplicationIdempotencyResolutionDenial {}
